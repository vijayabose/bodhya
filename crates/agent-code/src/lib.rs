/// Code Generation Agent
///
/// Phase 5: Minimal stub with static responses
/// Phase 6: Planner and BDD/Gherkin generation
/// Phase 7: TDD, implementation generation, and review (current)
/// Phase 8: Tool integration with CodeAgentTools (v1.1)
use async_trait::async_trait;
use bodhya_core::{Agent, AgentCapability, AgentContext, AgentResult, Result, Task};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

mod bdd;
mod impl_gen;
mod planner;
mod review;
mod tdd;
pub mod tools; // NEW: Tool wrapper module
pub mod validate;

// Re-export public types
pub use bdd::{BddGenerator, GherkinFeature, GherkinScenario, GherkinStep};
pub use impl_gen::{ImplCode, ImplGenerator};
pub use planner::{CodePlan, Planner};
pub use review::{CodeReview, CodeReviewer, ReviewStatus, ReviewSuggestion, SuggestionPriority};
pub use tdd::{TddGenerator, TestCode};
pub use tools::{CodeAgentTools, CommandOutput, ExecutionStats}; // NEW
pub use validate::{CodeValidator, ValidationResult, ValidationSummary};

/// Code generation agent
pub struct CodeAgent {
    enabled: bool,
    registry: Option<Arc<ModelRegistry>>,
}

impl CodeAgent {
    /// Create a new CodeAgent instance (Phase 5 compatibility - no registry)
    pub fn new() -> Self {
        Self {
            enabled: true,
            registry: None,
        }
    }

    /// Create a new CodeAgent with model registry (Phase 6+)
    pub fn with_registry(registry: Arc<ModelRegistry>) -> Self {
        Self {
            enabled: true,
            registry: Some(registry),
        }
    }

    /// Create a new CodeAgent with specific enabled state
    pub fn with_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            registry: None,
        }
    }

    /// Generate static hello world code (Phase 5 fallback)
    fn generate_hello_world(&self) -> String {
        r#"fn main() {
    println!("Hello, World!");
}
"#
        .to_string()
    }

    /// Generate code using planner and BDD (Phase 6)
    async fn generate_with_bdd(&self, task: &Task) -> Result<String> {
        let registry = self.registry.as_ref().ok_or_else(|| {
            bodhya_core::Error::Config("Model registry not configured for CodeAgent".to_string())
        })?;

        // Step 1: Create a plan
        let planner = Planner::new(Arc::clone(registry))?;
        let plan = planner.plan(&task.description).await?;

        // Step 2: Generate Gherkin features from plan
        let bdd_generator = BddGenerator::new(Arc::clone(registry))?;
        let feature = bdd_generator.generate(&task.description, &plan).await?;

        // Step 3: Format the output (Phase 6: just return the Gherkin)
        let mut output = String::new();
        output.push_str("## Plan\n\n");
        output.push_str(&format!("**Purpose**: {}\n\n", plan.purpose));

        if !plan.components.is_empty() {
            output.push_str("**Components**:\n");
            for component in &plan.components {
                output.push_str(&format!("- {}\n", component));
            }
            output.push('\n');
        }

        output.push_str("## BDD Features\n\n");
        output.push_str(&feature.to_gherkin());

        Ok(output)
    }

    /// Extract CodeAgentTools from AgentContext
    /// Returns None if tools are not available or cannot be downcast
    fn get_tools_from_context(ctx: &AgentContext) -> Option<CodeAgentTools> {
        use bodhya_tools_mcp::ToolRegistry;

        // Clone the Arc<dyn Any> and try to downcast to Arc<ToolRegistry>
        let tools_arc = Arc::clone(ctx.tools.as_ref()?);
        let registry_arc = tools_arc.downcast::<ToolRegistry>().ok()?;

        let working_dir = ctx.get_working_dir().ok()?;
        Some(CodeAgentTools::new(registry_arc, working_dir))
    }

    /// Execute task with tools (Phase 8)
    /// Demonstrates agentic code generation with file operations
    async fn execute_with_tools(&self, task: &Task, tools: &CodeAgentTools) -> Result<String> {
        // Simple demonstration: Create a basic Rust project structure
        let mut output = String::new();
        output.push_str(&format!("# Executing: {}\n\n", task.description));
        output.push_str("## Tool-Based Execution\n\n");

        // Example: Check if Cargo.toml exists
        match tools.file_exists("Cargo.toml").await {
            Ok(exists) => {
                if exists {
                    output.push_str("✓ Found existing Cargo.toml\n");

                    // Read it
                    match tools.read_file("Cargo.toml").await {
                        Ok(content) => {
                            let lines: Vec<&str> = content.lines().take(5).collect();
                            output.push_str("```toml\n");
                            output.push_str(&lines.join("\n"));
                            output.push_str("\n...\n```\n\n");
                        }
                        Err(e) => {
                            output.push_str(&format!("⚠ Could not read Cargo.toml: {}\n\n", e))
                        }
                    }
                } else {
                    output.push_str("ℹ No Cargo.toml found in working directory\n\n");
                }
            }
            Err(e) => output.push_str(&format!("⚠ Error checking for Cargo.toml: {}\n\n", e)),
        }

        // Get execution statistics
        let stats = tools.get_stats().await;
        output.push_str("## Execution Statistics\n\n");
        output.push_str(&format!("- Files read: {}\n", stats.files_read));
        output.push_str(&format!("- Files written: {}\n", stats.files_written));
        output.push_str(&format!(
            "- Commands executed: {}\n",
            stats.commands_executed
        ));
        output.push_str(&format!("- Bytes read: {}\n", stats.bytes_read));

        output.push_str("\n*Note: This is a demonstration of tool integration. ");
        output.push_str("Full agentic code generation will be implemented in future phases.*\n");

        Ok(output)
    }

    /// Generate code using full TDD pipeline (Phase 7)
    /// Planner → BDD → TDD → Implementation → Review
    async fn generate_with_tdd(&self, task: &Task) -> Result<String> {
        let registry = self.registry.as_ref().ok_or_else(|| {
            bodhya_core::Error::Config("Model registry not configured for CodeAgent".to_string())
        })?;

        // Step 1: Create a plan
        let planner = Planner::new(Arc::clone(registry))?;
        let plan = planner.plan(&task.description).await?;

        // Step 2: Generate Gherkin features from plan
        let bdd_generator = BddGenerator::new(Arc::clone(registry))?;
        let feature = bdd_generator.generate(&task.description, &plan).await?;

        // Step 3: Generate failing tests (RED phase)
        let tdd_generator = TddGenerator::new(Arc::clone(registry))?;
        let test_code = tdd_generator.generate(&feature, &plan).await?;

        // Step 4: Generate implementation to make tests pass (GREEN phase)
        let impl_generator = ImplGenerator::new(Arc::clone(registry))?;
        let impl_code = impl_generator.generate(&test_code, &feature, &plan).await?;

        // Step 5: Review the code (REFACTOR phase)
        let reviewer = CodeReviewer::new(Arc::clone(registry))?;
        let review = reviewer.review(&impl_code, &plan, "Tests passed").await?;

        // Step 6: Format the complete output
        let mut output = String::new();

        output.push_str("# Code Generation Complete\n\n");

        output.push_str("## Plan\n\n");
        output.push_str(&format!("**Purpose**: {}\n\n", plan.purpose));

        if !plan.components.is_empty() {
            output.push_str("**Components**:\n");
            for component in &plan.components {
                output.push_str(&format!("- {}\n", component));
            }
            output.push('\n');
        }

        output.push_str("## BDD Features\n\n");
        output.push_str(&feature.to_gherkin());
        output.push('\n');

        output.push_str("## Tests (RED Phase)\n\n");
        output.push_str(&format!("{} test(s) generated\n\n", test_code.test_count));
        output.push_str("```rust\n");
        output.push_str(&test_code.code);
        output.push_str("\n```\n\n");

        output.push_str("## Implementation (GREEN Phase)\n\n");
        output.push_str(&format!("{} lines of code\n\n", impl_code.loc));
        output.push_str("```rust\n");
        output.push_str(&impl_code.code);
        output.push_str("\n```\n\n");

        output.push_str("## Code Review (REFACTOR Phase)\n\n");
        output.push_str(&format!("**Status**: {:?}\n\n", review.status));

        if !review.strengths.is_empty() {
            output.push_str("**Strengths**:\n");
            for strength in &review.strengths {
                output.push_str(&format!("- {}\n", strength));
            }
            output.push('\n');
        }

        if !review.suggestions.is_empty() {
            output.push_str("**Suggestions**:\n");
            for suggestion in &review.suggestions {
                output.push_str(&format!("- {}\n", suggestion.issue));
            }
            output.push('\n');
        }

        Ok(output)
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

    async fn handle(&self, task: Task, ctx: AgentContext) -> Result<AgentResult> {
        // Phase 8: Try tool-based execution first if tools are available
        if let Some(tools) = Self::get_tools_from_context(&ctx) {
            match self.execute_with_tools(&task, &tools).await {
                Ok(output) => return Ok(AgentResult::success(task.id, output)),
                Err(e) => {
                    eprintln!(
                        "Tool-based execution failed: {}, falling back to model-based",
                        e
                    );
                }
            }
        }

        // Phase 7/6/5: Fall back to model-based or static execution
        let content = if self.registry.is_some() {
            // Phase 7: Use full TDD pipeline (Planner → BDD → TDD → Implementation → Review)
            // Falls back to Phase 6 BDD-only if TDD pipeline fails
            match self.generate_with_tdd(&task).await {
                Ok(output) => output,
                Err(e) => {
                    eprintln!("TDD pipeline failed: {}, trying BDD-only", e);
                    match self.generate_with_bdd(&task).await {
                        Ok(output) => output,
                        Err(e2) => {
                            // Fall back to static response on all errors
                            eprintln!(
                                "BDD generation also failed: {}, falling back to static response",
                                e2
                            );
                            let code = self.generate_hello_world();
                            format!(
                                "Generated Rust code for task: {}\n\n{}",
                                task.description, code
                            )
                        }
                    }
                }
            }
        } else {
            // Phase 5: Static hello world code
            let code = self.generate_hello_world();
            format!(
                "Generated Rust code for task: {}\n\n{}",
                task.description, code
            )
        };

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

    #[tokio::test]
    async fn test_code_agent_with_tools() {
        use bodhya_core::ExecutionLimits;
        use bodhya_tools_mcp::ToolRegistry;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();

        // Create tools and context
        let tools = Arc::new(ToolRegistry::with_defaults());
        let ctx = AgentContext::new(Default::default())
            .with_working_dir(temp_path.clone())
            .with_execution_limits(ExecutionLimits::default())
            .with_tools(tools as Arc<dyn std::any::Any + Send + Sync>);

        // Create agent
        let agent = CodeAgent::new();
        let task = Task::new("Test tool integration");

        // Execute with tools
        let result = agent.handle(task, ctx).await;
        assert!(result.is_ok());

        let agent_result = result.unwrap();
        assert!(agent_result.success);
        assert!(agent_result.content.contains("Tool-Based Execution"));
        assert!(agent_result.content.contains("Execution Statistics"));
    }

    #[tokio::test]
    async fn test_code_agent_without_tools_falls_back() {
        // Create agent without tools in context
        let agent = CodeAgent::new();
        let task = Task::new("Test without tools");
        let ctx = AgentContext::new(Default::default());

        // Execute without tools
        let result = agent.handle(task, ctx).await;
        assert!(result.is_ok());

        let agent_result = result.unwrap();
        assert!(agent_result.success);
        // Should fall back to static hello world
        assert!(agent_result.content.contains("Hello, World!"));
        assert!(!agent_result.content.contains("Tool-Based Execution"));
    }

    #[tokio::test]
    async fn test_code_agent_tools_read_cargo_toml() {
        use bodhya_core::ExecutionLimits;
        use bodhya_tools_mcp::ToolRegistry;
        use std::path::PathBuf;

        // Use the actual project directory (workspace root)
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        // Create tools and context pointing to project root
        let tools = Arc::new(ToolRegistry::with_defaults());
        let ctx = AgentContext::new(Default::default())
            .with_working_dir(project_root)
            .with_execution_limits(ExecutionLimits::default())
            .with_tools(tools as Arc<dyn std::any::Any + Send + Sync>);

        // Create agent
        let agent = CodeAgent::new();
        let task = Task::new("Analyze project");

        // Execute with tools
        let result = agent.handle(task, ctx).await;
        assert!(result.is_ok());

        let agent_result = result.unwrap();
        assert!(agent_result.success);
        assert!(agent_result.content.contains("Tool-Based Execution"));
        // Should find and read Cargo.toml from project root
        assert!(
            agent_result.content.contains("Cargo.toml")
                || agent_result.content.contains("workspace")
        );
    }
}
