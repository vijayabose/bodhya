/// Agent abstractions for Bodhya
///
/// This module defines the core Agent trait and related types for implementing
/// domain-specific agents (code, mail, etc.) with capability-based routing.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::Result;
use std::path::PathBuf;
use std::sync::Arc;

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

/// Execution limits to prevent resource exhaustion and infinite loops
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionLimits {
    /// Maximum number of iterations in agentic loop
    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
    /// Maximum number of file write operations
    #[serde(default = "default_max_file_writes")]
    pub max_file_writes: usize,
    /// Maximum number of command executions
    #[serde(default = "default_max_command_executions")]
    pub max_command_executions: usize,
    /// Global timeout in seconds
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_max_iterations() -> usize {
    3
}

fn default_max_file_writes() -> usize {
    20
}

fn default_max_command_executions() -> usize {
    10
}

fn default_timeout_secs() -> u64 {
    300 // 5 minutes
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_iterations: default_max_iterations(),
            max_file_writes: default_max_file_writes(),
            max_command_executions: default_max_command_executions(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

impl ExecutionLimits {
    /// Create new execution limits with custom values
    pub fn new(
        max_iterations: usize,
        max_file_writes: usize,
        max_command_executions: usize,
        timeout_secs: u64,
    ) -> Self {
        Self {
            max_iterations,
            max_file_writes,
            max_command_executions,
            timeout_secs,
        }
    }

    /// Create limits with no restrictions (use cautiously!)
    pub fn unlimited() -> Self {
        Self {
            max_iterations: usize::MAX,
            max_file_writes: usize::MAX,
            max_command_executions: usize::MAX,
            timeout_secs: u64::MAX,
        }
    }
}

/// Execution mode for code generation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    /// Generate code only (no file writes or command execution)
    GenerateOnly,
    /// Execute code (write files and run tests)
    #[default]
    Execute,
    /// Execute with retry (observe-retry-fix workflow - Phase 3)
    ExecuteWithRetry,
}

impl ExecutionMode {
    /// Parse execution mode from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "generate-only" | "generate_only" | "generate" => Some(Self::GenerateOnly),
            "execute" | "exec" => Some(Self::Execute),
            "execute-with-retry" | "execute_with_retry" | "retry" => Some(Self::ExecuteWithRetry),
            _ => None,
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GenerateOnly => "generate-only",
            Self::Execute => "execute",
            Self::ExecuteWithRetry => "execute-with-retry",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::GenerateOnly => "Generate code only (no file operations)",
            Self::Execute => "Generate code, write files, and run tests",
            Self::ExecuteWithRetry => "Execute with retry on failures (agentic loop)",
        }
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Context provided to agents when handling tasks
#[derive(Clone)]
pub struct AgentContext {
    /// Application configuration
    pub config: AppConfig,
    /// Task-specific metadata
    pub metadata: serde_json::Value,
    /// Working directory for file operations
    pub working_dir: Option<PathBuf>,
    /// Execution limits to prevent resource exhaustion
    pub execution_limits: ExecutionLimits,
    /// Execution mode (generate-only, execute, execute-with-retry)
    pub execution_mode: ExecutionMode,
    /// Tool registry (type-erased to avoid circular dependency)
    /// Agents can downcast this to ToolRegistry using std::any::Any
    pub tools: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl AgentContext {
    /// Create a new agent context
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            metadata: serde_json::Value::Null,
            working_dir: None,
            execution_limits: ExecutionLimits::default(),
            execution_mode: ExecutionMode::default(),
            tools: None,
        }
    }

    /// Add metadata to the context
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the working directory for file operations
    pub fn with_working_dir(mut self, working_dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(working_dir.into());
        self
    }

    /// Set execution limits
    pub fn with_execution_limits(mut self, limits: ExecutionLimits) -> Self {
        self.execution_limits = limits;
        self
    }

    /// Set execution mode
    pub fn with_execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Set the tool registry (type-erased)
    /// The registry should be Arc<ToolRegistry> which agents can downcast
    pub fn with_tools(mut self, tools: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Get the working directory, or current directory if not set
    pub fn get_working_dir(&self) -> Result<PathBuf> {
        self.working_dir
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| {
                crate::errors::Error::Config("No working directory available".to_string())
            })
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

    #[test]
    fn test_execution_limits_default() {
        let limits = ExecutionLimits::default();
        assert_eq!(limits.max_iterations, 3);
        assert_eq!(limits.max_file_writes, 20);
        assert_eq!(limits.max_command_executions, 10);
        assert_eq!(limits.timeout_secs, 300);
    }

    #[test]
    fn test_execution_limits_custom() {
        let limits = ExecutionLimits::new(5, 50, 20, 600);
        assert_eq!(limits.max_iterations, 5);
        assert_eq!(limits.max_file_writes, 50);
        assert_eq!(limits.max_command_executions, 20);
        assert_eq!(limits.timeout_secs, 600);
    }

    #[test]
    fn test_execution_limits_unlimited() {
        let limits = ExecutionLimits::unlimited();
        assert_eq!(limits.max_iterations, usize::MAX);
        assert_eq!(limits.max_file_writes, usize::MAX);
        assert_eq!(limits.max_command_executions, usize::MAX);
        assert_eq!(limits.timeout_secs, u64::MAX);
    }

    #[test]
    fn test_execution_limits_serialization() {
        let limits = ExecutionLimits::new(5, 50, 20, 600);
        let json = serde_json::to_string(&limits).unwrap();
        let deserialized: ExecutionLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_iterations, 5);
        assert_eq!(deserialized.max_file_writes, 50);
        assert_eq!(deserialized.max_command_executions, 20);
        assert_eq!(deserialized.timeout_secs, 600);
    }

    #[test]
    fn test_agent_context_with_working_dir() {
        let config = AppConfig::default();
        let ctx = AgentContext::new(config).with_working_dir("/tmp/test");
        assert_eq!(ctx.working_dir, Some(PathBuf::from("/tmp/test")));
    }

    #[test]
    fn test_agent_context_with_execution_limits() {
        let config = AppConfig::default();
        let limits = ExecutionLimits::new(5, 50, 20, 600);
        let ctx = AgentContext::new(config).with_execution_limits(limits.clone());
        assert_eq!(ctx.execution_limits.max_iterations, 5);
        assert_eq!(ctx.execution_limits.max_file_writes, 50);
    }

    #[test]
    fn test_agent_context_get_working_dir() {
        let config = AppConfig::default();

        // Test with explicit working dir
        let ctx = AgentContext::new(config.clone()).with_working_dir("/tmp/explicit");
        let working_dir = ctx.get_working_dir().unwrap();
        assert_eq!(working_dir, PathBuf::from("/tmp/explicit"));

        // Test fallback to current directory
        let ctx_no_dir = AgentContext::new(config);
        let working_dir_fallback = ctx_no_dir.get_working_dir();
        assert!(working_dir_fallback.is_ok());
    }

    #[test]
    fn test_agent_context_builder_chain() {
        let config = AppConfig::default();
        let metadata = serde_json::json!({"request_id": "req-123"});
        let limits = ExecutionLimits::new(5, 50, 20, 600);

        let ctx = AgentContext::new(config)
            .with_metadata(metadata.clone())
            .with_working_dir("/tmp/chain")
            .with_execution_limits(limits);

        assert_eq!(ctx.metadata, metadata);
        assert_eq!(ctx.working_dir, Some(PathBuf::from("/tmp/chain")));
        assert_eq!(ctx.execution_limits.max_iterations, 5);
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

    #[test]
    fn test_execution_mode_parse() {
        assert_eq!(
            ExecutionMode::parse("generate-only"),
            Some(ExecutionMode::GenerateOnly)
        );
        assert_eq!(
            ExecutionMode::parse("generate_only"),
            Some(ExecutionMode::GenerateOnly)
        );
        assert_eq!(
            ExecutionMode::parse("generate"),
            Some(ExecutionMode::GenerateOnly)
        );
        assert_eq!(
            ExecutionMode::parse("execute"),
            Some(ExecutionMode::Execute)
        );
        assert_eq!(ExecutionMode::parse("exec"), Some(ExecutionMode::Execute));
        assert_eq!(
            ExecutionMode::parse("execute-with-retry"),
            Some(ExecutionMode::ExecuteWithRetry)
        );
        assert_eq!(
            ExecutionMode::parse("execute_with_retry"),
            Some(ExecutionMode::ExecuteWithRetry)
        );
        assert_eq!(
            ExecutionMode::parse("retry"),
            Some(ExecutionMode::ExecuteWithRetry)
        );
        assert_eq!(ExecutionMode::parse("invalid"), None);
    }

    #[test]
    fn test_execution_mode_default() {
        assert_eq!(ExecutionMode::default(), ExecutionMode::Execute);
    }

    #[test]
    fn test_execution_mode_as_str() {
        assert_eq!(ExecutionMode::GenerateOnly.as_str(), "generate-only");
        assert_eq!(ExecutionMode::Execute.as_str(), "execute");
        assert_eq!(
            ExecutionMode::ExecuteWithRetry.as_str(),
            "execute-with-retry"
        );
    }

    #[test]
    fn test_execution_mode_description() {
        assert!(ExecutionMode::GenerateOnly
            .description()
            .to_lowercase()
            .contains("generate"));
        assert!(ExecutionMode::Execute
            .description()
            .to_lowercase()
            .contains("files"));
        assert!(ExecutionMode::ExecuteWithRetry
            .description()
            .to_lowercase()
            .contains("retry"));
    }

    #[test]
    fn test_agent_context_with_execution_mode() {
        let config = AppConfig::default();
        let ctx = AgentContext::new(config).with_execution_mode(ExecutionMode::GenerateOnly);
        assert_eq!(ctx.execution_mode, ExecutionMode::GenerateOnly);
    }
}
