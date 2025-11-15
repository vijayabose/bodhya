/// Model backend abstractions for Bodhya
///
/// This module defines the core traits and types for model inference,
/// supporting both local and remote model backends with pluggable engagement modes.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

/// Engagement mode determines how aggressively to use remote models
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngagementMode {
    /// Local-only inference (v1 default)
    #[default]
    Minimum,
    /// Local primary, remote for difficult tasks (future)
    Medium,
    /// Remote heavily used (future)
    Maximum,
}

impl std::str::FromStr for EngagementMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "minimum" | "min" => Ok(EngagementMode::Minimum),
            "medium" | "med" => Ok(EngagementMode::Medium),
            "maximum" | "max" => Ok(EngagementMode::Maximum),
            _ => Err(Error::InvalidInput(format!(
                "Invalid engagement mode: {}. Valid values: minimum, medium, maximum",
                s
            ))),
        }
    }
}

/// Role that a model plays in the agent workflow
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelRole {
    /// Planning and task decomposition
    Planner,
    /// Code generation
    Coder,
    /// Code review and critique
    Reviewer,
    /// Email and communication writing
    Writer,
    /// Text summarization
    Summarizer,
    /// General-purpose reasoning
    General,
}

impl std::fmt::Display for ModelRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelRole::Planner => write!(f, "planner"),
            ModelRole::Coder => write!(f, "coder"),
            ModelRole::Reviewer => write!(f, "reviewer"),
            ModelRole::Writer => write!(f, "writer"),
            ModelRole::Summarizer => write!(f, "summarizer"),
            ModelRole::General => write!(f, "general"),
        }
    }
}

impl std::str::FromStr for ModelRole {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "planner" => Ok(ModelRole::Planner),
            "coder" => Ok(ModelRole::Coder),
            "reviewer" => Ok(ModelRole::Reviewer),
            "writer" => Ok(ModelRole::Writer),
            "summarizer" => Ok(ModelRole::Summarizer),
            "general" => Ok(ModelRole::General),
            _ => Err(Error::InvalidInput(format!("Invalid model role: {}", s))),
        }
    }
}

/// Request to a model backend for inference
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelRequest {
    /// Role of the model being invoked
    pub role: ModelRole,
    /// Domain context (e.g., "code", "mail")
    pub domain: String,
    /// The prompt/input text
    pub prompt: String,
    /// Optional temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional max tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

impl ModelRequest {
    /// Create a new model request
    pub fn new(role: ModelRole, domain: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            role,
            domain: domain.into(),
            prompt: prompt.into(),
            temperature: None,
            max_tokens: None,
        }
    }

    /// Set temperature for this request
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max tokens for this request
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// Response from a model backend
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelResponse {
    /// Generated text output
    pub text: String,
    /// Optional metadata (tokens used, timing, etc.)
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ModelResponse {
    /// Create a new model response
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a response with metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Backend type identifier
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendType {
    /// Local inference via mistral.rs
    Local,
    /// Remote API (future)
    Remote,
}

/// Trait for model inference backends
///
/// This trait abstracts over different model backends (local via mistral.rs,
/// remote APIs, etc.) providing a uniform interface for agents.
#[async_trait]
pub trait ModelBackend: Send + Sync {
    /// Unique identifier for this backend
    fn id(&self) -> &str;

    /// Backend type (local or remote)
    fn backend_type(&self) -> BackendType;

    /// Generate a response for the given request
    async fn generate(&self, request: ModelRequest) -> Result<ModelResponse>;

    /// Check if this backend is available and healthy
    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Default implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_mode_default() {
        assert_eq!(EngagementMode::default(), EngagementMode::Minimum);
    }

    #[test]
    fn test_engagement_mode_from_str() {
        assert_eq!(
            "minimum".parse::<EngagementMode>().unwrap(),
            EngagementMode::Minimum
        );
        assert_eq!(
            "min".parse::<EngagementMode>().unwrap(),
            EngagementMode::Minimum
        );
        assert_eq!(
            "medium".parse::<EngagementMode>().unwrap(),
            EngagementMode::Medium
        );
        assert_eq!(
            "maximum".parse::<EngagementMode>().unwrap(),
            EngagementMode::Maximum
        );

        assert!("invalid".parse::<EngagementMode>().is_err());
    }

    #[test]
    fn test_model_role_display() {
        assert_eq!(ModelRole::Planner.to_string(), "planner");
        assert_eq!(ModelRole::Coder.to_string(), "coder");
        assert_eq!(ModelRole::Writer.to_string(), "writer");
    }

    #[test]
    fn test_model_role_from_str() {
        assert_eq!("planner".parse::<ModelRole>().unwrap(), ModelRole::Planner);
        assert_eq!("coder".parse::<ModelRole>().unwrap(), ModelRole::Coder);
        assert_eq!(
            "reviewer".parse::<ModelRole>().unwrap(),
            ModelRole::Reviewer
        );

        assert!("invalid".parse::<ModelRole>().is_err());
    }

    #[test]
    fn test_model_request_builder() {
        let req = ModelRequest::new(ModelRole::Planner, "code", "test prompt")
            .with_temperature(0.7)
            .with_max_tokens(1000);

        assert_eq!(req.role, ModelRole::Planner);
        assert_eq!(req.domain, "code");
        assert_eq!(req.prompt, "test prompt");
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.max_tokens, Some(1000));
    }

    #[test]
    fn test_model_response_creation() {
        let resp = ModelResponse::new("generated text");
        assert_eq!(resp.text, "generated text");
        assert_eq!(resp.metadata, serde_json::Value::Null);

        let resp = resp.with_metadata(serde_json::json!({"tokens": 100}));
        assert_eq!(resp.metadata["tokens"], 100);
    }

    #[test]
    fn test_engagement_mode_serialization() {
        let mode = EngagementMode::Minimum;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, r#""minimum""#);

        let deserialized: EngagementMode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, mode);
    }

    #[test]
    fn test_model_role_serialization() {
        let role = ModelRole::Coder;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, r#""coder""#);

        let deserialized: ModelRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, role);
    }

    // Mock backend for testing
    struct MockBackend;

    #[async_trait]
    impl ModelBackend for MockBackend {
        fn id(&self) -> &str {
            "mock-backend"
        }

        fn backend_type(&self) -> BackendType {
            BackendType::Local
        }

        async fn generate(&self, request: ModelRequest) -> Result<ModelResponse> {
            Ok(ModelResponse::new(format!(
                "Mock response for: {}",
                request.prompt
            )))
        }
    }

    #[tokio::test]
    async fn test_model_backend_trait() {
        let backend = MockBackend;
        assert_eq!(backend.id(), "mock-backend");
        assert_eq!(backend.backend_type(), BackendType::Local);

        let req = ModelRequest::new(ModelRole::Planner, "test", "hello");
        let resp = backend.generate(req).await.unwrap();
        assert!(resp.text.contains("Mock response"));

        let health = backend.health_check().await.unwrap();
        assert!(health);
    }
}
