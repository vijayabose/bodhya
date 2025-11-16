/// BDD and Gherkin feature generation
///
/// This module handles generating Gherkin feature files from task descriptions and plans.
use crate::planner::CodePlan;
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

/// A Gherkin feature with scenarios
#[derive(Clone, Debug, PartialEq)]
pub struct GherkinFeature {
    /// Feature name
    pub name: String,
    /// Feature description
    pub description: String,
    /// Scenarios in the feature
    pub scenarios: Vec<GherkinScenario>,
}

impl GherkinFeature {
    /// Create a new empty feature
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            scenarios: Vec::new(),
        }
    }

    /// Add a scenario to the feature
    pub fn add_scenario(&mut self, scenario: GherkinScenario) {
        self.scenarios.push(scenario);
    }

    /// Format as Gherkin text
    pub fn to_gherkin(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Feature: {}\n", self.name));
        output.push_str(&format!("  {}\n", self.description));

        for scenario in &self.scenarios {
            output.push('\n');
            output.push_str(&format!("  Scenario: {}\n", scenario.name));

            for step in &scenario.steps {
                output.push_str(&format!("    {} {}\n", step.keyword, step.text));
            }
        }

        output
    }
}

/// A Gherkin scenario with Given/When/Then steps
#[derive(Clone, Debug, PartialEq)]
pub struct GherkinScenario {
    /// Scenario name
    pub name: String,
    /// Steps in the scenario
    pub steps: Vec<GherkinStep>,
}

impl GherkinScenario {
    /// Create a new scenario
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    /// Add a step to the scenario
    pub fn add_step(&mut self, step: GherkinStep) {
        self.steps.push(step);
    }

    /// Add a Given step
    pub fn given(&mut self, text: impl Into<String>) {
        self.add_step(GherkinStep::given(text));
    }

    /// Add a When step
    pub fn when(&mut self, text: impl Into<String>) {
        self.add_step(GherkinStep::when(text));
    }

    /// Add a Then step
    pub fn then(&mut self, text: impl Into<String>) {
        self.add_step(GherkinStep::then(text));
    }
}

/// A Gherkin step (Given/When/Then/And)
#[derive(Clone, Debug, PartialEq)]
pub struct GherkinStep {
    /// Step keyword (Given, When, Then, And)
    pub keyword: String,
    /// Step text
    pub text: String,
}

impl GherkinStep {
    /// Create a new step
    pub fn new(keyword: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            keyword: keyword.into(),
            text: text.into(),
        }
    }

    /// Create a Given step
    pub fn given(text: impl Into<String>) -> Self {
        Self::new("Given", text)
    }

    /// Create a When step
    pub fn when(text: impl Into<String>) -> Self {
        Self::new("When", text)
    }

    /// Create a Then step
    pub fn then(text: impl Into<String>) -> Self {
        Self::new("Then", text)
    }

    /// Create an And step
    pub fn and(text: impl Into<String>) -> Self {
        Self::new("And", text)
    }
}

/// BDD feature generator
pub struct BddGenerator {
    registry: Arc<ModelRegistry>,
    prompt_template: String,
}

impl BddGenerator {
    /// Create a new BDD generator
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        let prompt_template = Self::load_prompt_template()?;

        Ok(Self {
            registry,
            prompt_template,
        })
    }

    /// Load the BDD prompt template
    fn load_prompt_template() -> Result<String> {
        let prompt_path = std::path::Path::new("prompts/code/bdd.txt");

        if prompt_path.exists() {
            std::fs::read_to_string(prompt_path).map_err(|e| {
                bodhya_core::Error::Config(format!("Failed to load BDD prompt: {}", e))
            })
        } else {
            // Embedded default prompt
            Ok(include_str!("../../../prompts/code/bdd.txt").to_string())
        }
    }

    /// Generate Gherkin features from task description and plan
    pub async fn generate(
        &self,
        task_description: &str,
        plan: &CodePlan,
    ) -> Result<GherkinFeature> {
        // Build prompt from template
        let plan_text = self.format_plan(plan);
        let prompt = self
            .prompt_template
            .replace("{task_description}", task_description)
            .replace("{plan}", &plan_text);

        // Get planner model from registry (BDD uses same model as planner in Phase 6)
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

        // Parse Gherkin from response
        let feature = Self::parse_gherkin_from_response(&response.text, task_description);

        Ok(feature)
    }

    /// Format a plan for inclusion in the prompt
    fn format_plan(&self, plan: &CodePlan) -> String {
        let mut output = String::new();

        output.push_str(&format!("Purpose: {}\n\n", plan.purpose));

        if !plan.components.is_empty() {
            output.push_str("Components:\n");
            for component in &plan.components {
                output.push_str(&format!("- {}\n", component));
            }
            output.push('\n');
        }

        if !plan.requirements.is_empty() {
            output.push_str("Requirements:\n");
            for req in &plan.requirements {
                output.push_str(&format!("- {}\n", req));
            }
            output.push('\n');
        }

        if !plan.edge_cases.is_empty() {
            output.push_str("Edge Cases:\n");
            for edge_case in &plan.edge_cases {
                output.push_str(&format!("- {}\n", edge_case));
            }
            output.push('\n');
        }

        if !plan.approach.is_empty() {
            output.push_str(&format!("Approach: {}\n", plan.approach));
        }

        output
    }

    /// Parse Gherkin from model response
    ///
    /// Simple parser for Phase 6. Looks for Feature/Scenario/Given/When/Then keywords.
    fn parse_gherkin_from_response(response: &str, task_description: &str) -> GherkinFeature {
        let lines: Vec<&str> = response.lines().collect();

        let mut feature_name = String::new();
        let mut feature_description = String::new();
        let mut scenarios: Vec<GherkinScenario> = Vec::new();
        let mut current_scenario: Option<GherkinScenario> = None;
        let mut in_code_block = false;

        for line in lines {
            let trimmed = line.trim();

            // Skip code block markers
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if !in_code_block {
                continue;
            }

            // Parse Feature
            if trimmed.starts_with("Feature:") {
                feature_name = trimmed
                    .strip_prefix("Feature:")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                continue;
            }

            // Parse Scenario
            if trimmed.starts_with("Scenario:") {
                // Save previous scenario if any
                if let Some(scenario) = current_scenario.take() {
                    scenarios.push(scenario);
                }

                let scenario_name = trimmed.strip_prefix("Scenario:").unwrap_or("").trim();
                current_scenario = Some(GherkinScenario::new(scenario_name));
                continue;
            }

            // Parse steps
            if let Some(ref mut scenario) = current_scenario {
                if trimmed.starts_with("Given ") {
                    let text = trimmed.strip_prefix("Given ").unwrap_or("");
                    scenario.given(text);
                } else if trimmed.starts_with("When ") {
                    let text = trimmed.strip_prefix("When ").unwrap_or("");
                    scenario.when(text);
                } else if trimmed.starts_with("Then ") {
                    let text = trimmed.strip_prefix("Then ").unwrap_or("");
                    scenario.then(text);
                } else if trimmed.starts_with("And ") {
                    let text = trimmed.strip_prefix("And ").unwrap_or("");
                    scenario.add_step(GherkinStep::and(text));
                }
            } else if !trimmed.is_empty() && feature_description.is_empty() {
                // Feature description (before first scenario)
                feature_description = trimmed.to_string();
            }
        }

        // Save last scenario
        if let Some(scenario) = current_scenario {
            scenarios.push(scenario);
        }

        // Fallback if parsing failed
        if feature_name.is_empty() {
            feature_name = format!("Code for: {}", task_description);
        }

        if feature_description.is_empty() {
            feature_description = "Generated feature".to_string();
        }

        let mut feature = GherkinFeature::new(feature_name, feature_description);
        for scenario in scenarios {
            feature.add_scenario(scenario);
        }

        // If no scenarios were parsed, create a default one
        if feature.scenarios.is_empty() {
            let mut scenario = GherkinScenario::new("Basic behavior");
            scenario.given("the module exists");
            scenario.when("it is used");
            scenario.then("it behaves correctly");
            feature.add_scenario(scenario);
        }

        feature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gherkin_feature_creation() {
        let feature = GherkinFeature::new("Config loader", "Load configuration from files");
        assert_eq!(feature.name, "Config loader");
        assert_eq!(feature.description, "Load configuration from files");
        assert_eq!(feature.scenarios.len(), 0);
    }

    #[test]
    fn test_gherkin_scenario_creation() {
        let scenario = GherkinScenario::new("Load valid config");
        assert_eq!(scenario.name, "Load valid config");
        assert_eq!(scenario.steps.len(), 0);
    }

    #[test]
    fn test_gherkin_scenario_add_steps() {
        let mut scenario = GherkinScenario::new("Test scenario");
        scenario.given("a valid config file");
        scenario.when("the config is loaded");
        scenario.then("the config is parsed correctly");

        assert_eq!(scenario.steps.len(), 3);
        assert_eq!(scenario.steps[0].keyword, "Given");
        assert_eq!(scenario.steps[1].keyword, "When");
        assert_eq!(scenario.steps[2].keyword, "Then");
    }

    #[test]
    fn test_gherkin_step_creation() {
        let step = GherkinStep::given("a precondition");
        assert_eq!(step.keyword, "Given");
        assert_eq!(step.text, "a precondition");

        let step = GherkinStep::when("an action");
        assert_eq!(step.keyword, "When");

        let step = GherkinStep::then("an outcome");
        assert_eq!(step.keyword, "Then");

        let step = GherkinStep::and("another condition");
        assert_eq!(step.keyword, "And");
    }

    #[test]
    fn test_feature_to_gherkin() {
        let mut feature = GherkinFeature::new("Math operations", "Test basic math");

        let mut scenario = GherkinScenario::new("Addition");
        scenario.given("two numbers");
        scenario.when("they are added");
        scenario.then("the result is correct");

        feature.add_scenario(scenario);

        let gherkin = feature.to_gherkin();

        assert!(gherkin.contains("Feature: Math operations"));
        assert!(gherkin.contains("Test basic math"));
        assert!(gherkin.contains("Scenario: Addition"));
        assert!(gherkin.contains("Given two numbers"));
        assert!(gherkin.contains("When they are added"));
        assert!(gherkin.contains("Then the result is correct"));
    }

    #[test]
    fn test_parse_gherkin_from_response() {
        let response = r#"
Here's the Gherkin feature:

```gherkin
Feature: Config loader
  Load and validate configuration files

  Scenario: Load valid YAML config
    Given a valid config.yaml file
    When the load_config function is called
    Then the Config struct is populated
    And no errors are returned

  Scenario: Handle missing file
    Given the config file does not exist
    When load_config is called
    Then an error is returned
```
"#;

        let feature = BddGenerator::parse_gherkin_from_response(response, "config loader");

        assert_eq!(feature.name, "Config loader");
        assert_eq!(feature.scenarios.len(), 2);

        let scenario1 = &feature.scenarios[0];
        assert_eq!(scenario1.name, "Load valid YAML config");
        assert_eq!(scenario1.steps.len(), 4);
        assert_eq!(scenario1.steps[0].keyword, "Given");
        assert!(scenario1.steps[0].text.contains("valid config.yaml"));

        let scenario2 = &feature.scenarios[1];
        assert_eq!(scenario2.name, "Handle missing file");
        assert_eq!(scenario2.steps.len(), 3);
    }

    #[test]
    fn test_parse_gherkin_fallback() {
        let response = "This is not valid Gherkin";
        let feature = BddGenerator::parse_gherkin_from_response(response, "test task");

        // Should have fallback values
        assert!(feature.name.contains("test task"));
        assert_eq!(feature.scenarios.len(), 1); // Default scenario
        assert_eq!(feature.scenarios[0].name, "Basic behavior");
    }

    #[test]
    fn test_format_plan() {
        let mut plan = CodePlan::with_purpose("Build a config loader");
        plan.add_component("Config struct");
        plan.add_component("load_config function");
        plan.add_requirement("Support YAML");
        plan.add_edge_case("Empty file");
        plan.set_approach("Use serde_yaml");

        // Create a minimal registry (won't actually use it for formatting)
        use bodhya_model_registry::ModelManifest;
        use std::collections::HashMap;
        let manifest = ModelManifest {
            models: HashMap::new(),
            backends: HashMap::new(),
        };
        let registry = Arc::new(ModelRegistry::from_manifest(manifest, "/tmp/models"));
        let generator = BddGenerator::new(registry).unwrap();

        let formatted = generator.format_plan(&plan);

        assert!(formatted.contains("Purpose: Build a config loader"));
        assert!(formatted.contains("Components:"));
        assert!(formatted.contains("- Config struct"));
        assert!(formatted.contains("Requirements:"));
        assert!(formatted.contains("- Support YAML"));
        assert!(formatted.contains("Edge Cases:"));
        assert!(formatted.contains("- Empty file"));
        assert!(formatted.contains("Approach: Use serde_yaml"));
    }

    #[test]
    fn test_load_prompt_template() {
        let template = BddGenerator::load_prompt_template();
        assert!(template.is_ok());
        let template = template.unwrap();
        assert!(template.contains("{task_description}"));
        assert!(template.contains("{plan}"));
        assert!(template.contains("Gherkin"));
    }
}
