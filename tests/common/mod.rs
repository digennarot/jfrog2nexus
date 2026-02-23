//! Shared integration test utilities

use std::time::Duration;
use reqwest::Client;
use tracing::info;
use wiremock::MockServer;

pub async fn wait_for_service(url: &str) {
    let client = Client::new();
    let max_attempts = 30;
    
    for _ in 0..max_attempts {
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    panic!("Service at {} did not become ready", url);
}

pub struct TestContext {
    pub jfrog_server: Option<MockServer>,
    pub nexus_server: Option<MockServer>,
    pub config_path: String,
    _temp_file: Option<tempfile::NamedTempFile>,
}

impl TestContext {
    pub async fn new() -> Self {
        let temp_file = tempfile::Builder::new()
            .prefix("jfrog2nexus_test_")
            .suffix(".yaml")
            .tempfile()
            .expect("Failed to create temp file");
            
        let config_path = temp_file.path().to_string_lossy().to_string();

        if std::env::var("J2N_REAL_SERVICES").unwrap_or_default() == "true" {
            let jfrog_url = std::env::var("JFROG_URL").unwrap_or_else(|_| "http://localhost:8081/".to_string());
            let nexus_url = std::env::var("NEXUS_URL").unwrap_or_else(|_| "http://localhost:8083/".to_string());
            
            let yaml = format!(
                r#"
jfrog:
  url: "{}"
nexus:
  url: "{}"
mappings:
  - source: "maven-local"
    target: "maven-target"
    type: "maven"
  - source: "docker-local"
    target: "docker-target"
    type: "docker"
  - source: "pypi-local"
    target: "pypi-target"
    type: "pypi"
  - source: "npm-local"
    target: "npm-target"
    type: "npm"
  - source: "nuget-local"
    target: "nuget-target"
    type: "nuget"
  - source: "helm-local"
    target: "helm-target"
    type: "helm"
  - source: "go-local"
    target: "go-target"
    type: "go"
  - source: "raw-local"
    target: "raw-target"
    type: "raw"
"#,
                jfrog_url,
                nexus_url
            );
            std::fs::write(&config_path, yaml).expect("Failed to write real test config");
            
            return Self {
                jfrog_server: None,
                nexus_server: None,
                config_path,
                _temp_file: Some(temp_file),
            };
        }

        let jfrog_server = MockServer::start().await;
        let nexus_server = MockServer::start().await;
        
        let yaml = format!(
            r#"
jfrog:
  url: "{}/"
nexus:
  url: "{}/"
mappings:
  - source: "maven-local"
    target: "maven-target"
    type: "maven"
"#,
            jfrog_server.uri(),
            nexus_server.uri()
        );
        
        std::fs::write(&config_path, yaml).expect("Failed to write test config");

        Self {
            jfrog_server: Some(jfrog_server),
            nexus_server: Some(nexus_server),
            config_path,
            _temp_file: Some(temp_file),
        }
    }
}

pub mod factories {
    pub fn mock_sync_args(config_path: &str) -> Vec<String> {
        vec![
            "jfrog2nexus".to_string(),
            "sync".to_string(),
            "--config".to_string(), config_path.to_string(),
        ]
    }
}
