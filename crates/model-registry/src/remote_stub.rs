/// Remote model backend stub
///
/// This module provides placeholder implementations for remote model backends
/// (OpenAI-compatible APIs, etc.). This is not used in v1 but provides the
/// structure for future remote model integration.
use async_trait::async_trait;
use bodhya_core::{BackendType, Error, ModelBackend, ModelRequest, ModelResponse, Result};

/// Configuration for remote model backend
#[derive(Clone, Debug)]
pub struct RemoteBackendConfig {
    /// Model ID/name
    pub model_id: String,
    /// API base URL
    pub api_base: String,
    /// API key (optional)
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for RemoteBackendConfig {
    fn default() -> Self {
        Self {
            model_id: String::from("unknown"),
            api_base: String::from("http://localhost:8000/v1"),
            api_key: None,
            timeout_seconds: 30,
        }
    }
}

/// Stub implementation for remote model backends
///
/// This is a placeholder for future remote model integration. In v1, attempting
/// to use this will return an error.
pub struct RemoteBackend {
    config: RemoteBackendConfig,
}

impl RemoteBackend {
    /// Create a new remote backend
    pub fn new(config: RemoteBackendConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ModelBackend for RemoteBackend {
    fn id(&self) -> &str {
        &self.config.model_id
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Remote
    }

    async fn generate(&self, _request: ModelRequest) -> Result<ModelResponse> {
        // Remote backends are not supported in v1
        Err(Error::EngagementViolation(
            "Remote model backends are not supported in v1. \
             Please use local models with EngagementMode::Minimum."
                .to_string(),
        ))
    }

    async fn health_check(&self) -> Result<bool> {
        // Remote backends are not available in v1
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bodhya_core::ModelRole;

    #[test]
    fn test_remote_backend_creation() {
        let config = RemoteBackendConfig {
            model_id: "gpt-4".to_string(),
            api_base: "https://api.openai.com/v1".to_string(),
            api_key: Some("sk-test".to_string()),
            timeout_seconds: 60,
        };

        let backend = RemoteBackend::new(config);
        assert_eq!(backend.id(), "gpt-4");
        assert_eq!(backend.backend_type(), BackendType::Remote);
    }

    #[tokio::test]
    async fn test_generate_returns_error() {
        let backend = RemoteBackend::new(RemoteBackendConfig::default());

        let request = ModelRequest::new(ModelRole::General, "test", "Test prompt");
        let result = backend.generate(request).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::EngagementViolation(_)));
    }

    #[tokio::test]
    async fn test_health_check_returns_false() {
        let backend = RemoteBackend::new(RemoteBackendConfig::default());
        let health = backend.health_check().await.unwrap();
        assert!(!health);
    }

    #[test]
    fn test_default_config() {
        let config = RemoteBackendConfig::default();
        assert_eq!(config.model_id, "unknown");
        assert_eq!(config.api_base, "http://localhost:8000/v1");
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout_seconds, 30);
    }
}
