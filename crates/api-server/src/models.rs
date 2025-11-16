/// API request and response models
use bodhya_core::AgentCapability;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request to submit a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskRequest {
    /// Optional domain hint for routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// Task description
    pub description: String,

    /// Optional structured payload
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Response when task is submitted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTaskResponse {
    /// Assigned task ID
    pub task_id: String,

    /// Task status
    pub status: TaskStatus,

    /// When the task was created
    pub created_at: DateTime<Utc>,
}

/// Task status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is queued/pending
    Pending,

    /// Task is currently being processed
    InProgress,

    /// Task completed successfully
    Completed,

    /// Task failed with error
    Failed,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Failed)
    }
}

/// Task state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Task ID
    pub task_id: String,

    /// Current status
    pub status: TaskStatus,

    /// Optional domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// Task description
    pub description: String,

    /// When task was created
    pub created_at: DateTime<Utc>,

    /// When task was started (if in progress or completed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,

    /// When task completed (if finished)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Progress percentage (0-100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: String,

    /// Whether task succeeded
    pub success: bool,

    /// Result content (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Metadata
    #[serde(default)]
    pub metadata: serde_json::Value,

    /// When task completed
    pub completed_at: DateTime<Utc>,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID
    pub id: String,

    /// Agent domain
    pub domain: String,

    /// Supported intents
    pub intents: Vec<String>,

    /// Description
    pub description: String,

    /// Whether agent is enabled
    pub enabled: bool,
}

impl From<&AgentCapability> for AgentInfo {
    fn from(cap: &AgentCapability) -> Self {
        Self {
            id: cap.domain.clone(), // Use domain as ID for now
            domain: cap.domain.clone(),
            intents: cap.intents.clone(),
            description: cap.description.clone(),
            enabled: true, // Assume enabled if capability exists
        }
    }
}

/// List of available agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentList {
    pub agents: Vec<AgentInfo>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: Some(details.into()),
        }
    }
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// Task status update
    TaskStatus {
        task_id: String,
        status: TaskStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        progress: Option<u8>,
    },

    /// Task output chunk (streaming)
    TaskOutput {
        task_id: String,
        content: String,
    },

    /// Task completed
    TaskComplete {
        task_id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },

    /// Error occurred
    Error {
        message: String,
    },

    /// Ping/pong for keep-alive
    Ping,
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_terminal() {
        assert!(!TaskStatus::Pending.is_terminal());
        assert!(!TaskStatus::InProgress.is_terminal());
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
    }

    #[test]
    fn test_submit_task_request_deserialize() {
        let json = r#"{"description":"test task","domain":"code"}"#;
        let req: SubmitTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.description, "test task");
        assert_eq!(req.domain, Some("code".to_string()));
    }

    #[test]
    fn test_error_response() {
        let err = ErrorResponse::new("something failed");
        assert_eq!(err.error, "something failed");
        assert!(err.details.is_none());

        let err2 = ErrorResponse::with_details("failed", "more info");
        assert_eq!(err2.error, "failed");
        assert_eq!(err2.details, Some("more info".to_string()));
    }

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::TaskStatus {
            task_id: "123".to_string(),
            status: TaskStatus::InProgress,
            progress: Some(50),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("task_status"));
        assert!(json.contains("in_progress"));
    }
}
