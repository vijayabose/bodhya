/// Mail Generation Agent
///
/// Phase 8: Email drafting and refinement
use async_trait::async_trait;
use bodhya_core::{Agent, AgentCapability, AgentContext, AgentResult, Result, Task};
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

mod classify;
mod draft;
mod refine;

// Re-export public types
pub use classify::{EmailCategory, EmailClassification, EmailClassifier};
pub use draft::{DraftGenerator, EmailDraft};
pub use refine::{EmailRefiner, RefinedEmail, RefinementGoal};

/// Mail generation agent
pub struct MailAgent {
    enabled: bool,
    registry: Option<Arc<ModelRegistry>>,
}

impl MailAgent {
    /// Create a new MailAgent instance (without registry)
    pub fn new() -> Self {
        Self {
            enabled: true,
            registry: None,
        }
    }

    /// Create a new MailAgent with model registry
    pub fn with_registry(registry: Arc<ModelRegistry>) -> Self {
        Self {
            enabled: true,
            registry: Some(registry),
        }
    }

    /// Create a new MailAgent with specific enabled state
    pub fn with_enabled(enabled: bool) -> Self {
        Self {
            enabled,
            registry: None,
        }
    }

    /// Generate a static email (fallback)
    fn generate_static_email(&self, task_description: &str) -> String {
        format!(
            "Subject: Regarding: {}\n\nDear Recipient,\n\nThis is a placeholder email regarding your request.\n\nBest regards"
            ,
            task_description
        )
    }

    /// Generate email with drafting and optional refinement
    async fn generate_email(&self, task: &Task) -> Result<String> {
        let registry = self.registry.as_ref().ok_or_else(|| {
            bodhya_core::Error::Config("Model registry not configured for MailAgent".to_string())
        })?;

        // Extract context and purpose from task description
        // Simple heuristic: first 100 chars as context, rest as purpose
        let (context, purpose) = if task.description.len() > 100 {
            (&task.description[..100], &task.description[100..])
        } else {
            ("General correspondence", task.description.as_str())
        };

        // Step 1: Generate initial draft
        let draft_generator = DraftGenerator::new(Arc::clone(registry))?;
        let draft = draft_generator.generate(context, purpose).await?;

        // Step 2: Refine the draft
        let refiner = EmailRefiner::new(Arc::clone(registry))?;
        let refined = refiner.refine(&draft, RefinementGoal::All).await?;

        // Step 3: Format the output
        let mut output = String::new();

        output.push_str("# Email Generation Complete\n\n");

        output.push_str("## Final Email\n\n");
        output.push_str(&refined.draft.full_email);
        output.push('\n');

        if !refined.changes.is_empty() {
            output.push_str("\n## Refinements Applied\n\n");
            for change in &refined.changes {
                output.push_str(&format!("- {}\n", change));
            }
        }

        Ok(output)
    }
}

impl Default for MailAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for MailAgent {
    fn id(&self) -> &'static str {
        "mail"
    }

    fn capability(&self) -> AgentCapability {
        AgentCapability {
            domain: "mail".to_string(),
            intents: vec![
                "draft".to_string(),
                "write".to_string(),
                "compose".to_string(),
                "email".to_string(),
            ],
            keywords: vec![
                "email".to_string(),
                "mail".to_string(),
                "draft".to_string(),
                "write".to_string(),
                "compose".to_string(),
                "letter".to_string(),
                "message".to_string(),
                "correspondence".to_string(),
            ],
            description: "Drafts and refines emails with appropriate tone and clarity".to_string(),
        }
    }

    async fn handle(&self, task: Task, _ctx: AgentContext) -> Result<AgentResult> {
        let content = if self.registry.is_some() {
            // Use draft and refine pipeline
            match self.generate_email(&task).await {
                Ok(output) => output,
                Err(e) => {
                    // Fall back to static email on error
                    eprintln!("Email generation failed: {}, falling back to static", e);
                    self.generate_static_email(&task.description)
                }
            }
        } else {
            // No registry: static email
            self.generate_static_email(&task.description)
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
    fn test_mail_agent_creation() {
        let agent = MailAgent::new();
        assert_eq!(agent.id(), "mail");
        assert!(agent.is_enabled());
    }

    #[test]
    fn test_mail_agent_default() {
        let agent = MailAgent::default();
        assert_eq!(agent.id(), "mail");
        assert!(agent.is_enabled());
    }

    #[test]
    fn test_mail_agent_with_enabled() {
        let agent_enabled = MailAgent::with_enabled(true);
        assert!(agent_enabled.is_enabled());

        let agent_disabled = MailAgent::with_enabled(false);
        assert!(!agent_disabled.is_enabled());
    }

    #[test]
    fn test_mail_agent_capability() {
        let agent = MailAgent::new();
        let cap = agent.capability();

        assert_eq!(cap.domain, "mail");
        assert!(cap.keywords.contains(&"email".to_string()));
        assert!(cap.keywords.contains(&"draft".to_string()));
        assert!(cap.keywords.contains(&"mail".to_string()));
        assert!(!cap.description.is_empty());
    }

    #[tokio::test]
    async fn test_mail_agent_handle_returns_success() {
        let agent = MailAgent::new();
        let task = Task::new("Write a follow-up email");
        let ctx = AgentContext::new(Default::default());

        let result = agent.handle(task, ctx).await;
        assert!(result.is_ok());

        let agent_result = result.unwrap();
        assert!(agent_result.success);
        assert!(agent_result.content.contains("Subject"));
    }

    #[tokio::test]
    async fn test_mail_agent_handle_includes_task_description() {
        let agent = MailAgent::new();
        let task = Task::new("Meeting invitation for next Tuesday");
        let ctx = AgentContext::new(Default::default());

        let result = agent.handle(task, ctx).await.unwrap();
        assert!(result.content.contains("Meeting invitation"));
    }

    #[tokio::test]
    async fn test_mail_agent_handle_different_tasks() {
        let agent = MailAgent::new();
        let ctx = AgentContext::new(Default::default());

        let tasks = vec![
            "Write a thank you email",
            "Draft a follow-up message",
            "Compose a meeting invitation",
        ];

        for task_desc in tasks {
            let task = Task::new(task_desc);
            let result = agent.handle(task, ctx.clone()).await;
            assert!(result.is_ok());
            assert!(result.unwrap().success);
        }
    }

    #[test]
    fn test_generate_static_email() {
        let agent = MailAgent::new();
        let email = agent.generate_static_email("Project update");

        assert!(email.contains("Subject"));
        assert!(email.contains("Project update"));
        assert!(email.contains("Dear Recipient"));
    }
}
