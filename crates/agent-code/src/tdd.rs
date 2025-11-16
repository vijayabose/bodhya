/// TDD test generation (RED phase)
///
/// This module handles generating failing tests from Gherkin scenarios.
use crate::bdd::GherkinFeature;
use crate::planner::CodePlan;
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

/// Generated test code
#[derive(Clone, Debug, PartialEq)]
pub struct TestCode {
    /// The generated Rust test code
    pub code: String,
    /// Number of test cases generated
    pub test_count: usize,
}

impl TestCode {
    /// Create a new TestCode instance
    pub fn new(code: impl Into<String>) -> Self {
        let code = code.into();
        let test_count = Self::count_tests(&code);
        Self { code, test_count }
    }

    /// Count the number of #[test] and #[tokio::test] attributes in the code
    fn count_tests(code: &str) -> usize {
        code.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed == "#[test]" || trimmed == "#[tokio::test]"
            })
            .count()
    }
}

/// TDD test generator
pub struct TddGenerator {
    registry: Arc<ModelRegistry>,
    prompt_template: String,
}

impl TddGenerator {
    /// Create a new TDD generator
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        let prompt_template = Self::load_prompt_template()?;

        Ok(Self {
            registry,
            prompt_template,
        })
    }

    /// Load the TDD prompt template
    fn load_prompt_template() -> Result<String> {
        let prompt_path = std::path::Path::new("prompts/code/tdd.txt");

        if prompt_path.exists() {
            std::fs::read_to_string(prompt_path).map_err(|e| {
                bodhya_core::Error::Config(format!("Failed to load TDD prompt: {}", e))
            })
        } else {
            // Embedded default prompt
            Ok(include_str!("../../../prompts/code/tdd.txt").to_string())
        }
    }

    /// Generate test code from Gherkin feature and plan
    pub async fn generate(&self, feature: &GherkinFeature, plan: &CodePlan) -> Result<TestCode> {
        // Build prompt from template
        let gherkin_text = feature.to_gherkin();
        let plan_text = self.format_plan(plan);

        let prompt = self
            .prompt_template
            .replace("{gherkin_feature}", &gherkin_text)
            .replace("{plan_context}", &plan_text);

        // Get coder model from registry (TDD uses coder model in Phase 7)
        let model_info =
            self.registry
                .get_model(&ModelRole::Coder, "code", &EngagementMode::Minimum)?;

        // Create model request
        let request = ModelRequest::new(ModelRole::Coder, "code", prompt);

        // Call the model backend
        let backend = self.registry.get_backend(&model_info.id).ok_or_else(|| {
            bodhya_core::Error::Config(format!(
                "Backend '{}' not found for model '{}'",
                model_info.definition.backend, model_info.id
            ))
        })?;

        let response = backend.generate(request).await?;

        // Extract Rust code from response
        let test_code = Self::extract_rust_code(&response.text);

        Ok(TestCode::new(test_code))
    }

    /// Format a plan for inclusion in the prompt
    fn format_plan(&self, plan: &CodePlan) -> String {
        let mut output = String::new();

        output.push_str(&format!("Purpose: {}\n", plan.purpose));

        if !plan.components.is_empty() {
            output.push_str("\nComponents:\n");
            for component in &plan.components {
                output.push_str(&format!("- {}\n", component));
            }
        }

        output
    }

    /// Extract Rust code from markdown code blocks
    fn extract_rust_code(response: &str) -> String {
        let lines: Vec<&str> = response.lines().collect();
        let mut in_code_block = false;
        let mut code_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("```rust") || trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                code_lines.push(line);
            }
        }

        if code_lines.is_empty() {
            // No code block found, return the whole response
            response.to_string()
        } else {
            code_lines.join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_code_creation() {
        let code = r#"
#[test]
fn test_example() {
    assert_eq!(1, 1);
}
"#;
        let test_code = TestCode::new(code);
        assert_eq!(test_code.test_count, 1);
        assert!(test_code.code.contains("test_example"));
    }

    #[test]
    fn test_count_tests() {
        let code = r#"
#[test]
fn test_one() {}

#[tokio::test]
async fn test_two() {}

#[test]
fn test_three() {}
"#;
        let count = TestCode::count_tests(code);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_tests_no_tests() {
        let code = "fn regular_function() {}";
        let count = TestCode::count_tests(code);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_extract_rust_code() {
        let response = r#"
Here's the test code:

```rust
#[test]
fn test_example() {
    assert!(true);
}
```

This test will verify the functionality.
"#;

        let code = TddGenerator::extract_rust_code(response);
        assert!(code.contains("#[test]"));
        assert!(code.contains("test_example"));
        assert!(!code.contains("Here's the test code"));
    }

    #[test]
    fn test_extract_rust_code_no_markers() {
        let response = "Just some plain text";
        let code = TddGenerator::extract_rust_code(response);
        assert_eq!(code, "Just some plain text");
    }

    #[test]
    fn test_format_plan() {
        let mut plan = CodePlan::with_purpose("Build a calculator");
        plan.add_component("add function");
        plan.add_component("subtract function");

        use bodhya_model_registry::ModelManifest;
        use std::collections::HashMap;
        let manifest = ModelManifest {
            models: HashMap::new(),
            backends: HashMap::new(),
        };
        let registry = Arc::new(ModelRegistry::from_manifest(manifest, "/tmp/models"));
        let generator = TddGenerator::new(registry).unwrap();

        let formatted = generator.format_plan(&plan);

        assert!(formatted.contains("Purpose: Build a calculator"));
        assert!(formatted.contains("Components:"));
        assert!(formatted.contains("- add function"));
        assert!(formatted.contains("- subtract function"));
    }

    #[test]
    fn test_load_prompt_template() {
        let template = TddGenerator::load_prompt_template();
        assert!(template.is_ok());
        let template = template.unwrap();
        assert!(template.contains("{gherkin_feature}"));
        assert!(template.contains("{plan_context}"));
        assert!(template.contains("TDD") || template.contains("test"));
    }

    #[test]
    fn test_test_code_from_actual_test() {
        let code = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(5 - 3, 2);
    }
}
"#;
        let test_code = TestCode::new(code);
        assert_eq!(test_code.test_count, 2);
    }
}
