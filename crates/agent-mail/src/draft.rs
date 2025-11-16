/// Email draft generation
///
/// This module handles generating email drafts from context and purpose.
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

/// A generated email draft
#[derive(Clone, Debug, PartialEq)]
pub struct EmailDraft {
    /// Email subject line
    pub subject: String,
    /// Email body content
    pub body: String,
    /// Complete formatted email
    pub full_email: String,
}

impl EmailDraft {
    /// Create a new email draft
    pub fn new(subject: impl Into<String>, body: impl Into<String>) -> Self {
        let subject = subject.into();
        let body = body.into();
        let full_email = format!("Subject: {}\n\n{}", subject, body);

        Self {
            subject,
            body,
            full_email,
        }
    }

    /// Parse email from formatted text
    pub fn from_text(text: &str) -> Self {
        let mut subject = String::new();
        let mut body = String::new();
        let mut in_body = false;
        let mut subject_found = false;

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("**Subject**:") || trimmed.starts_with("Subject:") {
                subject = trimmed
                    .strip_prefix("**Subject**:")
                    .or_else(|| trimmed.strip_prefix("Subject:"))
                    .unwrap_or("")
                    .trim()
                    .to_string();
                subject_found = true;
                continue;
            }

            if trimmed.starts_with("**Body**:") || trimmed.starts_with("Body:") {
                in_body = true;
                continue;
            }

            // After subject is found, start collecting body (unless explicit Body: marker seen)
            if subject_found && !in_body && !trimmed.is_empty() && !trimmed.starts_with("**") {
                in_body = true;
            }

            if in_body && !trimmed.is_empty() && !trimmed.starts_with("**") {
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(line);
            }
        }

        // Fallback: if no structured format found, use entire text as body
        if subject.is_empty() && body.is_empty() {
            body = text.to_string();
            subject = "Email Draft".to_string();
        }

        Self::new(subject, body)
    }
}

/// Email draft generator
pub struct DraftGenerator {
    registry: Arc<ModelRegistry>,
    prompt_template: String,
}

impl DraftGenerator {
    /// Create a new draft generator
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        let prompt_template = Self::load_prompt_template()?;

        Ok(Self {
            registry,
            prompt_template,
        })
    }

    /// Load the draft prompt template
    fn load_prompt_template() -> Result<String> {
        let prompt_path = std::path::Path::new("prompts/mail/draft.txt");

        if prompt_path.exists() {
            std::fs::read_to_string(prompt_path).map_err(|e| {
                bodhya_core::Error::Config(format!("Failed to load draft prompt: {}", e))
            })
        } else {
            // Embedded default prompt
            Ok(include_str!("../../../prompts/mail/draft.txt").to_string())
        }
    }

    /// Generate an email draft
    pub async fn generate(&self, context: &str, purpose: &str) -> Result<EmailDraft> {
        // Build prompt from template
        let prompt = self
            .prompt_template
            .replace("{context}", context)
            .replace("{purpose}", purpose);

        // Get drafter model from registry (uses general/mail writer model)
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

        // Parse email draft from response
        Ok(EmailDraft::from_text(&response.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_draft_creation() {
        let draft = EmailDraft::new("Meeting Request", "Let's schedule a meeting next week.");

        assert_eq!(draft.subject, "Meeting Request");
        assert_eq!(draft.body, "Let's schedule a meeting next week.");
        assert!(draft.full_email.contains("Subject: Meeting Request"));
        assert!(draft.full_email.contains("Let's schedule a meeting"));
    }

    #[test]
    fn test_email_draft_from_text() {
        let text = r#"
**Subject**: Project Update

**Body**:
Dear Team,

I wanted to provide a quick update on our project progress.

Best regards,
John
"#;

        let draft = EmailDraft::from_text(text);

        assert_eq!(draft.subject, "Project Update");
        assert!(draft.body.contains("Dear Team"));
        assert!(draft.body.contains("Best regards"));
    }

    #[test]
    fn test_email_draft_from_text_simple_format() {
        let text = r#"
Subject: Follow-up

Body:
Hi there,

Following up on our last conversation.

Thanks
"#;

        let draft = EmailDraft::from_text(text);

        assert_eq!(draft.subject, "Follow-up");
        assert!(draft.body.contains("Hi there"));
    }

    #[test]
    fn test_email_draft_from_text_fallback() {
        let text = "This is just some plain text without structure";

        let draft = EmailDraft::from_text(text);

        assert_eq!(draft.subject, "Email Draft");
        assert_eq!(draft.body, text);
    }

    #[test]
    fn test_load_prompt_template() {
        let template = DraftGenerator::load_prompt_template();
        assert!(template.is_ok());
        let template = template.unwrap();
        assert!(template.contains("{context}"));
        assert!(template.contains("{purpose}"));
        assert!(template.contains("email") || template.contains("Email"));
    }

    #[test]
    fn test_email_draft_multiline_body() {
        let draft = EmailDraft::new("Weekly Report", "Line 1\nLine 2\nLine 3");

        assert_eq!(draft.subject, "Weekly Report");
        assert!(draft.body.contains("Line 1"));
        assert!(draft.body.contains("Line 2"));
        assert!(draft.body.contains("Line 3"));
    }

    #[test]
    fn test_email_draft_empty_fields() {
        let draft = EmailDraft::new("", "");

        assert_eq!(draft.subject, "");
        assert_eq!(draft.body, "");
        assert_eq!(draft.full_email, "Subject: \n\n");
    }
}
