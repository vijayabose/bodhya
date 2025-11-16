/// Agent routing based on capabilities
///
/// This module implements intelligent routing that matches tasks to agents
/// based on their capability metadata (domain, intents, keywords).
use bodhya_core::{Agent, AgentCapability, Error, Result, Task};
use std::sync::Arc;

/// Router for selecting agents based on task requirements
pub struct AgentRouter {
    /// Registered agents
    agents: Vec<Arc<dyn Agent>>,
}

impl AgentRouter {
    /// Create a new router
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    /// Register an agent
    pub fn register(&mut self, agent: Arc<dyn Agent>) {
        self.agents.push(agent);
    }

    /// Select the best agent for a task
    ///
    /// Selection logic:
    /// 1. If task has domain_hint, filter by exact domain match
    /// 2. Score remaining agents by keyword matches in task description
    /// 3. Return highest scoring enabled agent
    /// 4. Error if no suitable agent found
    pub fn select_agent(&self, task: &Task) -> Result<Arc<dyn Agent>> {
        if self.agents.is_empty() {
            return Err(Error::Agent("No agents registered".to_string()));
        }

        // Filter enabled agents
        let enabled_agents: Vec<_> = self.agents.iter().filter(|a| a.is_enabled()).collect();

        if enabled_agents.is_empty() {
            return Err(Error::Agent("No enabled agents available".to_string()));
        }

        // If domain hint provided, filter by domain
        let candidates: Vec<_> = if let Some(ref domain_hint) = task.domain_hint {
            enabled_agents
                .into_iter()
                .filter(|a| a.capability().domain.eq_ignore_ascii_case(domain_hint))
                .collect()
        } else {
            enabled_agents
        };

        if candidates.is_empty() {
            if let Some(ref domain) = task.domain_hint {
                Err(Error::AgentNotFound(format!(
                    "No enabled agent found for domain '{}'",
                    domain
                )))
            } else {
                // No domain hint, fall through to keyword matching
                self.select_by_keywords(
                    &self
                        .agents
                        .iter()
                        .filter(|a| a.is_enabled())
                        .collect::<Vec<_>>(),
                    task,
                )
            }
        } else {
            // Score by keyword matching
            self.select_by_keywords(&candidates, task)
        }
    }

    /// Select agent by keyword matching
    fn select_by_keywords(
        &self,
        candidates: &[&Arc<dyn Agent>],
        task: &Task,
    ) -> Result<Arc<dyn Agent>> {
        let mut best_agent: Option<&Arc<dyn Agent>> = None;
        let mut best_score = 0;

        for agent in candidates {
            let capability = agent.capability();
            let score = self.score_capability(&capability, task);

            if score > best_score {
                best_score = score;
                best_agent = Some(agent);
            }
        }

        match best_agent {
            Some(agent) => Ok(Arc::clone(agent)),
            None => {
                // Fallback: return first candidate if no keywords matched
                if let Some(first) = candidates.first() {
                    Ok(Arc::clone(first))
                } else {
                    Err(Error::AgentNotFound(
                        "No suitable agent found for task".to_string(),
                    ))
                }
            }
        }
    }

    /// Score how well a capability matches a task
    fn score_capability(&self, capability: &AgentCapability, task: &Task) -> usize {
        let mut score = 0;
        let desc_lower = task.description.to_lowercase();

        // Check keyword matches
        for keyword in &capability.keywords {
            if desc_lower.contains(&keyword.to_lowercase()) {
                score += 10; // Each keyword match adds 10 points
            }
        }

        // Bonus if domain matches (even without domain_hint)
        if task
            .domain_hint
            .as_ref()
            .is_some_and(|d| capability.domain.eq_ignore_ascii_case(d))
        {
            score += 50;
        }

        score
    }

    /// Get all registered agents
    pub fn agents(&self) -> &[Arc<dyn Agent>] {
        &self.agents
    }

    /// Get agent by ID
    pub fn get_agent(&self, id: &str) -> Option<Arc<dyn Agent>> {
        self.agents.iter().find(|a| a.id() == id).cloned()
    }

    /// Create a router with a specific set of agents
    pub fn with_agents(agents: Vec<Arc<dyn Agent>>) -> Self {
        Self { agents }
    }
}

impl Default for AgentRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AgentRouter {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use bodhya_core::{AgentContext, AgentResult};

    // Mock agent for testing
    struct MockAgent {
        id: &'static str,
        capability: AgentCapability,
        enabled: bool,
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn id(&self) -> &'static str {
            self.id
        }

        fn capability(&self) -> AgentCapability {
            self.capability.clone()
        }

        async fn handle(&self, task: Task, _ctx: AgentContext) -> Result<AgentResult> {
            Ok(AgentResult::success(
                task.id,
                format!("Handled by {}", self.id),
            ))
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    fn create_code_agent() -> Arc<dyn Agent> {
        Arc::new(MockAgent {
            id: "code",
            capability: AgentCapability::new(
                "code",
                vec!["generate".to_string(), "refine".to_string()],
                "Code generation agent",
            )
            .with_keywords(vec![
                "code".to_string(),
                "rust".to_string(),
                "function".to_string(),
                "generate".to_string(),
            ]),
            enabled: true,
        })
    }

    fn create_mail_agent() -> Arc<dyn Agent> {
        Arc::new(MockAgent {
            id: "mail",
            capability: AgentCapability::new(
                "mail",
                vec!["draft".to_string(), "refine".to_string()],
                "Email writing agent",
            )
            .with_keywords(vec![
                "email".to_string(),
                "mail".to_string(),
                "write".to_string(),
                "draft".to_string(),
            ]),
            enabled: true,
        })
    }

    #[test]
    fn test_router_creation() {
        let router = AgentRouter::new();
        assert_eq!(router.agents().len(), 0);
    }

    #[test]
    fn test_register_agent() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());
        router.register(create_mail_agent());

        assert_eq!(router.agents().len(), 2);
    }

    #[test]
    fn test_select_agent_with_domain_hint() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());
        router.register(create_mail_agent());

        let task = Task::new("Some task description").with_domain("code");
        let agent = router.select_agent(&task).unwrap();

        assert_eq!(agent.id(), "code");
    }

    #[test]
    fn test_select_agent_by_keywords() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());
        router.register(create_mail_agent());

        // Task with "email" keyword should route to mail agent
        let task = Task::new("Write an email to the team");
        let agent = router.select_agent(&task).unwrap();
        assert_eq!(agent.id(), "mail");

        // Task with "rust" keyword should route to code agent
        let task = Task::new("Generate Rust code for a parser");
        let agent = router.select_agent(&task).unwrap();
        assert_eq!(agent.id(), "code");
    }

    #[test]
    fn test_select_agent_no_agents() {
        let router = AgentRouter::new();
        let task = Task::new("Test task");

        let result = router.select_agent(&task);
        assert!(result.is_err());
        match result {
            Err(Error::Agent(_)) => {}
            _ => panic!("Expected Error::Agent variant"),
        }
    }

    #[test]
    fn test_select_agent_domain_not_found() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());

        let task = Task::new("Test task").with_domain("nonexistent");
        let result = router.select_agent(&task);

        assert!(result.is_err());
        match result {
            Err(Error::AgentNotFound(_)) => {}
            _ => panic!("Expected Error::AgentNotFound variant"),
        }
    }

    #[test]
    fn test_select_agent_disabled() {
        let mut router = AgentRouter::new();

        // Add disabled agent
        let disabled = Arc::new(MockAgent {
            id: "disabled",
            capability: AgentCapability::new("test", vec![], "Disabled agent"),
            enabled: false,
        });
        router.register(disabled);

        let task = Task::new("Test task").with_domain("test");
        let result = router.select_agent(&task);

        assert!(result.is_err());
    }

    #[test]
    fn test_select_agent_filters_disabled() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());

        // Add disabled mail agent
        let disabled_mail = Arc::new(MockAgent {
            id: "mail_disabled",
            capability: AgentCapability::new("mail", vec![], "Disabled mail agent")
                .with_keywords(vec!["email".to_string()]),
            enabled: false,
        });
        router.register(disabled_mail);

        // Should not select disabled agent even though it matches keywords
        let task = Task::new("Write an email");
        let agent = router.select_agent(&task).unwrap();

        // Should fallback to code agent since mail is disabled
        assert_eq!(agent.id(), "code");
    }

    #[test]
    fn test_get_agent_by_id() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());
        router.register(create_mail_agent());

        let agent = router.get_agent("code").unwrap();
        assert_eq!(agent.id(), "code");

        let agent = router.get_agent("mail").unwrap();
        assert_eq!(agent.id(), "mail");

        assert!(router.get_agent("nonexistent").is_none());
    }

    #[test]
    fn test_keyword_scoring() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());
        router.register(create_mail_agent());

        // Multiple keyword matches should increase score
        let task = Task::new("Generate Rust code for a function");
        let agent = router.select_agent(&task).unwrap();
        assert_eq!(agent.id(), "code"); // Has "generate", "rust", "code", "function"
    }

    #[test]
    fn test_case_insensitive_matching() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());

        let task = Task::new("GENERATE RUST CODE");
        let agent = router.select_agent(&task).unwrap();
        assert_eq!(agent.id(), "code");
    }

    #[test]
    fn test_domain_hint_case_insensitive() {
        let mut router = AgentRouter::new();
        router.register(create_code_agent());

        let task = Task::new("Test").with_domain("CODE");
        let agent = router.select_agent(&task).unwrap();
        assert_eq!(agent.id(), "code");
    }
}
