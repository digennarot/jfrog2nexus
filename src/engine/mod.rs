use crate::config::{JfrogConfig, NexusConfig};
use anyhow::Result;
use reqwest::Client;
use secrecy::ExposeSecret;
pub mod hasher;
pub mod retry;
pub mod scanner;
pub mod state_store;
pub mod target_mapper;
pub mod throttler;
pub mod transfer;
use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum TransferError {
    #[error("JFrog API error: {0}")]
    JfrogApi(String),
    #[error("Nexus API error: {0}")]
    NexusApi(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid configuration: {0}")]
    Config(String),
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Unauthorized access to {0}")]
    Unauthorized(String),
}

pub fn create_client(proxy_url: Option<&url::Url>) -> Result<Client, TransferError> {
    let mut builder = Client::builder().user_agent("jfrog2nexus/0.1.0 (Enterprise Migration Tool)");

    if let Some(proxy) = proxy_url {
        debug!(url = %proxy, "Configuring HTTP proxy");
        builder = builder.proxy(reqwest::Proxy::all(proxy.as_str())?);
    }

    Ok(builder.build()?)
}

pub async fn check_jfrog_connectivity(
    config: &JfrogConfig,
    client: &Client,
) -> Result<(), TransferError> {
    debug!(url = %config.url, "Checking JFrog connectivity");

    // JFrog Artifactory ping endpoint
    let ping_url = config.url.join("api/system/ping")?;

    let response = client
        .get(ping_url)
        .header(
            "Authorization",
            format!("Bearer {}", config.token.expose_secret()),
        )
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        Err(TransferError::JfrogApi(format!("{} - {}", status, body)))
    }
}

pub async fn check_nexus_connectivity(
    config: &NexusConfig,
    client: &Client,
) -> Result<(), TransferError> {
    debug!(url = %config.url, "Checking Nexus connectivity");

    // Nexus Repository Manager status endpoint
    let status_url = config.url.join("service/rest/v1/status")?;

    let response = client
        .get(status_url)
        .header(
            "Authorization",
            format!("Bearer {}", config.token.expose_secret()),
        )
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        Err(TransferError::NexusApi(format!("{} - {}", status, body)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_check_jfrog_connectivity_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/system/ping"))
            .and(header("Authorization", "Bearer jfrog-token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let config = JfrogConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "jfrog-token".into(),
            token_file: None,
        };
        let client = Client::new();

        let result = check_jfrog_connectivity(&config, &client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_jfrog_connectivity_failure() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/system/ping"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        let config = JfrogConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "bad-token".into(),
            token_file: None,
        };
        let client = Client::new();

        let result = check_jfrog_connectivity(&config, &client).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("401 Unauthorized"));
    }

    #[tokio::test]
    async fn test_check_nexus_connectivity_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/service/rest/v1/status"))
            .and(header("Authorization", "Bearer nexus-token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let config = NexusConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "nexus-token".into(),
            token_file: None,
        };
        let client = Client::new();

        let result = check_nexus_connectivity(&config, &client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_nexus_connectivity_failure() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/service/rest/v1/status"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&mock_server)
            .await;

        let config = NexusConfig {
            url: Url::parse(&mock_server.uri()).unwrap(),
            token: "nexus-token".into(),
            token_file: None,
        };
        let client = Client::new();

        let result = check_nexus_connectivity(&config, &client).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("503 Service Unavailable"));
    }
}
