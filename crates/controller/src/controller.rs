/// Simple controller wrapper for easy API server integration
use async_trait::async_trait;
use bodhya_core::{Agent, AgentResult, AppConfig, Task};
use std::sync::Arc;

use crate::orchestrator::TaskOrchestrator;

/// Simple controller that combines routing and orchestration
pub struct Controller {
    orchestrator: TaskOrchestrator,
}

impl Controller {
    /// Create a new controller with given agents
    pub fn new(agents: Vec<Arc<dyn Agent>>) -> Self {
        let config = AppConfig::default();
        let mut orchestrator = TaskOrchestrator::new(config);

        // Register all agents
        for agent in agents {
            orchestrator.router_mut().register(agent);
        }

        Self { orchestrator }
    }

    /// Execute a task
    pub async fn execute(&self, task: Task) -> bodhya_core::Result<AgentResult> {
        self.orchestrator.execute(task).await
    }

    /// List all registered agents
    pub fn list_agents(&self) -> Vec<Box<dyn Agent>> {
        self.orchestrator
            .router()
            .agents()
            .iter()
            .map(|agent| {
                // Clone the Arc reference and convert to Box<dyn Agent>
                let agent_clone: Box<dyn Agent> = Box::new(AgentClone {
                    agent: Arc::clone(agent),
                });
                agent_clone
            })
            .collect()
    }
}

/// Helper struct to clone agents for listing
struct AgentClone {
    agent: Arc<dyn Agent>,
}

#[async_trait]
impl Agent for AgentClone {
    fn id(&self) -> &'static str {
        self.agent.id()
    }

    fn capability(&self) -> bodhya_core::AgentCapability {
        self.agent.capability()
    }

    async fn handle(
        &self,
        task: Task,
        ctx: bodhya_core::AgentContext,
    ) -> bodhya_core::Result<AgentResult> {
        self.agent.handle(task, ctx).await
    }

    fn is_enabled(&self) -> bool {
        self.agent.is_enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use bodhya_core::{AgentCapability, AgentContext};

    struct TestAgent {
        id: &'static str,
        domain: String,
    }

    #[async_trait]
    impl Agent for TestAgent {
        fn id(&self) -> &'static str {
            self.id
        }

        fn capability(&self) -> AgentCapability {
            AgentCapability::new(
                self.domain.clone(),
                vec!["test".to_string()],
                "Test agent".to_string(),
            )
        }

        async fn handle(&self, task: Task, _ctx: AgentContext) -> bodhya_core::Result<AgentResult> {
            Ok(AgentResult::success(task.id, "test result"))
        }
    }

    #[tokio::test]
    async fn test_controller_creation() {
        let agent = Arc::new(TestAgent {
            id: "test",
            domain: "test".to_string(),
        }) as Arc<dyn Agent>;

        let controller = Controller::new(vec![agent]);
        assert_eq!(controller.list_agents().len(), 1);
    }

    #[tokio::test]
    async fn test_controller_execute() {
        let agent = Arc::new(TestAgent {
            id: "test",
            domain: "test".to_string(),
        }) as Arc<dyn Agent>;

        let controller = Controller::new(vec![agent]);

        let task = Task::new("test task").with_domain("test");
        let result = controller.execute(task).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "test result");
    }
}
