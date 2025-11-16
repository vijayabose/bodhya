/// Code review and improvement suggestions (REFACTOR phase)
///
/// This module handles reviewing generated code and suggesting improvements.
use crate::impl_gen::ImplCode;
use crate::planner::CodePlan;
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

/// Code review result
#[derive(Clone, Debug, PartialEq)]
pub enum ReviewStatus {
    /// Code is approved without changes
    Approved,
    /// Code needs minor changes
    NeedsMinorChanges,
    /// Code needs major changes
    NeedsMajorChanges,
}

/// Review suggestion
#[derive(Clone, Debug, PartialEq)]
pub enum SuggestionPriority {
    High,
    Medium,
    Low,
}

/// A single review suggestion
#[derive(Clone, Debug, PartialEq)]
pub struct ReviewSuggestion {
    /// Description of the issue
    pub issue: String,
    /// Recommended fix
    pub recommendation: String,
    /// Priority level
    pub priority: SuggestionPriority,
}

impl ReviewSuggestion {
    /// Create a new suggestion
    pub fn new(
        issue: impl Into<String>,
        recommendation: impl Into<String>,
        priority: SuggestionPriority,
    ) -> Self {
        Self {
            issue: issue.into(),
            recommendation: recommendation.into(),
            priority,
        }
    }
}

/// Code review result
#[derive(Clone, Debug, PartialEq)]
pub struct CodeReview {
    /// Overall review status
    pub status: ReviewStatus,
    /// What the code does well
    pub strengths: Vec<String>,
    /// Suggestions for improvement
    pub suggestions: Vec<ReviewSuggestion>,
    /// Refactoring opportunities
    pub refactoring_opportunities: Vec<String>,
    /// Raw review text from model
    pub raw_review: String,
}

impl CodeReview {
    /// Create a new review
    pub fn new(raw_review: impl Into<String>) -> Self {
        let raw_review = raw_review.into();
        let status = Self::parse_status(&raw_review);
        let strengths = Self::parse_strengths(&raw_review);
        let suggestions = Self::parse_suggestions(&raw_review);
        let refactoring_opportunities = Self::parse_refactoring(&raw_review);

        Self {
            status,
            strengths,
            suggestions,
            refactoring_opportunities,
            raw_review,
        }
    }

    /// Parse review status from text
    fn parse_status(text: &str) -> ReviewStatus {
        let upper = text.to_uppercase();
        if upper.contains("APPROVED") && !upper.contains("NEEDS") {
            ReviewStatus::Approved
        } else if upper.contains("NEEDS_MAJOR_CHANGES") || upper.contains("MAJOR CHANGES") {
            ReviewStatus::NeedsMajorChanges
        } else if upper.contains("NEEDS_MINOR_CHANGES") || upper.contains("MINOR CHANGES") {
            ReviewStatus::NeedsMinorChanges
        } else {
            // Default to needing minor changes if unclear
            ReviewStatus::NeedsMinorChanges
        }
    }

    /// Parse strengths from review text
    fn parse_strengths(text: &str) -> Vec<String> {
        let mut strengths = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut in_strengths_section = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("## Strengths") {
                in_strengths_section = true;
                continue;
            }

            if in_strengths_section {
                if trimmed.starts_with("##") {
                    break;
                }
                if trimmed.starts_with('-') {
                    let strength = trimmed.trim_start_matches('-').trim();
                    if !strength.is_empty() {
                        strengths.push(strength.to_string());
                    }
                }
            }
        }

        strengths
    }

    /// Parse suggestions from review text (simplified for Phase 7)
    fn parse_suggestions(text: &str) -> Vec<ReviewSuggestion> {
        let mut suggestions = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut in_suggestions_section = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("## Suggestions") {
                in_suggestions_section = true;
                continue;
            }

            if in_suggestions_section {
                if trimmed.starts_with("##") {
                    break;
                }
                if trimmed.contains("**Issue**:") {
                    // Handle both "**Issue**:" and "1. **Issue**:"
                    let issue = if let Some(pos) = trimmed.find("**Issue**:") {
                        trimmed[pos + 10..].trim().to_string()
                    } else {
                        String::new()
                    };

                    if !issue.is_empty() {
                        suggestions.push(ReviewSuggestion {
                            issue,
                            recommendation: String::new(),
                            priority: SuggestionPriority::Medium,
                        });
                    }
                }
            }
        }

        suggestions
    }

    /// Parse refactoring opportunities from review text
    fn parse_refactoring(text: &str) -> Vec<String> {
        let mut opportunities = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut in_refactoring_section = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("## Refactoring") {
                in_refactoring_section = true;
                continue;
            }

            if in_refactoring_section {
                if trimmed.starts_with("##") {
                    break;
                }
                if trimmed.starts_with('-') {
                    let opportunity = trimmed.trim_start_matches('-').trim();
                    if !opportunity.is_empty() {
                        opportunities.push(opportunity.to_string());
                    }
                }
            }
        }

        opportunities
    }
}

/// Code reviewer
pub struct CodeReviewer {
    registry: Arc<ModelRegistry>,
    prompt_template: String,
}

impl CodeReviewer {
    /// Create a new code reviewer
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        let prompt_template = Self::load_prompt_template()?;

        Ok(Self {
            registry,
            prompt_template,
        })
    }

    /// Load the reviewer prompt template
    fn load_prompt_template() -> Result<String> {
        let prompt_path = std::path::Path::new("prompts/code/reviewer.txt");

        if prompt_path.exists() {
            std::fs::read_to_string(prompt_path).map_err(|e| {
                bodhya_core::Error::Config(format!("Failed to load reviewer prompt: {}", e))
            })
        } else {
            // Embedded default prompt
            Ok(include_str!("../../../prompts/code/reviewer.txt").to_string())
        }
    }

    /// Review generated code
    pub async fn review(
        &self,
        impl_code: &ImplCode,
        plan: &CodePlan,
        test_results: &str,
    ) -> Result<CodeReview> {
        // Build prompt from template
        let plan_text = self.format_plan(plan);

        let prompt = self
            .prompt_template
            .replace("{plan_context}", &plan_text)
            .replace("{generated_code}", &impl_code.code)
            .replace("{test_results}", test_results);

        // Get reviewer model from registry
        let model_info =
            self.registry
                .get_model(&ModelRole::Reviewer, "code", &EngagementMode::Minimum)?;

        // Create model request
        let request = ModelRequest::new(ModelRole::Reviewer, "code", prompt);

        // Call the model backend
        let backend = self.registry.get_backend(&model_info.id).ok_or_else(|| {
            bodhya_core::Error::Config(format!(
                "Backend '{}' not found for model '{}'",
                model_info.definition.backend, model_info.id
            ))
        })?;

        let response = backend.generate(request).await?;

        // Parse review from response
        Ok(CodeReview::new(response.text))
    }

    /// Format a plan for inclusion in the prompt
    fn format_plan(&self, plan: &CodePlan) -> String {
        let mut output = String::new();

        output.push_str(&format!("Purpose: {}\n", plan.purpose));

        if !plan.requirements.is_empty() {
            output.push_str("\nRequirements:\n");
            for req in &plan.requirements {
                output.push_str(&format!("- {}\n", req));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_status_parsing() {
        assert_eq!(
            CodeReview::parse_status("## Review Summary\nAPPROVED"),
            ReviewStatus::Approved
        );
        assert_eq!(
            CodeReview::parse_status("## Review Summary\nNEEDS_MINOR_CHANGES"),
            ReviewStatus::NeedsMinorChanges
        );
        assert_eq!(
            CodeReview::parse_status("## Review Summary\nNEEDS_MAJOR_CHANGES"),
            ReviewStatus::NeedsMajorChanges
        );
    }

    #[test]
    fn test_parse_strengths() {
        let review_text = r#"
## Review Summary
APPROVED

## Strengths
- Well-structured code
- Good error handling
- Clear documentation

## Suggestions
None
"#;

        let strengths = CodeReview::parse_strengths(review_text);
        assert_eq!(strengths.len(), 3);
        assert_eq!(strengths[0], "Well-structured code");
        assert_eq!(strengths[1], "Good error handling");
        assert_eq!(strengths[2], "Clear documentation");
    }

    #[test]
    fn test_parse_suggestions() {
        let review_text = r#"
## Suggestions for Improvement
1. **Issue**: Missing error handling
   **Recommendation**: Add Result types
   **Priority**: HIGH

2. **Issue**: Poor variable names
   **Recommendation**: Use descriptive names
   **Priority**: MEDIUM
"#;

        let suggestions = CodeReview::parse_suggestions(review_text);
        assert!(suggestions.len() >= 2);
        assert!(suggestions[0].issue.contains("Missing error handling"));
        assert!(suggestions[1].issue.contains("Poor variable names"));
    }

    #[test]
    fn test_parse_refactoring() {
        let review_text = r#"
## Refactoring Opportunities
- Extract common logic into helper function
- Consider using builder pattern
- Reduce function complexity

## Conclusion
Overall good code.
"#;

        let refactoring = CodeReview::parse_refactoring(review_text);
        assert_eq!(refactoring.len(), 3);
        assert!(refactoring[0].contains("Extract common logic"));
        assert!(refactoring[1].contains("builder pattern"));
    }

    #[test]
    fn test_code_review_creation() {
        let review_text = r#"
## Review Summary
APPROVED

## Strengths
- Clean code
- Good tests

## Suggestions for Improvement
None

## Refactoring Opportunities
- None
"#;

        let review = CodeReview::new(review_text);
        assert_eq!(review.status, ReviewStatus::Approved);
        assert_eq!(review.strengths.len(), 2);
    }

    #[test]
    fn test_review_suggestion_creation() {
        let suggestion = ReviewSuggestion::new(
            "Missing docs",
            "Add documentation",
            SuggestionPriority::High,
        );

        assert_eq!(suggestion.issue, "Missing docs");
        assert_eq!(suggestion.recommendation, "Add documentation");
        assert_eq!(suggestion.priority, SuggestionPriority::High);
    }

    #[test]
    fn test_load_prompt_template() {
        let template = CodeReviewer::load_prompt_template();
        assert!(template.is_ok());
        let template = template.unwrap();
        assert!(template.contains("{plan_context}"));
        assert!(template.contains("{generated_code}"));
        assert!(template.contains("review") || template.contains("Review"));
    }

    #[test]
    fn test_format_plan() {
        let mut plan = CodePlan::with_purpose("Build a web server");
        plan.add_requirement("Must handle concurrent requests");
        plan.add_requirement("Must support REST API");

        use bodhya_model_registry::ModelManifest;
        use std::collections::HashMap;
        let manifest = ModelManifest {
            models: HashMap::new(),
            backends: HashMap::new(),
        };
        let registry = Arc::new(ModelRegistry::from_manifest(manifest, "/tmp/models"));
        let reviewer = CodeReviewer::new(registry).unwrap();

        let formatted = reviewer.format_plan(&plan);

        assert!(formatted.contains("Purpose: Build a web server"));
        assert!(formatted.contains("Requirements:"));
        assert!(formatted.contains("- Must handle concurrent requests"));
    }
}
