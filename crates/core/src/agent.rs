/// Agent abstractions for Bodhya
///
/// This module defines the core Agent trait and related types for implementing
/// domain-specific agents (code, mail, etc.) with capability-based routing.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::Result;

/// Represents a task to be handled by an agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: String,
    /// Optional domain hint for routing (e.g., "code", "mail")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_hint: Option<String>,
    /// Human-readable task description
    pub description: String,
    /// Structured task payload
    #[serde(default)]
    pub payload: serde_json::Value,
    /// Task creation timestamp
    #[serde(default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Task {
    /// Create a new task with auto-generated ID
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            domain_hint: None,
            description: description.into(),
            payload: serde_json::Value::Null,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a task with a specific domain hint
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain_hint = Some(domain.into());
        self
    }

    /// Add structured payload to the task
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }
}

/// Result returned by an agent after handling a task
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentResult {
    /// The task ID this result corresponds to
    pub task_id: String,
    /// Main content/output from the agent
    pub content: String,
    /// Optional structured metadata (metrics, logs, etc.)
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Whether the task completed successfully
    pub success: bool,
    /// Optional error message if task failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AgentResult {
    /// Create a successful result
    pub fn success(task_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            content: content.into(),
            metadata: serde_json::Value::Null,
            success: true,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(task_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            content: String::new(),
            metadata: serde_json::Value::Null,
            success: false,
            error: Some(error.into()),
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Describes an agent's capabilities for intelligent routing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Primary domain (e.g., "code", "mail", "summarization")
    pub domain: String,
    /// Supported intents/actions (e.g., ["generate", "refine", "test"])
    pub intents: Vec<String>,
    /// Human-readable description
    pub description: String,
    /// Keywords for matching task descriptions
    #[serde(default)]
    pub keywords: Vec<String>,
}

impl AgentCapability {
    /// Create a new capability descriptor
    pub fn new(
        domain: impl Into<String>,
        intents: Vec<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            intents,
            description: description.into(),
            keywords: Vec::new(),
        }
    }

    /// Add keywords for better routing
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Check if this capability matches a task description
    pub fn matches(&self, description: &str) -> bool {
        let desc_lower = description.to_lowercase();

        // Check if any keyword appears in the description
        self.keywords
            .iter()
            .any(|kw| desc_lower.contains(&kw.to_lowercase()))
    }
}

/// Context provided to agents when handling tasks
#[derive(Clone)]
pub struct AgentContext {
    /// Application configuration
    pub config: AppConfig,
    /// Task-specific metadata
    pub metadata: serde_json::Value,
}

impl AgentContext {
    /// Create a new agent context
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            metadata: serde_json::Value::Null,
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Core trait for domain agents
///
/// All agents (CodeAgent, MailAgent, etc.) must implement this trait to
/// participate in the Bodhya platform.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Unique agent identifier (e.g., "code", "mail")
    fn id(&self) -> &'static str;

    /// Agent's capabilities for routing
    fn capability(&self) -> AgentCapability;

    /// Handle a task and return a result
    async fn handle(&self, task: Task, ctx: AgentContext) -> Result<AgentResult>;

    /// Optional: Check if agent is enabled/available
    fn is_enabled(&self) -> bool {
        true // Default: always enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test task");
        assert!(!task.id.is_empty());
        assert_eq!(task.description, "Test task");
        assert!(task.domain_hint.is_none());
        assert_eq!(task.payload, serde_json::Value::Null);
    }

    #[test]
    fn test_task_with_domain() {
        let task = Task::new("Write code").with_domain("code");
        assert_eq!(task.domain_hint, Some("code".to_string()));
    }

    #[test]
    fn test_task_with_payload() {
        let payload = serde_json::json!({"key": "value"});
        let task = Task::new("Test").with_payload(payload.clone());
        assert_eq!(task.payload, payload);
    }

    #[test]
    fn test_agent_result_success() {
        let result = AgentResult::success("task-123", "Generated code");
        assert!(result.success);
        assert_eq!(result.task_id, "task-123");
        assert_eq!(result.content, "Generated code");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_agent_result_failure() {
        let result = AgentResult::failure("task-456", "Model not found");
        assert!(!result.success);
        assert_eq!(result.task_id, "task-456");
        assert_eq!(result.error, Some("Model not found".to_string()));
    }

    #[test]
    fn test_agent_result_with_metadata() {
        let metadata = serde_json::json!({"tokens": 500, "latency_ms": 1200});
        let result = AgentResult::success("task-789", "Done").with_metadata(metadata.clone());
        assert_eq!(result.metadata, metadata);
    }

    #[test]
    fn test_agent_capability_creation() {
        let cap = AgentCapability::new(
            "code",
            vec!["generate".to_string(), "refine".to_string()],
            "Code generation agent",
        );
        assert_eq!(cap.domain, "code");
        assert_eq!(cap.intents.len(), 2);
        assert_eq!(cap.description, "Code generation agent");
    }

    #[test]
    fn test_capability_matches() {
        let cap = AgentCapability::new("code", vec!["generate".to_string()], "Code agent")
            .with_keywords(vec![
                "code".to_string(),
                "rust".to_string(),
                "function".to_string(),
            ]);

        assert!(cap.matches("Generate Rust code"));
        assert!(cap.matches("Write a function"));
        assert!(cap.matches("Create code for parsing"));
        assert!(!cap.matches("Write an email"));
    }

    #[test]
    fn test_agent_context_creation() {
        let config = AppConfig::default();
        let ctx = AgentContext::new(config);
        assert_eq!(ctx.metadata, serde_json::Value::Null);
    }

    #[test]
    fn test_agent_context_with_metadata() {
        let config = AppConfig::default();
        let metadata = serde_json::json!({"request_id": "req-123"});
        let ctx = AgentContext::new(config).with_metadata(metadata.clone());
        assert_eq!(ctx.metadata, metadata);
    }

    // Mock agent for testing
    struct MockAgent {
        id: &'static str,
        enabled: bool,
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn id(&self) -> &'static str {
            self.id
        }

        fn capability(&self) -> AgentCapability {
            AgentCapability::new("test", vec!["mock".to_string()], "Mock agent for testing")
        }

        async fn handle(&self, task: Task, _ctx: AgentContext) -> Result<AgentResult> {
            Ok(AgentResult::success(task.id, "Mock result"))
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    #[tokio::test]
    async fn test_agent_trait_implementation() {
        let agent = MockAgent {
            id: "mock",
            enabled: true,
        };

        assert_eq!(agent.id(), "mock");
        assert!(agent.is_enabled());

        let task = Task::new("Test task");
        let config = AppConfig::default();
        let ctx = AgentContext::new(config);

        let result = agent.handle(task, ctx).await.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "Mock result");
    }

    #[test]
    fn test_task_serialization() {
        let task = Task::new("Test").with_domain("code");
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.description, task.description);
        assert_eq!(deserialized.domain_hint, task.domain_hint);
    }
}
