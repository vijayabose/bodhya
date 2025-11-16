/// Code planning and task decomposition
///
/// This module handles the first step of the CodeAgent pipeline:
/// analyzing a task description and creating a structured plan.
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

/// A structured plan for code generation
#[derive(Clone, Debug, PartialEq)]
pub struct CodePlan {
    /// What the code is meant to do
    pub purpose: String,
    /// Main components needed (functions, structs, modules)
    pub components: Vec<String>,
    /// Key functional requirements
    pub requirements: Vec<String>,
    /// Edge cases to consider
    pub edge_cases: Vec<String>,
    /// High-level implementation approach
    pub approach: String,
}

impl CodePlan {
    /// Create a new empty plan
    pub fn new() -> Self {
        Self {
            purpose: String::new(),
            components: Vec::new(),
            requirements: Vec::new(),
            edge_cases: Vec::new(),
            approach: String::new(),
        }
    }

    /// Create a plan with a purpose
    pub fn with_purpose(purpose: impl Into<String>) -> Self {
        Self {
            purpose: purpose.into(),
            components: Vec::new(),
            requirements: Vec::new(),
            edge_cases: Vec::new(),
            approach: String::new(),
        }
    }

    /// Add a component to the plan
    pub fn add_component(&mut self, component: impl Into<String>) {
        self.components.push(component.into());
    }

    /// Add a requirement to the plan
    pub fn add_requirement(&mut self, requirement: impl Into<String>) {
        self.requirements.push(requirement.into());
    }

    /// Add an edge case to the plan
    pub fn add_edge_case(&mut self, edge_case: impl Into<String>) {
        self.edge_cases.push(edge_case.into());
    }

    /// Set the implementation approach
    pub fn set_approach(&mut self, approach: impl Into<String>) {
        self.approach = approach.into();
    }
}

impl Default for CodePlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Task planner for code generation
pub struct Planner {
    registry: Arc<ModelRegistry>,
    prompt_template: String,
}

impl Planner {
    /// Create a new planner with model registry
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        // Load prompt template from embedded resource or file
        let prompt_template = Self::load_prompt_template()?;

        Ok(Self {
            registry,
            prompt_template,
        })
    }

    /// Load the planner prompt template
    fn load_prompt_template() -> Result<String> {
        // Try to load from file first, fall back to embedded default
        let prompt_path = std::path::Path::new("prompts/code/planner.txt");

        if prompt_path.exists() {
            std::fs::read_to_string(prompt_path).map_err(|e| {
                bodhya_core::Error::Config(format!("Failed to load planner prompt: {}", e))
            })
        } else {
            // Embedded default prompt
            Ok(include_str!("../../../prompts/code/planner.txt").to_string())
        }
    }

    /// Generate a plan from a task description
    pub async fn plan(&self, task_description: &str) -> Result<CodePlan> {
        // For Phase 6, we'll use the model registry to call the planner model
        // Build the prompt from template
        let prompt = self
            .prompt_template
            .replace("{task_description}", task_description);

        // Get planner model from registry
        let model_info =
            self.registry
                .get_model(&ModelRole::Planner, "code", &EngagementMode::Minimum)?;

        // Create model request
        let request = ModelRequest::new(ModelRole::Planner, "code", prompt);

        // Call the model backend
        let backend = self.registry.get_backend(&model_info.id).ok_or_else(|| {
            bodhya_core::Error::Config(format!(
                "Backend '{}' not found for model '{}'",
                model_info.definition.backend, model_info.id
            ))
        })?;

        let response = backend.generate(request).await?;

        // Parse the model response into a structured plan
        // For Phase 6, we'll do simple parsing
        let plan = Self::parse_plan_from_response(&response.text, task_description);

        Ok(plan)
    }

    /// Parse plan from model response
    ///
    /// This is a simple parser for Phase 6. Future phases may use more sophisticated parsing.
    fn parse_plan_from_response(response: &str, task_description: &str) -> CodePlan {
        let mut plan = CodePlan::new();

        // Simple heuristic parsing
        // Look for sections marked with ** or ## headings

        let lines: Vec<&str> = response.lines().collect();
        let mut current_section = "";

        for line in lines {
            let trimmed = line.trim();

            // Detect sections
            if trimmed.starts_with("**Purpose") || trimmed.starts_with("## Purpose") {
                current_section = "purpose";
                // Extract content after colon if present
                if let Some(content) = trimmed.split(':').nth(1) {
                    let content = content.trim();
                    if !content.is_empty() {
                        plan.purpose = content.to_string();
                    }
                }
                continue;
            } else if trimmed.starts_with("**Components") || trimmed.starts_with("## Components") {
                current_section = "components";
                continue;
            } else if trimmed.starts_with("**Requirements")
                || trimmed.starts_with("## Requirements")
            {
                current_section = "requirements";
                continue;
            } else if trimmed.starts_with("**Edge Cases") || trimmed.starts_with("## Edge Cases") {
                current_section = "edge_cases";
                continue;
            } else if trimmed.starts_with("**Approach") || trimmed.starts_with("## Approach") {
                current_section = "approach";
                // Extract content after colon if present
                if let Some(content) = trimmed.split(':').nth(1) {
                    let content = content.trim();
                    if !content.is_empty() {
                        plan.approach = content.to_string();
                    }
                }
                continue;
            }

            // Skip empty lines and section headers
            if trimmed.is_empty() || trimmed.starts_with("**") || trimmed.starts_with("##") {
                continue;
            }

            // Add content to current section
            match current_section {
                "purpose" => {
                    if plan.purpose.is_empty() {
                        plan.purpose = trimmed.to_string();
                    } else {
                        plan.purpose.push(' ');
                        plan.purpose.push_str(trimmed);
                    }
                }
                "components" => {
                    // Remove bullet points and dashes
                    let component = trimmed
                        .trim_start_matches('-')
                        .trim_start_matches('*')
                        .trim();
                    if !component.is_empty() {
                        plan.add_component(component);
                    }
                }
                "requirements" => {
                    let req = trimmed
                        .trim_start_matches('-')
                        .trim_start_matches('*')
                        .trim();
                    if !req.is_empty() {
                        plan.add_requirement(req);
                    }
                }
                "edge_cases" => {
                    let edge_case = trimmed
                        .trim_start_matches('-')
                        .trim_start_matches('*')
                        .trim();
                    if !edge_case.is_empty() {
                        plan.add_edge_case(edge_case);
                    }
                }
                "approach" => {
                    if plan.approach.is_empty() {
                        plan.approach = trimmed.to_string();
                    } else {
                        plan.approach.push(' ');
                        plan.approach.push_str(trimmed);
                    }
                }
                _ => {}
            }
        }

        // Fallback: if we couldn't parse anything, create a basic plan
        if plan.purpose.is_empty() {
            plan.purpose = format!("Implement: {}", task_description);
        }

        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_plan_creation() {
        let plan = CodePlan::new();
        assert_eq!(plan.purpose, "");
        assert_eq!(plan.components.len(), 0);
        assert_eq!(plan.requirements.len(), 0);
        assert_eq!(plan.edge_cases.len(), 0);
        assert_eq!(plan.approach, "");
    }

    #[test]
    fn test_code_plan_with_purpose() {
        let plan = CodePlan::with_purpose("Build a config loader");
        assert_eq!(plan.purpose, "Build a config loader");
        assert_eq!(plan.components.len(), 0);
    }

    #[test]
    fn test_code_plan_add_component() {
        let mut plan = CodePlan::new();
        plan.add_component("load_config function");
        plan.add_component("Config struct");

        assert_eq!(plan.components.len(), 2);
        assert_eq!(plan.components[0], "load_config function");
        assert_eq!(plan.components[1], "Config struct");
    }

    #[test]
    fn test_code_plan_add_requirement() {
        let mut plan = CodePlan::new();
        plan.add_requirement("Must support YAML");
        plan.add_requirement("Must validate schema");

        assert_eq!(plan.requirements.len(), 2);
        assert_eq!(plan.requirements[0], "Must support YAML");
    }

    #[test]
    fn test_code_plan_add_edge_case() {
        let mut plan = CodePlan::new();
        plan.add_edge_case("Empty file");
        plan.add_edge_case("Malformed YAML");

        assert_eq!(plan.edge_cases.len(), 2);
        assert_eq!(plan.edge_cases[0], "Empty file");
    }

    #[test]
    fn test_code_plan_set_approach() {
        let mut plan = CodePlan::new();
        plan.set_approach("Use serde_yaml for parsing");

        assert_eq!(plan.approach, "Use serde_yaml for parsing");
    }

    #[test]
    fn test_code_plan_default() {
        let plan = CodePlan::default();
        assert_eq!(plan.purpose, "");
    }

    #[test]
    fn test_parse_plan_with_sections() {
        let response = r#"
**Purpose**: Load configuration from YAML files

**Components**:
- Config struct to hold settings
- load_config function to read and parse files
- validate function to check schema

**Requirements**:
- Must support YAML format
- Must validate required fields
- Must return helpful error messages

**Edge Cases**:
- Empty file
- Malformed YAML
- Missing required fields

**Approach**: Use serde_yaml for parsing and custom validation logic
"#;

        let plan = Planner::parse_plan_from_response(response, "config loader");

        assert!(plan.purpose.contains("Load configuration"));
        assert_eq!(plan.components.len(), 3);
        assert!(plan.components[0].contains("Config struct"));
        assert_eq!(plan.requirements.len(), 3);
        assert!(plan.requirements[0].contains("YAML"));
        assert_eq!(plan.edge_cases.len(), 3);
        assert!(plan.edge_cases[0].contains("Empty file"));
        assert!(plan.approach.contains("serde_yaml"));
    }

    #[test]
    fn test_parse_plan_fallback() {
        let response = "This is just some unstructured text";
        let plan = Planner::parse_plan_from_response(response, "build a function");

        // Should have fallback purpose
        assert!(plan.purpose.contains("Implement"));
        assert!(plan.purpose.contains("build a function"));
    }

    #[test]
    fn test_load_prompt_template() {
        let template = Planner::load_prompt_template();
        assert!(template.is_ok());
        let template = template.unwrap();
        assert!(template.contains("{task_description}"));
        assert!(template.contains("planning"));
    }
}
