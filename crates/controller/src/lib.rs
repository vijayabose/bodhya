/// Bodhya Controller
///
/// This crate provides the central controller agent for task routing,
/// engagement mode management, and orchestration.
pub use controller::Controller;
pub use engagement::{EngagementManager, EngagementOperation, EngagementStrategy};
pub use orchestrator::TaskOrchestrator;
pub use routing::AgentRouter;

pub mod controller;
pub mod engagement;
pub mod orchestrator;
pub mod routing;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use async_trait::async_trait;
    use bodhya_core::{
        Agent, AgentCapability, AgentContext, AgentResult, AppConfig, EngagementMode, Task,
    };
    use std::sync::Arc;

    // Mock code agent for integration testing
    struct MockCodeAgent;

    #[async_trait]
    impl Agent for MockCodeAgent {
        fn id(&self) -> &'static str {
            "code"
        }

        fn capability(&self) -> AgentCapability {
            AgentCapability::new(
                "code",
                vec!["generate".to_string(), "refine".to_string()],
                "Code generation and refinement agent",
            )
            .with_keywords(vec![
                "code".to_string(),
                "rust".to_string(),
                "function".to_string(),
                "generate".to_string(),
            ])
        }

        async fn handle(&self, task: Task, _ctx: AgentContext) -> bodhya_core::Result<AgentResult> {
            Ok(AgentResult::success(
                task.id,
                "fn hello() { println!(\"Hello, world!\"); }",
            ))
        }
    }

    // Mock mail agent for integration testing
    struct MockMailAgent;

    #[async_trait]
    impl Agent for MockMailAgent {
        fn id(&self) -> &'static str {
            "mail"
        }

        fn capability(&self) -> AgentCapability {
            AgentCapability::new(
                "mail",
                vec!["draft".to_string(), "refine".to_string()],
                "Email drafting and refinement agent",
            )
            .with_keywords(vec![
                "email".to_string(),
                "mail".to_string(),
                "write".to_string(),
                "draft".to_string(),
            ])
        }

        async fn handle(&self, task: Task, _ctx: AgentContext) -> bodhya_core::Result<AgentResult> {
            Ok(AgentResult::success(
                task.id,
                "Dear Team,\n\nThis is a test email.\n\nBest regards",
            ))
        }
    }

    fn create_test_config() -> AppConfig {
        AppConfig {
            engagement_mode: EngagementMode::Minimum,
            ..Default::default()
        }
    }

    /// Integration test: Full workflow from task to result
    #[tokio::test]
    async fn test_full_task_execution_workflow() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        // Register agents
        orchestrator.router_mut().register(Arc::new(MockCodeAgent));
        orchestrator.router_mut().register(Arc::new(MockMailAgent));

        // Execute code task
        let code_task = Task::new("Generate a Rust function");
        let result = orchestrator.execute(code_task).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("fn hello"));

        // Execute mail task
        let mail_task = Task::new("Draft an email to the team");
        let result = orchestrator.execute(mail_task).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("Dear Team"));
    }

    /// Integration test: Domain hint routing
    #[tokio::test]
    async fn test_domain_hint_routing() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        orchestrator.router_mut().register(Arc::new(MockCodeAgent));
        orchestrator.router_mut().register(Arc::new(MockMailAgent));

        // Explicit domain hint should override keyword matching
        let task = Task::new("Do something").with_domain("code");
        let result = orchestrator.execute(task).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("fn hello")); // Routed to code agent
    }

    /// Integration test: Keyword-based routing
    #[tokio::test]
    async fn test_keyword_based_routing() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        orchestrator.router_mut().register(Arc::new(MockCodeAgent));
        orchestrator.router_mut().register(Arc::new(MockMailAgent));

        // Task with email keywords should route to mail agent
        let task = Task::new("Write an email about the project status");
        let result = orchestrator.execute(task).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("Dear Team")); // Routed to mail agent

        // Task with code keywords should route to code agent
        let task = Task::new("Generate Rust code for parsing");
        let result = orchestrator.execute(task).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("fn hello")); // Routed to code agent
    }

    /// Integration test: Engagement mode enforcement
    #[test]
    fn test_engagement_mode_enforcement() {
        let config = AppConfig {
            engagement_mode: EngagementMode::Minimum,
            ..Default::default()
        };

        let orchestrator = TaskOrchestrator::new(config);
        let engagement = orchestrator.engagement();

        // Minimum mode should not allow remote
        assert!(!engagement.is_remote_allowed());
        assert_eq!(*engagement.mode(), EngagementMode::Minimum);

        // Strategy should prefer local
        let strategy = engagement.get_strategy();
        assert!(strategy.prefer_local);
        assert!(!strategy.allow_remote_fallback);
    }

    /// Integration test: Batch execution
    #[tokio::test]
    async fn test_batch_execution() {
        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        orchestrator.router_mut().register(Arc::new(MockCodeAgent));
        orchestrator.router_mut().register(Arc::new(MockMailAgent));

        let tasks = vec![
            Task::new("Generate code").with_domain("code"),
            Task::new("Draft email").with_domain("mail"),
            Task::new("More code").with_domain("code"),
        ];

        let results = orchestrator.execute_batch(tasks).await;

        assert_eq!(results.len(), 3);
        for result in results {
            assert!(result.is_ok());
            assert!(result.unwrap().success);
        }
    }

    /// Integration test: No agent found
    #[tokio::test]
    async fn test_no_agent_found() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config);
        // No agents registered

        let task = Task::new("Test task");
        let result = orchestrator.execute(task).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), bodhya_core::Error::Agent(_)));
    }

    /// Integration test: Agent can access tools from context
    #[tokio::test]
    async fn test_agent_can_access_tools() {
        use bodhya_tools_mcp::ToolRegistry;

        // Mock agent that accesses tools from context
        struct ToolAwareAgent;

        #[async_trait]
        impl Agent for ToolAwareAgent {
            fn id(&self) -> &'static str {
                "tool-test"
            }

            fn capability(&self) -> AgentCapability {
                AgentCapability::new("test", vec!["test".to_string()], "Tool-aware test agent")
                    .with_keywords(vec!["tool".to_string(), "test".to_string()])
            }

            async fn handle(
                &self,
                task: Task,
                ctx: AgentContext,
            ) -> bodhya_core::Result<AgentResult> {
                // Verify tools are available in context
                let tools = ctx.tools.expect("Tools should be available in context");

                // Downcast to ToolRegistry
                let _registry = tools
                    .downcast_ref::<ToolRegistry>()
                    .expect("Should be able to downcast to ToolRegistry");

                Ok(AgentResult::success(
                    task.id,
                    "Successfully accessed tools from context",
                ))
            }
        }

        let config = create_test_config();
        let mut orchestrator = TaskOrchestrator::new(config);

        // Register tool-aware agent
        orchestrator.router_mut().register(Arc::new(ToolAwareAgent));

        // Execute task that requires tools
        let task = Task::new("Test tool access").with_domain("test");
        let result = orchestrator.execute(task).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("Successfully accessed tools"));
    }

    /// Integration test: Verify ToolRegistry is properly initialized
    #[test]
    fn test_orchestrator_has_tools() {
        let config = create_test_config();
        let orchestrator = TaskOrchestrator::new(config);

        let tools = orchestrator.tools();

        // Verify it has default tools registered
        assert!(tools.get_tool("filesystem").is_some());
        assert!(tools.get_tool("shell").is_some());

        // Verify tools list
        let tool_list = tools.list_tools();
        assert!(tool_list.contains(&"filesystem".to_string()));
        assert!(tool_list.contains(&"shell".to_string()));
    }
}
