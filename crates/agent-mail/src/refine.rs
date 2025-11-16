/// Email refinement and improvement
///
/// This module handles refining email drafts for better tone, clarity, and professionalism.
use crate::draft::EmailDraft;
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

/// Refinement goals for an email
#[derive(Clone, Debug, PartialEq)]
pub enum RefinementGoal {
    /// Improve clarity and remove ambiguity
    Clarity,
    /// Make tone more polite and professional
    Tone,
    /// Reduce verbosity while maintaining completeness
    Conciseness,
    /// All of the above
    All,
}

impl RefinementGoal {
    /// Convert to string for prompt
    pub fn as_str(&self) -> &str {
        match self {
            RefinementGoal::Clarity => "improve clarity and remove ambiguity",
            RefinementGoal::Tone => "make tone more polite and professional",
            RefinementGoal::Conciseness => "reduce verbosity while maintaining completeness",
            RefinementGoal::All => "improve clarity, tone, and conciseness",
        }
    }
}

/// A refined email with improvement notes
#[derive(Clone, Debug, PartialEq)]
pub struct RefinedEmail {
    /// The refined email draft
    pub draft: EmailDraft,
    /// List of changes made
    pub changes: Vec<String>,
}

impl RefinedEmail {
    /// Create a new refined email
    pub fn new(draft: EmailDraft, changes: Vec<String>) -> Self {
        Self { draft, changes }
    }

    /// Parse refined email from model response
    pub fn from_text(text: &str) -> Self {
        let mut refined_text = String::new();
        let mut changes = Vec::new();
        let mut in_refined_section = false;
        let mut in_changes_section = false;

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("**Refined Email**:") || trimmed.starts_with("Refined Email:") {
                in_refined_section = true;
                in_changes_section = false;
                continue;
            }

            if trimmed.starts_with("**Changes Made**:") || trimmed.starts_with("Changes Made:") {
                in_refined_section = false;
                in_changes_section = true;
                continue;
            }

            if in_refined_section && !trimmed.is_empty() && !trimmed.starts_with("**") {
                if !refined_text.is_empty() {
                    refined_text.push('\n');
                }
                refined_text.push_str(line);
            }

            if in_changes_section && trimmed.starts_with('-') {
                let change = trimmed.trim_start_matches('-').trim();
                if !change.is_empty() {
                    changes.push(change.to_string());
                }
            }
        }

        // Fallback: if no structured format found, use entire text
        if refined_text.is_empty() {
            refined_text = text.to_string();
        }

        let draft = EmailDraft::from_text(&refined_text);
        Self::new(draft, changes)
    }
}

/// Email refiner
pub struct EmailRefiner {
    registry: Arc<ModelRegistry>,
    prompt_template: String,
}

impl EmailRefiner {
    /// Create a new email refiner
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        let prompt_template = Self::load_prompt_template()?;

        Ok(Self {
            registry,
            prompt_template,
        })
    }

    /// Load the refine prompt template
    fn load_prompt_template() -> Result<String> {
        let prompt_path = std::path::Path::new("prompts/mail/refine.txt");

        if prompt_path.exists() {
            std::fs::read_to_string(prompt_path).map_err(|e| {
                bodhya_core::Error::Config(format!("Failed to load refine prompt: {}", e))
            })
        } else {
            // Embedded default prompt
            Ok(include_str!("../../../prompts/mail/refine.txt").to_string())
        }
    }

    /// Refine an email draft
    pub async fn refine(&self, draft: &EmailDraft, goal: RefinementGoal) -> Result<RefinedEmail> {
        // Build prompt from template
        let prompt = self
            .prompt_template
            .replace("{original_draft}", &draft.full_email)
            .replace("{goals}", goal.as_str());

        // Get writer model from registry (for email refinement)
        let model_info =
            self.registry
                .get_model(&ModelRole::Writer, "mail", &EngagementMode::Minimum)?;

        // Create model request
        let request = ModelRequest::new(ModelRole::Writer, "mail", prompt);

        // Call the model backend
        let backend = self.registry.get_backend(&model_info.id).ok_or_else(|| {
            bodhya_core::Error::Config(format!(
                "Backend '{}' not found for model '{}'",
                model_info.definition.backend, model_info.id
            ))
        })?;

        let response = backend.generate(request).await?;

        // Parse refined email from response
        Ok(RefinedEmail::from_text(&response.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refinement_goal_as_str() {
        assert_eq!(
            RefinementGoal::Clarity.as_str(),
            "improve clarity and remove ambiguity"
        );
        assert_eq!(
            RefinementGoal::Tone.as_str(),
            "make tone more polite and professional"
        );
        assert_eq!(
            RefinementGoal::Conciseness.as_str(),
            "reduce verbosity while maintaining completeness"
        );
        assert_eq!(
            RefinementGoal::All.as_str(),
            "improve clarity, tone, and conciseness"
        );
    }

    #[test]
    fn test_refined_email_creation() {
        let draft = EmailDraft::new("Test", "Body");
        let changes = vec!["Improved clarity".to_string(), "Fixed grammar".to_string()];

        let refined = RefinedEmail::new(draft, changes);

        assert_eq!(refined.draft.subject, "Test");
        assert_eq!(refined.changes.len(), 2);
        assert_eq!(refined.changes[0], "Improved clarity");
    }

    #[test]
    fn test_refined_email_from_text() {
        let text = r#"
**Refined Email**:
Subject: Meeting Follow-up

Dear Sarah,

Thank you for taking the time to meet with me yesterday. I wanted to follow up on our discussion about the project timeline.

Best regards,
John

**Changes Made**:
- Made tone more professional
- Improved clarity of main points
- Fixed grammar issues
"#;

        let refined = RefinedEmail::from_text(text);

        assert!(refined.draft.body.contains("Dear Sarah"));
        assert!(refined.draft.body.contains("Thank you"));
        assert_eq!(refined.changes.len(), 3);
        assert!(refined.changes[0].contains("professional"));
    }

    #[test]
    fn test_refined_email_from_text_simple() {
        let text = r#"
Refined Email:
Subject: Quick Update

Hi Team,

Just a quick update on the project.

Thanks

Changes Made:
- Shortened email
- Improved subject line
"#;

        let refined = RefinedEmail::from_text(text);

        assert!(refined.draft.body.contains("Hi Team"));
        assert_eq!(refined.changes.len(), 2);
    }

    #[test]
    fn test_refined_email_from_text_fallback() {
        let text = "This is just the refined email text without structure";

        let refined = RefinedEmail::from_text(text);

        assert!(refined.draft.body.contains("refined email text"));
        assert_eq!(refined.changes.len(), 0);
    }

    #[test]
    fn test_load_prompt_template() {
        let template = EmailRefiner::load_prompt_template();
        assert!(template.is_ok());
        let template = template.unwrap();
        assert!(template.contains("{original_draft}"));
        assert!(template.contains("{goals}"));
        assert!(template.contains("refine") || template.contains("improve"));
    }

    #[test]
    fn test_refined_email_with_multiple_changes() {
        let draft = EmailDraft::new("Original Subject", "Original body");
        let changes = vec![
            "Change 1".to_string(),
            "Change 2".to_string(),
            "Change 3".to_string(),
            "Change 4".to_string(),
        ];

        let refined = RefinedEmail::new(draft, changes);

        assert_eq!(refined.changes.len(), 4);
        assert_eq!(refined.changes[3], "Change 4");
    }

    #[test]
    fn test_refined_email_no_changes() {
        let draft = EmailDraft::new("Subject", "Body");
        let changes = Vec::new();

        let refined = RefinedEmail::new(draft, changes);

        assert_eq!(refined.changes.len(), 0);
    }
}
