/// Core error types for Bodhya platform
///
/// This module defines the error hierarchy used throughout the Bodhya codebase.
/// We use thiserror for ergonomic error definition and anyhow for application-level
/// error handling with context.
use thiserror::Error;

/// Result type alias using Bodhya's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type for Bodhya operations
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Model-related errors
    #[error("Model error: {0}")]
    Model(String),

    /// Agent-related errors
    #[error("Agent error: {0}")]
    Agent(String),

    /// Agent not found or unavailable
    #[error("Agent '{0}' not found or disabled")]
    AgentNotFound(String),

    /// Model not found or not installed
    #[error("Model '{0}' not found or not installed")]
    ModelNotFound(String),

    /// Tool execution errors
    #[error("Tool error: {0}")]
    Tool(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Checksum verification failed
    #[error("Checksum mismatch: {0}")]
    ChecksumMismatch(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid input or parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    /// Engagement mode violation (e.g., remote call attempted in minimum mode)
    #[error("Engagement mode violation: {0}")]
    EngagementViolation(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement conversions for common error types
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::AgentNotFound("code".to_string());
        assert_eq!(err.to_string(), "Agent 'code' not found or disabled");

        let err = Error::ModelNotFound("planner".to_string());
        assert_eq!(
            err.to_string(),
            "Model 'planner' not found or not installed"
        );

        let err = Error::Config("missing field".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing field");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, 42);
        }
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(Error::InvalidInput("bad value".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_engagement_violation_error() {
        let err = Error::EngagementViolation(
            "Remote model call attempted in minimum engagement mode".to_string(),
        );
        assert!(err.to_string().contains("Engagement mode violation"));
    }
}
