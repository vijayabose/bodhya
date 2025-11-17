/// Task orchestration and execution
///
/// This module handles the end-to-end task execution pipeline:
/// task intake -> routing -> agent execution -> result collection -> logging
use bodhya_core::{AgentContext, AgentResult, AppConfig, Task};
use bodhya_tools_mcp::ToolRegistry;
use std::path::PathBuf;
use std::sync::Arc;

use crate::engagement::EngagementManager;
use crate::routing::AgentRouter;

/// Central orchestrator for task execution
pub struct TaskOrchestrator {
    /// Agent router
    router: AgentRouter,
    /// Engagement manager
    engagement: EngagementManager,
    /// Application configuration
    config: AppConfig,
    /// Tool registry for file/command operations
    tools: Arc<ToolRegistry>,
    /// Working directory for file operations
    working_dir: Option<PathBuf>,
}

impl TaskOrchestrator {
    /// Create a new orchestrator
    pub fn new(config: AppConfig) -> Self {
        let engagement = EngagementManager::new(config.engagement_mode.clone());
        let tools = Arc::new(ToolRegistry::with_defaults());

        Self {
            router: AgentRouter::new(),
            engagement,
            config,
            tools,
            working_dir: None,
        }
    }

    /// Create a new orchestrator with custom tools
    pub fn with_tools(config: AppConfig, tools: Arc<ToolRegistry>) -> Self {
        let engagement = EngagementManager::new(config.engagement_mode.clone());

        Self {
            router: AgentRouter::new(),
            engagement,
            config,
            tools,
            working_dir: None,
        }
    }

    /// Set the working directory for file operations
    pub fn set_working_dir(&mut self, working_dir: impl Into<PathBuf>) {
        self.working_dir = Some(working_dir.into());
    }

    /// Get a reference to the tool registry
    pub fn tools(&self) -> &Arc<ToolRegistry> {
        &self.tools
    }

    /// Get a mutable reference to the router (for registering agents)
    pub fn router_mut(&mut self) -> &mut AgentRouter {
        &mut self.router
    }

    /// Get a reference to the router
    pub fn router(&self) -> &AgentRouter {
        &self.router
    }

    /// Get a reference to the engagement manager
    pub fn engagement(&self) -> &EngagementManager {
        &self.engagement
    }

    /// Execute a task
    ///
    /// This is the main entry point for task execution:
    /// 1. Select appropriate agent via router
    /// 2. Create agent context
    /// 3. Execute task through agent
    /// 4. Log execution metrics
    /// 5. Return result
    pub async fn execute(&self, task: Task) -> bodhya_core::Result<AgentResult> {
        tracing::info!(
            task_id = %task.id,
            description = %task.description,
            domain_hint = ?task.domain_hint,
            "Starting task execution"
        );

        // Select agent
        let agent = self.router.select_agent(&task)?;

        tracing::debug!(
            task_id = %task.id,
            agent_id = agent.id(),
            agent_domain = agent.capability().domain,
            "Selected agent for task"
        );

        // Create agent context with tools and working directory
        let mut context = AgentContext::new(self.config.clone())
            .with_tools(Arc::clone(&self.tools) as Arc<dyn std::any::Any + Send + Sync>);

        // Set working directory if specified
        if let Some(ref wd) = self.working_dir {
            context = context.with_working_dir(wd.clone());
        }

        // Execute task through agent
        let start_time = std::time::Instant::now();
        let result = agent.handle(task.clone(), context).await;
        let duration = start_time.elapsed();

        match &result {
            Ok(agent_result) => {
                tracing::info!(
                    task_id = %task.id,
                    agent_id = agent.id(),
                    success = agent_result.success,
                    duration_ms = duration.as_millis(),
                    "Task execution completed"
                );
            }
            Err(err) => {
                tracing::error!(
                    task_id = %task.id,
                    agent_id = agent.id(),
                    error = %err,
                    duration_ms = duration.as_millis(),
                    "Task execution failed"
                );
            }
        }

        result
    }

    /// Execute multiple tasks concurrently
    pub async fn execute_batch(&self, tasks: Vec<Task>) -> Vec<bodhya_core::Result<AgentResult>> {
        tracing::info!(count = tasks.len(), "Executing batch of tasks");

        let mut handles = Vec::new();

        for task in tasks {
            let orchestrator = self.clone_for_concurrent();
            let handle = tokio::spawn(async move { orchestrator.execute(task).await });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(bodhya_core::Error::Internal(format!(
                    "Task execution panicked: {}",
                    e
                )))),
            }
        }

        results
    }

    /// Clone this orchestrator for concurrent execution
    /// (Only clones immutable parts, agents are Arc-wrapped)
    fn clone_for_concurrent(&self) -> Arc<Self> {
        Arc::new(Self {
            router: self.router.clone(),
            engagement: self.engagement.clone(),
            config: self.config.clone(),
            tools: Arc::clone(&self.tools),
            working_dir: self.working_dir.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use bodhya_core::{Agent, AgentCapability, EngagementMode};
    use std::sync::Arc;

    // Mock agent for testing
    struct MockAgent {
        id: &'static str,
        should_fail: bool,
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn id(&self) -> &'static str {
            self.id
        }

        fn capability(&self) -> AgentCapability {
            AgentCapability::new("test", vec!["test".to_string()], "Mock test agent")
                .with_keywords(vec!["test".to_string()])
        }

        async fn handle(&self, task: Task, _ctx: AgentContext) -> bodhya_core::Result<AgentResult> {
            if self.should_fail {
                Err(bodhya_core::Error::Agent("Mock failure".to_string()))
            } else {
                Ok(AgentResult::success(
                    task.id,
                    format!("Processed by {}", self.id),
                ))
            }
        }
    }

    fn create_test_config() -> AppConfig {
        AppConfig {
            engagement_mode: EngagementMode::Minimum,
            ..Default::default()
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config);

        assert_eq!(*orchestrator.engagement().mode(), EngagementMode::Minimum);
        assert_eq!(orchestrator.router().agents().len(), 0);
    }

    #[test]
    fn test_register_agent() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        let agent = Arc::new(MockAgent {
            id: "test",
            should_fail: false,
        });

        orchestrator.router_mut().register(agent);
        assert_eq!(orchestrator.router().agents().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_task_success() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        let agent = Arc::new(MockAgent {
            id: "test",
            should_fail: false,
        });
        orchestrator.router_mut().register(agent);

        let task = Task::new("Test task").with_domain("test");
        let result = orchestrator.execute(task).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("Processed by test"));
    }

    #[tokio::test]
    async fn test_execute_task_failure() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        let agent = Arc::new(MockAgent {
            id: "test",
            should_fail: true,
        });
        orchestrator.router_mut().register(agent);

        let task = Task::new("Test task").with_domain("test");
        let result = orchestrator.execute(task).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_task_no_agent() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config);

        let task = Task::new("Test task");
        let result = orchestrator.execute(task).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), bodhya_core::Error::Agent(_)));
    }

    #[tokio::test]
    async fn test_execute_batch() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        let agent = Arc::new(MockAgent {
            id: "test",
            should_fail: false,
        });
        orchestrator.router_mut().register(agent);

        let tasks = vec![
            Task::new("Task 1").with_domain("test"),
            Task::new("Task 2").with_domain("test"),
            Task::new("Task 3").with_domain("test"),
        ];

        let results = orchestrator.execute_batch(tasks).await;

        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_ok());
            let agent_result = result.unwrap();
            assert!(agent_result.success);
        }
    }

    #[tokio::test]
    async fn test_execute_batch_with_failures() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        // Agent that fails
        let agent = Arc::new(MockAgent {
            id: "test",
            should_fail: true,
        });
        orchestrator.router_mut().register(agent);

        let tasks = vec![
            Task::new("Task 1").with_domain("test"),
            Task::new("Task 2").with_domain("test"),
        ];

        let results = orchestrator.execute_batch(tasks).await;

        assert_eq!(results.len(), 2);
        for result in results {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_orchestrator_accessors() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config);

        assert_eq!(orchestrator.router().agents().len(), 0);
        assert_eq!(*orchestrator.engagement().mode(), EngagementMode::Minimum);
    }
}
