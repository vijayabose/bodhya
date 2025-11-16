/// Test case definition for MailAgent evaluation
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailTestCase {
    /// Unique identifier for this test case
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Email context (what is the situation?)
    pub context: String,

    /// Purpose of the email
    pub purpose: String,

    /// Expected characteristics
    pub validation: EmailValidation,

    /// Email type
    pub email_type: EmailType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailType {
    Formal,
    Informal,
    Professional,
    Customer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailValidation {
    /// Expected tone markers
    pub expected_tone: Vec<String>,

    /// Forbidden words/phrases (too casual, rude, etc.)
    pub forbidden_words: Vec<String>,

    /// Minimum length in words
    pub min_words: Option<usize>,

    /// Maximum length in words
    pub max_words: Option<usize>,

    /// Must include greeting
    pub must_have_greeting: bool,

    /// Must include closing
    pub must_have_closing: bool,

    /// Must include subject line
    pub must_have_subject: bool,
}

impl MailTestCase {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        context: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            context: context.into(),
            purpose: purpose.into(),
            validation: EmailValidation::default(),
            email_type: EmailType::Professional,
        }
    }

    pub fn with_email_type(mut self, email_type: EmailType) -> Self {
        self.email_type = email_type;
        self
    }

    pub fn with_validation(mut self, validation: EmailValidation) -> Self {
        self.validation = validation;
        self
    }
}

impl Default for EmailValidation {
    fn default() -> Self {
        Self {
            expected_tone: Vec::new(),
            forbidden_words: Vec::new(),
            min_words: None,
            max_words: None,
            must_have_greeting: true,
            must_have_closing: true,
            must_have_subject: true,
        }
    }
}

impl EmailValidation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expect_tone(mut self, tone: impl Into<String>) -> Self {
        self.expected_tone.push(tone.into());
        self
    }

    pub fn forbid_word(mut self, word: impl Into<String>) -> Self {
        self.forbidden_words.push(word.into());
        self
    }

    pub fn min_words(mut self, words: usize) -> Self {
        self.min_words = Some(words);
        self
    }

    pub fn max_words(mut self, words: usize) -> Self {
        self.max_words = Some(words);
        self
    }

    pub fn require_greeting(mut self) -> Self {
        self.must_have_greeting = true;
        self
    }

    pub fn require_closing(mut self) -> Self {
        self.must_have_closing = true;
        self
    }

    pub fn require_subject(mut self) -> Self {
        self.must_have_subject = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_case() {
        let case = MailTestCase::new(
            "intro",
            "Introduction Email",
            "Meeting new team",
            "Introduce yourself to the team",
        );

        assert_eq!(case.id, "intro");
        assert_eq!(case.name, "Introduction Email");
        assert!(case.validation.must_have_greeting);
    }

    #[test]
    fn test_validation_builder() {
        let validation = EmailValidation::new()
            .expect_tone("polite")
            .forbid_word("hey")
            .min_words(50)
            .max_words(200)
            .require_greeting()
            .require_closing()
            .require_subject();

        assert_eq!(validation.expected_tone.len(), 1);
        assert_eq!(validation.forbidden_words.len(), 1);
        assert_eq!(validation.min_words, Some(50));
        assert_eq!(validation.max_words, Some(200));
        assert!(validation.must_have_greeting);
        assert!(validation.must_have_closing);
        assert!(validation.must_have_subject);
    }

    #[test]
    fn test_email_types() {
        let formal = MailTestCase::new("t1", "T", "c", "p").with_email_type(EmailType::Formal);
        let informal = MailTestCase::new("t2", "T", "c", "p").with_email_type(EmailType::Informal);

        assert!(matches!(formal.email_type, EmailType::Formal));
        assert!(matches!(informal.email_type, EmailType::Informal));
    }
}
