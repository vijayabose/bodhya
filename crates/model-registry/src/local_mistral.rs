/// Local model backend using mistral.rs
///
/// This module provides a local inference backend that will eventually integrate
/// with mistral.rs for running GGUF models locally. For now, it's a stub that
/// returns mock responses for testing and development.
use async_trait::async_trait;
use bodhya_core::{BackendType, ModelBackend, ModelRequest, ModelResponse, Result};
use std::path::PathBuf;

/// Configuration for local mistral.rs backend
#[derive(Clone, Debug)]
pub struct LocalBackendConfig {
    /// Path to the model file
    pub model_path: PathBuf,
    /// Model ID for identification
    pub model_id: String,
    /// Device to run on (auto, cpu, cuda, metal)
    pub device: String,
    /// Number of threads to use
    pub num_threads: usize,
    /// Context size in tokens
    pub context_size: usize,
}

impl Default for LocalBackendConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            model_id: String::from("unknown"),
            device: String::from("auto"),
            num_threads: 4,
            context_size: 4096,
        }
    }
}

/// Stub implementation of local model backend
///
/// This is a placeholder that returns mock responses. In a future phase,
/// this will integrate with mistral.rs for actual local inference.
pub struct LocalMistralBackend {
    config: LocalBackendConfig,
}

impl LocalMistralBackend {
    /// Create a new local backend with the given configuration
    pub fn new(config: LocalBackendConfig) -> Self {
        Self { config }
    }

    /// Create a backend for a specific model file
    pub fn from_model_path(model_path: PathBuf, model_id: impl Into<String>) -> Self {
        Self {
            config: LocalBackendConfig {
                model_path,
                model_id: model_id.into(),
                ..Default::default()
            },
        }
    }
}

#[async_trait]
impl ModelBackend for LocalMistralBackend {
    fn id(&self) -> &str {
        &self.config.model_id
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Local
    }

    async fn generate(&self, request: ModelRequest) -> Result<ModelResponse> {
        // Stub implementation - returns a mock response
        // TODO: Integrate with mistral.rs for actual inference

        let response_text = format!(
            "[STUB] Local model response for role={:?} domain={}\nPrompt: {}\n\n\
             This is a placeholder response from the local backend stub. \
             In production, this would use mistral.rs to generate actual responses.",
            request.role, request.domain, request.prompt
        );

        let metadata = serde_json::json!({
            "backend": "local_stub",
            "model_id": self.config.model_id,
            "model_path": self.config.model_path.display().to_string(),
            "device": self.config.device,
            "stub": true,
        });

        Ok(ModelResponse::new(response_text).with_metadata(metadata))
    }

    async fn health_check(&self) -> Result<bool> {
        // Stub implementation - always returns true for testing
        // In a real implementation, this would check if the model file exists and can be loaded
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bodhya_core::ModelRole;

    #[test]
    fn test_local_backend_creation() {
        let config = LocalBackendConfig {
            model_path: PathBuf::from("/models/test.gguf"),
            model_id: "test_model".to_string(),
            device: "cpu".to_string(),
            num_threads: 8,
            context_size: 8192,
        };

        let backend = LocalMistralBackend::new(config.clone());
        assert_eq!(backend.id(), "test_model");
        assert_eq!(backend.backend_type(), BackendType::Local);
        assert_eq!(backend.config.device, "cpu");
        assert_eq!(backend.config.num_threads, 8);
    }

    #[test]
    fn test_from_model_path() {
        let backend = LocalMistralBackend::from_model_path(
            PathBuf::from("/models/planner.gguf"),
            "planner_model",
        );

        assert_eq!(backend.id(), "planner_model");
        assert!(backend
            .config
            .model_path
            .to_str()
            .unwrap()
            .contains("planner.gguf"));
    }

    #[tokio::test]
    async fn test_generate_stub_response() {
        let backend =
            LocalMistralBackend::from_model_path(PathBuf::from("/models/test.gguf"), "test_model");

        let request = ModelRequest::new(
            ModelRole::Planner,
            "code",
            "Generate a plan for a config loader",
        );

        let response = backend.generate(request).await.unwrap();

        assert!(response.text.contains("[STUB]"));
        assert!(response.text.contains("Planner"));
        assert!(response.text.contains("code"));
        assert!(response.text.contains("placeholder"));

        // Check metadata
        assert_eq!(response.metadata["backend"], "local_stub");
        assert_eq!(response.metadata["model_id"], "test_model");
        assert_eq!(response.metadata["stub"], true);
    }

    #[tokio::test]
    async fn test_health_check() {
        let backend =
            LocalMistralBackend::from_model_path(PathBuf::from("/nonexistent/model.gguf"), "test");

        // Stub implementation returns Ok even for nonexistent paths
        let health = backend.health_check().await.unwrap();
        assert!(health);
    }

    #[test]
    fn test_default_config() {
        let config = LocalBackendConfig::default();
        assert_eq!(config.device, "auto");
        assert_eq!(config.num_threads, 4);
        assert_eq!(config.context_size, 4096);
    }

    #[tokio::test]
    async fn test_different_roles_generate_different_responses() {
        let backend =
            LocalMistralBackend::from_model_path(PathBuf::from("/models/test.gguf"), "test");

        let planner_req = ModelRequest::new(ModelRole::Planner, "code", "Test prompt");
        let coder_req = ModelRequest::new(ModelRole::Coder, "code", "Test prompt");

        let planner_resp = backend.generate(planner_req).await.unwrap();
        let coder_resp = backend.generate(coder_req).await.unwrap();

        assert!(planner_resp.text.contains("Planner"));
        assert!(coder_resp.text.contains("Coder"));
    }
}
