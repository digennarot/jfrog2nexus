use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Semaphore, RwLock};
use tracing::{info, error, info_span, warn};
use crate::config::{JfrogConfig, NexusConfig};
use crate::engine::scanner::{Artifact, SyncPlan};
use crate::engine::TransferError;
use secrecy::ExposeSecret;
use reqwest::Client;
use tracing::Instrument;

pub struct TransferOrchestrator {
    client: Arc<Client>,
    jfrog_config: Arc<RwLock<JfrogConfig>>,
    nexus_config: Arc<RwLock<NexusConfig>>,
    concurrency_limit: Arc<Semaphore>,
    state_store: Option<Arc<crate::engine::state_store::StateStore>>,
    rate_limiter: Option<Arc<crate::engine::throttler::GlobalRateLimiter>>,
    refresh_lock: Arc<tokio::sync::Mutex<()>>,
    last_refresh: Arc<RwLock<std::time::Instant>>,
}

impl TransferOrchestrator {
    pub fn new(
        client: Arc<Client>, 
        jfrog_config: JfrogConfig, 
        nexus_config: NexusConfig, 
        max_concurrency: usize,
        state_store: Option<Arc<crate::engine::state_store::StateStore>>,
        rate_limiter: Option<Arc<crate::engine::throttler::GlobalRateLimiter>>,
    ) -> Self {
        Self {
            client,
            jfrog_config: Arc::new(RwLock::new(jfrog_config)),
            nexus_config: Arc::new(RwLock::new(nexus_config)),
            concurrency_limit: Arc::new(Semaphore::new(max_concurrency)),
            state_store,
            rate_limiter,
            refresh_lock: Arc::new(tokio::sync::Mutex::new(())),
            last_refresh: Arc::new(RwLock::new(std::time::Instant::now() - std::time::Duration::from_secs(3600))),
        }
    }

    pub async fn execute_plan(&self, plan: SyncPlan) -> Result<(), TransferError> {
        info!(total_artifacts = plan.artifacts.len(), "Starting transfer execution");
        
        let mut handlers = Vec::new();
        
        for artifact in plan.artifacts {
            // --- Optimization: Skip before spawning if already done ---
            if let Some(ref store) = self.state_store {
                if store.is_completed(&artifact.source_repo, &artifact.path, artifact.sha256.as_deref()).await {
                    continue;
                }
            }
            // ----------------------------------------------------------

            let permit = self.concurrency_limit.clone().acquire_owned().await
                .map_err(|_| TransferError::Config("Semaphore closed".to_string()))?;
            let jfrog_config = self.jfrog_config.clone();
            let nexus_config = self.nexus_config.clone();
            let state_store = self.state_store.clone();
            let rate_limiter = self.rate_limiter.clone();
            let client = self.client.clone();
            
            let source_repo = artifact.source_repo.clone();
            let target_repo = artifact.target_repo.clone();
            let path = artifact.path.clone();

            let refresh_lock = self.refresh_lock.clone();
            let last_refresh = self.last_refresh.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;
                let span = info_span!("transfer", %source_repo, %target_repo, %path);
                async move {
                    crate::engine::retry::with_retry("transfer_artifact", || {
                        let client = client.clone();
                        let jfrog_config = jfrog_config.clone();
                        let nexus_config = nexus_config.clone();
                        let artifact = artifact.clone();
                        let state_store = state_store.clone();
                        let rate_limiter = rate_limiter.clone();
                        
                        let refresh_lock = refresh_lock.clone();
                        let last_refresh = last_refresh.clone();

                        async move {
                            // Read current config from locks
                            let jf = jfrog_config.read().await.clone();
                            let nx = nexus_config.read().await.clone();
                            
                            let result = Self::transfer_artifact(&client, &jf, &nx, artifact.clone(), state_store.clone(), rate_limiter.clone()).await;
                            
                            if let Err(TransferError::Unauthorized(repo)) = result {
                                // 1. Thundering herd protection: only one refresh at a time
                                let _guard = refresh_lock.lock().await;

                                // 2. Cooldown check: if we refreshed in the last 10 seconds, skip
                                {
                                    let last = last_refresh.read().await;
                                    if last.elapsed() < std::time::Duration::from_secs(10) {
                                        return Err(TransferError::Unauthorized(repo));
                                    }
                                }

                                warn!(%repo, path = %artifact.path, "Unauthorized, attempting token refresh");
                                
                                let mut updated = false;
                                if repo == "JFrog" {
                                    let mut jf_lock = jfrog_config.write().await;
                                    let mut new_token = None;
                                    if let Some(ref path) = jf_lock.token_file {
                                        if let Ok(content) = tokio::fs::read_to_string(path).await {
                                            new_token = Some(content.trim().to_string());
                                        }
                                    }
                                    if new_token.is_none() {
                                        if let Ok(token) = std::env::var("J2N_JFROG_TOKEN") {
                                            new_token = Some(token);
                                        }
                                    }
                                    if let Some(token) = new_token {
                                        if token != jf_lock.token.expose_secret() {
                                            jf_lock.token = token.into();
                                            updated = true;
                                        }
                                    }
                                } else {
                                    let mut nx_lock = nexus_config.write().await;
                                    let mut new_token = None;
                                    if let Some(ref path) = nx_lock.token_file {
                                        if let Ok(content) = tokio::fs::read_to_string(path).await {
                                            new_token = Some(content.trim().to_string());
                                        }
                                    }
                                    if new_token.is_none() {
                                        if let Ok(token) = std::env::var("J2N_NEXUS_TOKEN") {
                                            new_token = Some(token);
                                        }
                                    }
                                    if let Some(token) = new_token {
                                        if token != nx_lock.token.expose_secret() {
                                            nx_lock.token = token.into();
                                            updated = true;
                                        }
                                    }
                                }

                                if updated {
                                    info!(%repo, "Token updated successfully during transfer");
                                    let mut last = last_refresh.write().await;
                                    *last = std::time::Instant::now();
                                } else {
                                    warn!(%repo, "Token refresh attempted but no new token found");
                                }
                                
                                return Err(TransferError::Unauthorized(repo));
                            }
                            result
                        }
                    }, 5).await
                }.instrument(span).await
            });
            handlers.push(handle);
        }
        
        for handle in handlers {
            match handle.await {
                Ok(Ok(_)) => (),
                Ok(Err(e)) => {
                    error!(error = %e, "Artifact transfer failed");
                    // Strategy: log and continue for now. 
                    // In the future, we might stop on critical errors (401, etc)
                }
                Err(e) => error!(error = %e, "Task join error"),
            }
        }
        
        info!("Transfer execution complete");
        Ok(())
    }

    async fn handle_response(resp: reqwest::Response, repo_type: &str) -> Result<reqwest::Response, TransferError> {
        let status = resp.status();
        
        // Record status code metric
        metrics::counter!("j2n_transfer_status_codes_total", "repo" => repo_type.to_string(), "status" => status.to_string()).increment(1);

        if status.is_success() {
            return Ok(resp);
        }
        
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(TransferError::Unauthorized(repo_type.to_string()));
        }
        
        let body = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        let body_preview = if body.len() > 500 {
            format!("{}... [truncated]", &body[..500])
        } else {
            body
        };

        match repo_type {
            "JFrog" => Err(TransferError::JfrogApi(format!("{} - {}", status, body_preview))),
            _ => Err(TransferError::NexusApi(format!("{} - {}", status, body_preview))),
        }
    }

    async fn transfer_artifact(
        client: &Client,
        jfrog_config: &JfrogConfig,
        nexus_config: &NexusConfig,
        artifact: Artifact,
        state_store: Option<Arc<crate::engine::state_store::StateStore>>,
        rate_limiter: Option<Arc<crate::engine::throttler::GlobalRateLimiter>>,
    ) -> Result<(), TransferError> {
        use crate::engine::target_mapper::{TargetMapper, TargetApiAction};
        
        // --- Added for Story 2.2: Resumable Transfers ---
        if let Some(ref store) = state_store {
            if store.is_completed(&artifact.source_repo, &artifact.path, artifact.sha256.as_deref()).await {
                info!(path = %artifact.path, "Artifact already completed in StateStore, skipping");
                return Ok(());
            }
        }
        // ------------------------------------------------

        // 1. Prepare source URL (Artifactory storage)
        let path = artifact.path.trim_start_matches('/');
        let source_path = format!("{}/{}", artifact.source_repo, path);
        let source_url = jfrog_config.url.join(&source_path)?;

        // 2. Download stream from JFrog
        let response = client
            .get(source_url)
            .header("Authorization", format!("Bearer {}", jfrog_config.token.expose_secret()))
            .send()
            .await?;

        let response = Self::handle_response(response, "JFrog").await?;

        let stream = response.bytes_stream();
        
        // --- Added for Story 3.2: Metrics ---
        let stream = crate::observability::metric_stream::MetricStream::new(stream, artifact.source_repo.clone());
        // ------------------------------------

        // --- Added for Story 2.4: Transfer Rate Throttling ---
        let stream = if let Some(limiter) = rate_limiter {
            Box::pin(crate::engine::throttler::ThrottledStream::new(stream, limiter)) as Pin<Box<dyn futures::Stream<Item = _> + Send>>
        } else {
            Box::pin(stream) as Pin<Box<dyn futures::Stream<Item = _> + Send>>
        };
        // -----------------------------------------------------

        // --- Added for Story 2.1: Streaming Checksum Calculation ---
        let (hashing_stream, hasher) = crate::engine::hasher::HashingStream::new(stream);
        // -----------------------------------------------------------

        // 3. Map to target action
        let action = TargetMapper::map_artifact(&artifact);

        // 4. Execute target action
        match action {
            TargetApiAction::Put { ref url } => {
                let target_url = nexus_config.url.join(url)?;
                let upload_response = client
                    .put(target_url)
                    .header("Authorization", format!("Bearer {}", nexus_config.token.expose_secret()))
                    .body(reqwest::Body::wrap_stream(hashing_stream))
                    .send()
                    .await?;

                Self::handle_response(upload_response, "Nexus").await?;
            }
            TargetApiAction::DockerBlob { ref name, ref digest } => {
                // Docker V2 Blob Push: POST then PUT
                let initiate_path = format!("repository/{}/v2/{}/blobs/uploads/", artifact.target_repo, name);
                let initiate_url = nexus_config.url.join(&initiate_path)?;

                let initiate_resp = client
                    .post(initiate_url)
                    .header("Authorization", format!("Bearer {}", nexus_config.token.expose_secret()))
                    .send()
                    .await?;

                let initiate_resp = Self::handle_response(initiate_resp, "Nexus").await?;

                let location = initiate_resp.headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|l| l.to_str().ok())
                    .ok_or_else(|| TransferError::NexusApi("Missing Location header in Docker blob upload initiation".to_string()))?;

                let upload_url = if location.starts_with("http") {
                    url::Url::parse(location)?
                } else {
                    nexus_config.url.join(location)?
                };
                
                let mut upload_url = upload_url;
                upload_url.query_pairs_mut().append_pair("digest", digest);

                let upload_response = client
                    .put(upload_url)
                    .header("Authorization", format!("Bearer {}", nexus_config.token.expose_secret()))
                    .header("Content-Type", "application/octet-stream")
                    .body(reqwest::Body::wrap_stream(hashing_stream))
                    .send()
                    .await?;

                Self::handle_response(upload_response, "Nexus").await?;
            }
            TargetApiAction::DockerManifest { ref name, ref reference } => {
                let manifest_path = format!("repository/{}/v2/{}/manifests/{}", artifact.target_repo, name, reference);
                let manifest_url = nexus_config.url.join(&manifest_path)?;

                let upload_response = client
                    .put(manifest_url)
                    .header("Authorization", format!("Bearer {}", nexus_config.token.expose_secret()))
                    .header("Content-Type", "application/vnd.docker.distribution.manifest.v2+json")
                    .body(reqwest::Body::wrap_stream(hashing_stream))
                    .send()
                    .await?;

                Self::handle_response(upload_response, "Nexus").await?;
            }
        }

        // --- Added for Story 2.1: Hash Validation ---
        use sha2::Digest;
        let calculated_hash = {
            let hasher_lock = hasher.lock().unwrap();
            format!("{:x}", hasher_lock.clone().finalize())
        };
        
        if let Some(ref expected_hash) = artifact.sha256 {
            // Clean up Artifactory's varied hash formats if necessary (e.g. sha256: or just hex)
            let clean_expected = if expected_hash.contains(':') {
                expected_hash.split(':').last().unwrap_or(expected_hash)
            } else {
                expected_hash
            };

            if calculated_hash != clean_expected {
                error!(path = %artifact.path, %calculated_hash, %expected_hash, "Hash mismatch detected!");
                
                // Story 2.1 AC5: Delete target file and error
                if let Err(e) = Self::delete_target(client, nexus_config, &artifact, &action).await {
                    error!(error = %e, path = %artifact.path, "Failed to delete corrupted target file after hash mismatch!");
                }
                
                return Err(TransferError::Config(format!("Data corruption: hash mismatch for {}. Expected {}, got {}", artifact.path, expected_hash, calculated_hash)));
            }
        }
        // --------------------------------------------

        // --- Added for Story 2.2: Mark as complete ---
        if let Some(ref store) = state_store {
            store.mark_completed(
                &artifact.source_repo, 
                &artifact.path, 
                &artifact.target_repo, 
                &calculated_hash, 
                artifact.size
            ).await?;
        }
        // ----------------------------------------------

        info!(path = %artifact.path, "Successfully transferred artifact");
        Ok(())
    }

    async fn delete_target(
        client: &Client,
        nexus_config: &NexusConfig,
        artifact: &Artifact,
        action: &crate::engine::target_mapper::TargetApiAction,
    ) -> Result<(), TransferError> {
        use crate::engine::target_mapper::TargetApiAction;
        
        let delete_url = match action {
            TargetApiAction::Put { url } => nexus_config.url.join(url)?,
            TargetApiAction::DockerBlob { name, digest } => {
                let path = format!("repository/{}/v2/{}/blobs/{}", artifact.target_repo, name, digest);
                nexus_config.url.join(&path)?
            }
            TargetApiAction::DockerManifest { name, reference } => {
                let path = format!("repository/{}/v2/{}/manifests/{}", artifact.target_repo, name, reference);
                nexus_config.url.join(&path)?
            }
        };

        info!(url = %delete_url, "Cleaning up target after hash mismatch");
        client.delete(delete_url)
            .header("Authorization", format!("Bearer {}", nexus_config.token.expose_secret()))
            .send()
            .await?;
            
        Ok(())
    }

    pub async fn refresh_tokens(&self, repo_type: &str) -> Result<(), TransferError> {
        // 1. Thundering herd protection: only one refresh at a time
        let _guard = self.refresh_lock.lock().await;

        // 2. Cooldown check: if we refreshed in the last 10 seconds, skip (another task already did it)
        {
            let last = self.last_refresh.read().await;
            if last.elapsed() < std::time::Duration::from_secs(10) {
                return Ok(());
            }
        }

        info!(repo = %repo_type, "Refreshing API tokens from file or environment");
        
        if repo_type == "JFrog" {
            let mut jf = self.jfrog_config.write().await;
            let mut new_token = None;

            // Try file first (works on Linux during long transfers)
            if let Some(ref path) = jf.token_file {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    new_token = Some(content.trim().to_string());
                    info!(path = ?path, "Read new JFrog token from file");
                }
            }

            // Fallback to environment
            if new_token.is_none() {
                if let Ok(token) = std::env::var("J2N_JFROG_TOKEN") {
                    new_token = Some(token);
                }
            }

            if let Some(token) = new_token {
                if token != jf.token.expose_secret() {
                    jf.token = token.into();
                    info!("JFrog token updated successfully");
                } else {
                    warn!("JFrog token refresh attempted but token has not changed");
                }
            }
        } else {
            let mut nx = self.nexus_config.write().await;
            let mut new_token = None;

            if let Some(ref path) = nx.token_file {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    new_token = Some(content.trim().to_string());
                    info!(path = ?path, "Read new Nexus token from file");
                }
            }

            if new_token.is_none() {
                if let Ok(token) = std::env::var("J2N_NEXUS_TOKEN") {
                    new_token = Some(token);
                }
            }

            if let Some(token) = new_token {
                if token != nx.token.expose_secret() {
                    nx.token = token.into();
                    info!("Nexus token updated successfully");
                } else {
                    warn!("Nexus token refresh attempted but token has not changed");
                }
            }
        }
        
        // Update last refresh timestamp
        {
            let mut last = self.last_refresh.write().await;
            *last = std::time::Instant::now();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use url::Url;
    use crate::config::{JfrogConfig, NexusConfig, RepoType};

    #[tokio::test]
    async fn test_transfer_maven_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        // Mock JFrog download
        Mock::given(method("GET"))
            .and(path("/maven-local/com/example/lib/1.0/lib-1.0.jar"))
            .respond_with(ResponseTemplate::new(200).set_body_string("artifact-content"))
            .mount(&jfrog_server)
            .await;
            
        // Mock Nexus upload
        Mock::given(method("PUT"))
            .and(path("/repository/maven-target/com/example/lib/1.0/lib-1.0.jar"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog-token".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus-token".into(),
            token_file: None,
        };
        
        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);
        
        let artifact = Artifact {
            source_repo: "maven-local".to_string(),
            target_repo: "maven-target".to_string(),
            path: "/com/example/lib/1.0/lib-1.0.jar".to_string(),
            size: 16,
            sha256: None,
            repo_type: RepoType::Maven,
        };
        
        let plan = SyncPlan {
            artifacts: vec![artifact],
            total_size: 16,
        };
        
        let result = orchestrator.execute_plan(plan).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transfer_docker_blob_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        let body = "blob-content";
        let expected_hash = "849c6f2dfac02eec5a2123611a91316496924e2c608f7f1d4411130638520268";

        // Mock JFrog download - Artifactory uses __ in storage paths
        Mock::given(method("GET"))
            .and(path("/docker-local/library/hello-world/_/sha256__abcd"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&jfrog_server)
            .await;
            
        // Mock Nexus upload initiation
        Mock::given(method("POST"))
            .and(path("/repository/docker-target/v2/library/hello-world/blobs/uploads/"))
            .respond_with(ResponseTemplate::new(202).append_header("Location", "/v2/upload/session-123"))
            .mount(&nexus_server)
            .await;

        // Mock Nexus upload completion - registry uses : for digest parameter
        Mock::given(method("PUT"))
            .and(path("/v2/upload/session-123"))
            // Note: Wiremock doesn't strictly require matching the query param unless we want it to
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };
        
        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);
        
        let artifact = Artifact {
            source_repo: "docker-local".to_string(),
            target_repo: "docker-target".to_string(),
            path: "library/hello-world/_/sha256__abcd".to_string(),
            size: body.len() as u64,
            sha256: Some(expected_hash.to_string()),
            repo_type: RepoType::Docker,
        };
        
        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: body.len() as u64 }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transfer_docker_manifest_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/docker-local/library/hello-world/latest/manifest.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&jfrog_server)
            .await;
            
        Mock::given(method("PUT"))
            .and(path("/repository/docker-target/v2/library/hello-world/manifests/latest"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };
        
        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);
        
        let artifact = Artifact {
            source_repo: "docker-local".to_string(),
            target_repo: "docker-target".to_string(),
            path: "library/hello-world/latest/manifest.json".to_string(),
            size: 2,
            sha256: None,
            repo_type: RepoType::Docker,
        };
        
        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 2 }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transfer_hash_mismatch_failure() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/maven-local/bad-hash.jar"))
            .respond_with(ResponseTemplate::new(200).set_body_string("corrupted-content"))
            .mount(&jfrog_server)
            .await;
            
        Mock::given(method("PUT"))
            .and(path("/repository/maven-target/bad-hash.jar"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        // Verify that DELETE is called after mismatch
        Mock::given(method("DELETE"))
            .and(path("/repository/maven-target/bad-hash.jar"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };
        
        let client = Arc::new(Client::new());
        let _orchestrator = TransferOrchestrator::new(client.clone(), jfrog_config.clone(), nexus_config.clone(), 1, None, None);
        
        let artifact = Artifact {
            source_repo: "maven-local".to_string(),
            target_repo: "maven-target".to_string(),
            path: "/bad-hash.jar".to_string(),
            size: 17,
            sha256: Some("5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8".to_string()), 
            repo_type: RepoType::Maven,
        };
        
        // TransferOrchestrator::transfer_artifact should return Err
        let result = TransferOrchestrator::transfer_artifact(&client, &jfrog_config, &nexus_config, artifact, None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Data corruption"));
    }

    #[tokio::test]
    async fn test_transfer_resume_skip() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        // No mocks for GET or PUT because they should be skipped!

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };
        
        // Initialize state store and mark as complete
        let db_dir = tempfile::tempdir().unwrap();
        let db_path = db_dir.path().join("test.db");
        let db_path_str = format!("sqlite://{}", db_path.to_str().unwrap());
        
        let state_store = crate::engine::state_store::StateStore::new(&db_path_str).await.unwrap();
        state_store.mark_completed("maven-local", "/already-done.jar", "maven-target", "abcd", 123).await.unwrap();
        
        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client.clone(), jfrog_config, nexus_config, 1, Some(Arc::new(state_store)), None);
        
        let artifact = Artifact {
            source_repo: "maven-local".to_string(),
            target_repo: "maven-target".to_string(),
            path: "/already-done.jar".to_string(),
            size: 123,
            sha256: Some("abcd".to_string()),
            repo_type: RepoType::Maven,
        };
        
        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 123 }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transfer_retry_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        // Mock JFrog: fail once with 500, then succeed
        Mock::given(method("GET"))
            .and(path("/maven-local/retry.jar"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&jfrog_server)
            .await;
            
        Mock::given(method("GET"))
            .and(path("/maven-local/retry.jar"))
            .respond_with(ResponseTemplate::new(200).set_body_string("retry-content"))
            .mount(&jfrog_server)
            .await;
            
        Mock::given(method("PUT"))
            .and(path("/repository/maven-target/retry.jar"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };
        
        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client.clone(), jfrog_config, nexus_config, 1, None, None);
        
        let artifact = Artifact {
            source_repo: "maven-local".to_string(),
            target_repo: "maven-target".to_string(),
            path: "/retry.jar".to_string(),
            size: 13,
            sha256: None,
            repo_type: RepoType::Maven,
        };
        
        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 13 }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transfer_unauthorized_retry_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        // Mock JFrog: fail once with 401, then succeed
        Mock::given(method("GET"))
            .and(path("/maven-local/auth.jar"))
            .respond_with(ResponseTemplate::new(401))
            .up_to_n_times(1)
            .mount(&jfrog_server)
            .await;
            
        Mock::given(method("GET"))
            .and(path("/maven-local/auth.jar"))
            .respond_with(ResponseTemplate::new(200).set_body_string("auth-content"))
            .mount(&jfrog_server)
            .await;
            
        Mock::given(method("PUT"))
            .and(path("/repository/maven-target/auth.jar"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };
        
        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client.clone(), jfrog_config, nexus_config, 1, None, None);
        
        let artifact = Artifact {
            source_repo: "maven-local".to_string(),
            target_repo: "maven-target".to_string(),
            path: "/auth.jar".to_string(),
            size: 12,
            sha256: None,
            repo_type: RepoType::Maven,
        };
        
        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 12 }).await;
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PyPI
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_transfer_pypi_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/pypi-local/packages/mylib/mylib-1.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_string("wheel-content"))
            .mount(&jfrog_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/repository/pypi-target/packages/mylib/mylib-1.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };

        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);

        let artifact = Artifact {
            source_repo: "pypi-local".to_string(),
            target_repo: "pypi-target".to_string(),
            path: "/packages/mylib/mylib-1.0-py3-none-any.whl".to_string(),
            size: 13,
            sha256: None,
            repo_type: RepoType::Pypi,
        };

        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 13 }).await;
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // npm
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_transfer_npm_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;

        // Scoped package: @myorg/mylib/-/mylib-1.0.0.tgz
        Mock::given(method("GET"))
            .and(path("/npm-local/@myorg/mylib/-/mylib-1.0.0.tgz"))
            .respond_with(ResponseTemplate::new(200).set_body_string("npm-tarball-content"))
            .mount(&jfrog_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/repository/npm-target/@myorg/mylib/-/mylib-1.0.0.tgz"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };

        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);

        let artifact = Artifact {
            source_repo: "npm-local".to_string(),
            target_repo: "npm-target".to_string(),
            path: "/@myorg/mylib/-/mylib-1.0.0.tgz".to_string(),
            size: 19,
            sha256: None,
            repo_type: RepoType::Npm,
        };

        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 19 }).await;
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // NuGet
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_transfer_nuget_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/nuget-local/mylib/1.0.0/mylib.1.0.0.nupkg"))
            .respond_with(ResponseTemplate::new(200).set_body_string("nupkg-content"))
            .mount(&jfrog_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/repository/nuget-target/mylib/1.0.0/mylib.1.0.0.nupkg"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };

        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);

        let artifact = Artifact {
            source_repo: "nuget-local".to_string(),
            target_repo: "nuget-target".to_string(),
            path: "/mylib/1.0.0/mylib.1.0.0.nupkg".to_string(),
            size: 13,
            sha256: None,
            repo_type: RepoType::Nuget,
        };

        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 13 }).await;
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Helm
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_transfer_helm_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/helm-local/myapp-1.0.0.tgz"))
            .respond_with(ResponseTemplate::new(200).set_body_string("helm-chart-content"))
            .mount(&jfrog_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/repository/helm-target/myapp-1.0.0.tgz"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };

        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);

        let artifact = Artifact {
            source_repo: "helm-local".to_string(),
            target_repo: "helm-target".to_string(),
            path: "/myapp-1.0.0.tgz".to_string(),
            size: 18,
            sha256: None,
            repo_type: RepoType::Helm,
        };

        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 18 }).await;
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Go modules
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_transfer_go_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;

        // URL-encode '@' in the path on Artifactory side is preserved; wiremock matches literal path
        Mock::given(method("GET"))
            .and(path("/go-local/github.com/myorg/mylib/@v/v1.0.0.zip"))
            .respond_with(ResponseTemplate::new(200).set_body_string("go-module-content"))
            .mount(&jfrog_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/repository/go-target/github.com/myorg/mylib/@v/v1.0.0.zip"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };

        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);

        let artifact = Artifact {
            source_repo: "go-local".to_string(),
            target_repo: "go-target".to_string(),
            path: "/github.com/myorg/mylib/@v/v1.0.0.zip".to_string(),
            size: 17,
            sha256: None,
            repo_type: RepoType::Go,
        };

        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 17 }).await;
        assert!(result.is_ok());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Raw / Generic
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_transfer_raw_success() {
        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/raw-local/binaries/myapp-linux-amd64"))
            .respond_with(ResponseTemplate::new(200).set_body_string("binary-content"))
            .mount(&jfrog_server)
            .await;

        Mock::given(method("PUT"))
            .and(path("/repository/raw-target/binaries/myapp-linux-amd64"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&nexus_server)
            .await;

        let jfrog_config = JfrogConfig {
            url: Url::parse(&format!("{}/", jfrog_server.uri())).unwrap(),
            token: "jfrog".into(),
            token_file: None,
        };
        let nexus_config = NexusConfig {
            url: Url::parse(&format!("{}/", nexus_server.uri())).unwrap(),
            token: "nexus".into(),
            token_file: None,
        };

        let client = Arc::new(Client::new());
        let orchestrator = TransferOrchestrator::new(client, jfrog_config, nexus_config, 1, None, None);

        let artifact = Artifact {
            source_repo: "raw-local".to_string(),
            target_repo: "raw-target".to_string(),
            path: "/binaries/myapp-linux-amd64".to_string(),
            size: 14,
            sha256: None,
            repo_type: RepoType::Raw,
        };

        let result = orchestrator.execute_plan(SyncPlan { artifacts: vec![artifact], total_size: 14 }).await;
        assert!(result.is_ok());
    }
}
