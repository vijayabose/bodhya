/// Code Generation Agent (Phase 5 stub)
///
/// This is a minimal stub implementation for Phase 5 vertical slice.
/// It implements the Agent trait and returns static code responses.
/// Full BDD/TDD implementation will be added in Phase 6-7.
use async_trait::async_trait;
use bodhya_core::{Agent, AgentCapability, AgentContext, AgentResult, Result, Task};

/// Code generation agent stub
pub struct CodeAgent {
    enabled: bool,
}

impl CodeAgent {
    /// Create a new CodeAgent instance
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Create a new CodeAgent with specific enabled state
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Generate static hello world code
    fn generate_hello_world(&self) -> String {
        r#"fn main() {
    println!("Hello, World!");
}
"#
        .to_string()
    }
}

impl Default for CodeAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for CodeAgent {
    fn id(&self) -> &'static str {
        "code"
    }

    fn capability(&self) -> AgentCapability {
        AgentCapability {
            domain: "code".to_string(),
            intents: vec![
                "generate".to_string(),
                "write".to_string(),
                "implement".to_string(),
                "create".to_string(),
            ],
            keywords: vec![
                "code".to_string(),
                "function".to_string(),
                "rust".to_string(),
                "generate".to_string(),
                "write".to_string(),
                "implement".to_string(),
                "create".to_string(),
                "program".to_string(),
                "hello".to_string(),
                "world".to_string(),
            ],
            description: "Generates Rust code and implements functions".to_string(),
        }
    }

    async fn handle(&self, task: Task, _ctx: AgentContext) -> Result<AgentResult> {
        // Phase 5 stub: return static hello world code for any request
        let code = self.generate_hello_world();

        let content = format!(
            "Generated Rust code for task: {}\n\n{}",
            task.description, code
        );

        Ok(AgentResult::success(task.id, content))
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_agent_creation() {
        let agent = CodeAgent::new();
        assert_eq!(agent.id(), "code");
        assert!(agent.is_enabled());
    }

    #[test]
    fn test_code_agent_default() {
        let agent = CodeAgent::default();
        assert_eq!(agent.id(), "code");
        assert!(agent.is_enabled());
    }

    #[test]
    fn test_code_agent_with_enabled() {
        let agent_enabled = CodeAgent::with_enabled(true);
        assert!(agent_enabled.is_enabled());

        let agent_disabled = CodeAgent::with_enabled(false);
        assert!(!agent_disabled.is_enabled());
    }

    #[test]
    fn test_code_agent_capability() {
        let agent = CodeAgent::new();
        let cap = agent.capability();

        assert_eq!(cap.domain, "code");
        assert!(cap.keywords.contains(&"code".to_string()));
        assert!(cap.keywords.contains(&"rust".to_string()));
        assert!(cap.keywords.contains(&"function".to_string()));
        assert!(!cap.description.is_empty());
    }

    #[tokio::test]
    async fn test_code_agent_handle_returns_success() {
        let agent = CodeAgent::new();
        let task = Task::new("Generate a hello world function");
        let ctx = AgentContext::new(Default::default());

        let result = agent.handle(task, ctx).await;
        assert!(result.is_ok());

        let agent_result = result.unwrap();
        assert!(agent_result.success);
        assert!(agent_result.content.contains("Hello, World!"));
        assert!(agent_result.content.contains("fn main()"));
    }

    #[tokio::test]
    async fn test_code_agent_handle_includes_task_description() {
        let agent = CodeAgent::new();
        let task = Task::new("Write a function that adds two numbers");
        let ctx = AgentContext::new(Default::default());

        let result = agent.handle(task, ctx).await.unwrap();
        assert!(result
            .content
            .contains("Write a function that adds two numbers"));
    }

    #[tokio::test]
    async fn test_code_agent_handle_different_tasks() {
        let agent = CodeAgent::new();
        let ctx = AgentContext::new(Default::default());

        let tasks = vec![
            "Generate hello world",
            "Create a Rust function",
            "Implement a calculator",
        ];

        for task_desc in tasks {
            let task = Task::new(task_desc);
            let result = agent.handle(task, ctx.clone()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().success);
        }
    }

    #[test]
    fn test_generate_hello_world() {
        let agent = CodeAgent::new();
        let code = agent.generate_hello_world();

        assert!(code.contains("fn main()"));
        assert!(code.contains("println!"));
        assert!(code.contains("Hello, World!"));
    }
}
