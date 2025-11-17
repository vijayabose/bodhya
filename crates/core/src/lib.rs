pub use agent::{
    Agent, AgentCapability, AgentContext, AgentResult, ExecutionLimits, ExecutionMode, Task,
};
pub use config::{AgentConfig, AppConfig, LoggingConfig, ModelConfigs, PathsConfig};
/// Bodhya Core Library
///
/// This crate provides the foundational types, traits, and abstractions
/// for the Bodhya multi-agent AI platform.
///
/// # Modules
///
/// - `errors`: Error types and Result aliases
/// - `config`: Configuration structures for app, agents, and models
/// - `model`: Model backend traits and types
/// - `agent`: Agent trait and task handling types
/// - `tool`: Tool and MCP interface abstractions
// Re-export commonly used types at the crate root
pub use errors::{Error, Result};
pub use model::{
    BackendType, EngagementMode, ModelBackend, ModelRequest, ModelResponse, ModelRole,
};
pub use tool::{McpClient, McpServerConfig, Tool, ToolRequest, ToolResponse};

// Public modules
pub mod agent;
pub mod config;
pub mod errors;
pub mod model;
pub mod tool;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_exports() {
        // Test that commonly used types are accessible from crate root
        let _err: Error = Error::Internal("test".to_string());
        let _mode: EngagementMode = EngagementMode::Minimum;
        let _role: ModelRole = ModelRole::Planner;
        let _task: Task = Task::new("test");
        let _config: AppConfig = AppConfig::default();
    }
}
