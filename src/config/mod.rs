use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub jfrog: JfrogConfig,
    pub nexus: NexusConfig,
    pub mappings: Vec<RepositoryMapping>,
    pub proxy: Option<ProxyConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JfrogConfig {
    pub url: Url,
    #[serde(default = "from_env_jfrog_token")]
    pub token: SecretString,
    pub token_file: Option<std::path::PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NexusConfig {
    pub url: Url,
    #[serde(default = "from_env_nexus_token")]
    pub token: SecretString,
    pub token_file: Option<std::path::PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RepositoryMapping {
    pub source: String,
    pub target: String,
    pub r#type: RepoType,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RepoType {
    Docker,
    Maven,
    Pypi,
    Npm,
    Nuget,
    Helm,
    Go,
    Raw,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProxyConfig {
    pub url: Url,
}

fn from_env_jfrog_token() -> SecretString {
    std::env::var("J2N_JFROG_TOKEN").unwrap_or_default().into()
}

fn from_env_nexus_token() -> SecretString {
    std::env::var("J2N_NEXUS_TOKEN").unwrap_or_default().into()
}

impl AppConfig {
    pub fn validate(&mut self) -> anyhow::Result<()> {
        // Enforce HTTPS for upstream connections per architecture NFR7
        // Allow HTTP only if explicitly enabled via environment variable (useful for local testing)
        let allow_http = std::env::var("J2N_ALLOW_HTTP").map(|v| v == "true").unwrap_or(false);

        if !allow_http {
            if self.jfrog.url.scheme() != "https" {
                anyhow::bail!("JFrog URL must use HTTPS: {}. Use J2N_ALLOW_HTTP=true to override for local testing.", self.jfrog.url);
            }
            if self.nexus.url.scheme() != "https" {
                anyhow::bail!("Nexus URL must use HTTPS: {}. Use J2N_ALLOW_HTTP=true to override for local testing.", self.nexus.url);
            }
        }

        // Ensure trailing slashes for reliable joining
        if !self.jfrog.url.path().ends_with('/') {
            self.jfrog.url.set_path(&format!("{}/", self.jfrog.url.path()));
        }
        if !self.nexus.url.path().ends_with('/') {
            self.nexus.url.set_path(&format!("{}/", self.nexus.url.path()));
        }

        // Ensure secrets are provided
        if self.jfrog.token.expose_secret().is_empty() {
            anyhow::bail!("J2N_JFROG_TOKEN is missing or empty");
        }
        if self.nexus.token.expose_secret().is_empty() {
            anyhow::bail!("J2N_NEXUS_TOKEN is missing or empty");
        }

        // Prevent empty migrations
        if self.mappings.is_empty() {
            anyhow::bail!("No repository mappings defined in configuration");
        }

        Ok(())
    }
}

/// Load configuration from a file asynchronously.
/// Uses `serde_yml` (the maintained fork of `serde_yaml`).
pub async fn load_config(path: &str) -> anyhow::Result<AppConfig> {
    let content = tokio::fs::read_to_string(path).await?;
    let mut config: AppConfig = serde_yml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_load_valid_config() {
        let yaml = r#"
jfrog:
  url: "https://jfrog.example.com"
nexus:
  url: "https://nexus.example.com"
mappings:
  - source: "docker-local"
    target: "docker-hosted"
    type: "docker"
"#;
        let path = "test_j2n.yaml";
        tokio::fs::write(path, yaml).await.unwrap();

        std::env::set_var("J2N_JFROG_TOKEN", "jfrog-secret");
        std::env::set_var("J2N_NEXUS_TOKEN", "nexus-secret");

        let config = load_config(path).await.expect("Failed to load config");
        
        assert_eq!(config.jfrog.url.as_str(), "https://jfrog.example.com/");
        assert_eq!(config.jfrog.token.expose_secret(), "jfrog-secret");
        assert_eq!(config.mappings.len(), 1);

        let _ = tokio::fs::remove_file(path).await;
    }

    #[test]
    #[serial]
    fn test_https_enforcement() {
        let yaml = r#"
jfrog:
  url: "http://jfrog.example.com"
nexus:
  url: "https://nexus.example.com"
mappings:
  - source: "a"
    target: "b"
    type: "maven"
"#;
        std::env::set_var("J2N_JFROG_TOKEN", "secret");
        std::env::set_var("J2N_NEXUS_TOKEN", "secret");

        let mut config: AppConfig = serde_yml::from_str(yaml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must use HTTPS"));
    }

    #[test]
    #[serial]
    fn test_empty_mappings() {
        let yaml = r#"
jfrog:
  url: "https://jfrog.example.com"
nexus:
  url: "https://nexus.example.com"
mappings: []
"#;
        std::env::set_var("J2N_JFROG_TOKEN", "secret");
        std::env::set_var("J2N_NEXUS_TOKEN", "secret");

        let mut config: AppConfig = serde_yml::from_str(yaml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No repository mappings"));
    }
}
