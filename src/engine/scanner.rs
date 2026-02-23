use reqwest::Client;
use serde::{Deserialize, Serialize};
use secrecy::ExposeSecret;
use tracing::info;
use crate::config::{JfrogConfig, RepoType};
use crate::engine::TransferError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artifact {
    pub source_repo: String,
    pub target_repo: String,
    pub path: String,
    pub size: u64,
    pub sha256: Option<String>,
    pub repo_type: RepoType,
}

#[derive(Debug, Serialize, Default)]
pub struct SyncPlan {
    pub artifacts: Vec<Artifact>,
    pub total_size: u64,
}

impl SyncPlan {
    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.total_size += artifact.size;
        self.artifacts.push(artifact);
    }
}

pub struct Scanner<'a> {
    client: &'a Client,
    config: &'a JfrogConfig,
}

impl<'a> Scanner<'a> {
    pub fn new(client: &'a Client, config: &'a JfrogConfig) -> Self {
        Self { client, config }
    }

    pub async fn scan_repo(&self, repo_key: &str, target_repo: &str, repo_type: RepoType) -> Result<Vec<Artifact>, TransferError> {
        info!(repo = %repo_key, "Scanning repository");
        
        // For now, using Artifactory File List API which is simple but lacks pagination.
        // In a production tool, AQL would be used for pagination.
        // Ensure we join safely. 
        let path = format!("api/storage/{}?list&deep=1", repo_key);
        let url = self.config.url.join(&path)?;
        
        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.token.expose_secret()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TransferError::JfrogApi(format!("Failed to list artifacts in repo {}: {} - {}", repo_key, status, body)));
        }

        let list_resp: FileListResponse = response.json().await?;
        
        let artifacts = list_resp.files.into_iter()
            .filter(|f| !f.folder)
            .map(|f| Artifact {
                source_repo: repo_key.to_string(),
                target_repo: target_repo.to_string(),
                path: f.uri,
                size: f.size,
                sha256: f.sha256,
                repo_type,
            })
            .collect();
        Ok(artifacts)
    }

    pub async fn build_plan(&self, mappings: &[crate::config::RepositoryMapping]) -> Result<SyncPlan, TransferError> {
        let mut plan = SyncPlan::default();
        for mapping in mappings {
            let artifacts = self.scan_repo(&mapping.source, &mapping.target, mapping.r#type).await?;
            for artifact in artifacts {
                plan.add_artifact(artifact);
            }
        }
        Ok(plan)
    }
}

#[derive(Deserialize)]
struct FileListResponse {
    files: Vec<FileEntry>,
}

#[derive(Deserialize)]
struct FileEntry {
    uri: String,
    size: u64,
    folder: bool,
    sha256: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use url::Url;
    use crate::config::RepoType;

    #[tokio::test]
    async fn test_scan_repo_success() {
        let mock_server = MockServer::start().await;
        
        let response_body = serde_json::json!({
            "files": [
                {
                    "uri": "/com/example/lib/1.0/lib-1.0.jar",
                    "size": 1024,
                    "folder": false,
                    "sha256": "abc123"
                },
                {
                    "uri": "/com/example/lib/1.0",
                    "size": 0,
                    "folder": true,
                    "sha256": null
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/storage/maven-local"))
            .and(query_param("list", ""))
            .and(query_param("deep", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let config = JfrogConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "test-token".into(),
            token_file: None,
        };
        let client = Client::new();
        let scanner = Scanner::new(&client, &config);

        let artifacts = scanner.scan_repo("maven-local", "maven-target", RepoType::Maven).await.unwrap();
        
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].path, "/com/example/lib/1.0/lib-1.0.jar");
        assert_eq!(artifacts[0].source_repo, "maven-local");
        assert_eq!(artifacts[0].target_repo, "maven-target");
        assert_eq!(artifacts[0].size, 1024);
    }

    #[tokio::test]
    async fn test_build_plan() {
        let mock_server = MockServer::start().await;
        
        let response_body = serde_json::json!({
            "files": [
                {
                    "uri": "/artifact1",
                    "size": 500,
                    "folder": false,
                    "sha256": "sha1"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/storage/repo1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body.clone()))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/storage/repo2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let config = JfrogConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "test".into(),
            token_file: None,
        };
        let client = Client::new();
        let scanner = Scanner::new(&client, &config);

        let mappings = vec![
            crate::config::RepositoryMapping {
                source: "repo1".to_string(),
                target: "target1".to_string(),
                r#type: RepoType::Maven,
            },
            crate::config::RepositoryMapping {
                source: "repo2".to_string(),
                target: "target2".to_string(),
                r#type: RepoType::Maven,
            },
        ];

        let plan = scanner.build_plan(&mappings).await.unwrap();
        
        assert_eq!(plan.artifacts.len(), 2);
        assert_eq!(plan.total_size, 1000);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PyPI
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_scan_repo_pypi() {
        let mock_server = MockServer::start().await;

        let response_body = serde_json::json!({
            "files": [
                {
                    "uri": "/packages/mylib/mylib-1.0-py3-none-any.whl",
                    "size": 2048,
                    "folder": false,
                    "sha256": "deadbeef"
                },
                {
                    "uri": "/packages/mylib/mylib-1.0.tar.gz",
                    "size": 4096,
                    "folder": false,
                    "sha256": "cafebabe"
                },
                {
                    "uri": "/packages/mylib",
                    "size": 0,
                    "folder": true,
                    "sha256": null
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/storage/pypi-local"))
            .and(query_param("list", ""))
            .and(query_param("deep", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let config = JfrogConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "test-token".into(),
            token_file: None,
        };
        let client = Client::new();
        let scanner = Scanner::new(&client, &config);

        let artifacts = scanner.scan_repo("pypi-local", "pypi-target", RepoType::Pypi).await.unwrap();

        // Folder entry should be filtered out
        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].path, "/packages/mylib/mylib-1.0-py3-none-any.whl");
        assert_eq!(artifacts[0].size, 2048);
        assert_eq!(artifacts[1].path, "/packages/mylib/mylib-1.0.tar.gz");
        assert_eq!(artifacts[1].size, 4096);
        // Both should target pypi-target repo
        assert!(artifacts.iter().all(|a| a.target_repo == "pypi-target"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // npm
    // ─────────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_scan_repo_npm() {
        let mock_server = MockServer::start().await;

        let response_body = serde_json::json!({
            "files": [
                {
                    "uri": "/@myorg/mylib/-/mylib-1.0.0.tgz",
                    "size": 512,
                    "folder": false,
                    "sha256": "aabbcc"
                },
                {
                    "uri": "/@myorg/mylib",
                    "size": 0,
                    "folder": true,
                    "sha256": null
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/storage/npm-local"))
            .and(query_param("list", ""))
            .and(query_param("deep", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let config = JfrogConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "test-token".into(),
            token_file: None,
        };
        let client = Client::new();
        let scanner = Scanner::new(&client, &config);

        let artifacts = scanner.scan_repo("npm-local", "npm-target", RepoType::Npm).await.unwrap();

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].path, "/@myorg/mylib/-/mylib-1.0.0.tgz");
        assert_eq!(artifacts[0].size, 512);
        assert_eq!(artifacts[0].target_repo, "npm-target");
    }
}
