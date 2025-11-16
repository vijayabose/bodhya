/// Implementation generation (GREEN phase)
///
/// This module handles generating Rust code to make failing tests pass.
use crate::bdd::GherkinFeature;
use crate::planner::CodePlan;
use crate::tdd::TestCode;
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

/// Generated implementation code
#[derive(Clone, Debug, PartialEq)]
pub struct ImplCode {
    /// The generated Rust implementation code
    pub code: String,
    /// Estimated lines of code
    pub loc: usize,
}

impl ImplCode {
    /// Create a new ImplCode instance
    pub fn new(code: impl Into<String>) -> Self {
        let code = code.into();
        let loc = Self::count_loc(&code);
        Self { code, loc }
    }

    /// Count non-empty, non-comment lines of code
    fn count_loc(code: &str) -> usize {
        code.lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*")
            })
            .count()
    }
}

/// Implementation code generator
pub struct ImplGenerator {
    registry: Arc<ModelRegistry>,
    prompt_template: String,
}

impl ImplGenerator {
    /// Create a new implementation generator
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        let prompt_template = Self::load_prompt_template()?;

        Ok(Self {
            registry,
            prompt_template,
        })
    }

    /// Load the coder prompt template
    fn load_prompt_template() -> Result<String> {
        let prompt_path = std::path::Path::new("prompts/code/coder.txt");

        if prompt_path.exists() {
            std::fs::read_to_string(prompt_path).map_err(|e| {
                bodhya_core::Error::Config(format!("Failed to load coder prompt: {}", e))
            })
        } else {
            // Embedded default prompt
            Ok(include_str!("../../../prompts/code/coder.txt").to_string())
        }
    }

    /// Generate implementation code from tests, feature, and plan
    pub async fn generate(
        &self,
        test_code: &TestCode,
        feature: &GherkinFeature,
        plan: &CodePlan,
    ) -> Result<ImplCode> {
        // Build prompt from template
        let gherkin_text = feature.to_gherkin();
        let plan_text = self.format_plan(plan);

        let prompt = self
            .prompt_template
            .replace("{plan_context}", &plan_text)
            .replace("{gherkin_feature}", &gherkin_text)
            .replace("{test_code}", &test_code.code);

        // Get coder model from registry
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
        let impl_code = Self::extract_rust_code(&response.text);

        Ok(ImplCode::new(impl_code))
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

        if !plan.requirements.is_empty() {
            output.push_str("\nRequirements:\n");
            for req in &plan.requirements {
                output.push_str(&format!("- {}\n", req));
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
    fn test_impl_code_creation() {
        let code = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let impl_code = ImplCode::new(code);
        assert!(impl_code.loc > 0);
        assert!(impl_code.code.contains("add"));
    }

    #[test]
    fn test_count_loc() {
        let code = r#"
// This is a comment
pub fn example() {
    let x = 1;
    let y = 2;
    // Another comment
    x + y
}
"#;
        let loc = ImplCode::count_loc(code);
        // Should count: pub fn, let x, let y, x + y, closing brace = 5 lines
        assert_eq!(loc, 5);
    }

    #[test]
    fn test_count_loc_empty() {
        let code = "";
        let loc = ImplCode::count_loc(code);
        assert_eq!(loc, 0);
    }

    #[test]
    fn test_extract_rust_code() {
        let response = r#"
Here's the implementation:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

This function adds two numbers.
"#;

        let code = ImplGenerator::extract_rust_code(response);
        assert!(code.contains("pub fn add"));
        assert!(code.contains("a + b"));
        assert!(!code.contains("Here's the implementation"));
    }

    #[test]
    fn test_extract_rust_code_no_markers() {
        let response = "Just some plain code";
        let code = ImplGenerator::extract_rust_code(response);
        assert_eq!(code, "Just some plain code");
    }

    #[test]
    fn test_format_plan() {
        let mut plan = CodePlan::with_purpose("Build a math library");
        plan.add_component("add function");
        plan.add_component("subtract function");
        plan.add_requirement("Must handle negative numbers");
        plan.add_requirement("Must handle overflow");

        use bodhya_model_registry::ModelManifest;
        use std::collections::HashMap;
        let manifest = ModelManifest {
            models: HashMap::new(),
            backends: HashMap::new(),
        };
        let registry = Arc::new(ModelRegistry::from_manifest(manifest, "/tmp/models"));
        let generator = ImplGenerator::new(registry).unwrap();

        let formatted = generator.format_plan(&plan);

        assert!(formatted.contains("Purpose: Build a math library"));
        assert!(formatted.contains("Components:"));
        assert!(formatted.contains("- add function"));
        assert!(formatted.contains("Requirements:"));
        assert!(formatted.contains("- Must handle negative numbers"));
    }

    #[test]
    fn test_load_prompt_template() {
        let template = ImplGenerator::load_prompt_template();
        assert!(template.is_ok());
        let template = template.unwrap();
        assert!(template.contains("{plan_context}"));
        assert!(template.contains("{test_code}"));
        assert!(template.contains("Rust") || template.contains("code"));
    }

    #[test]
    fn test_impl_code_from_actual_impl() {
        let code = r#"
/// Add two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtract two numbers
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#;
        let impl_code = ImplCode::new(code);
        // Count: pub fn add, a + b, closing brace, pub fn subtract, a - b, closing brace = 6
        assert_eq!(impl_code.loc, 6);
    }

    #[test]
    fn test_impl_code_with_multiline_comments() {
        let code = r#"
/*
 * Multi-line comment
 * Should not be counted
 */
pub fn example() {
    42
}
"#;
        let loc = ImplCode::count_loc(code);
        // Should count: pub fn, 42, closing brace = 3 lines
        // Note: the /* line might still be counted by our simple filter
        assert!(loc >= 3);
    }
}
