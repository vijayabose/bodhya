/// Email classification and policy checking (Future)
///
/// This module is a stub for future email classification and policy checking functionality.
/// In future versions, this will handle:
/// - Email categorization (formal/informal, business/personal)
/// - Policy compliance checking
/// - Sensitivity detection
/// - Content filtering
use crate::draft::EmailDraft;

/// Email category
#[derive(Clone, Debug, PartialEq)]
pub enum EmailCategory {
    /// Formal business email
    FormalBusiness,
    /// Informal business email
    InformalBusiness,
    /// Personal email
    Personal,
    /// Other/unknown
    Other,
}

/// Email classification result
#[derive(Clone, Debug, PartialEq)]
pub struct EmailClassification {
    /// Email category
    pub category: EmailCategory,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Policy compliance warnings (if any)
    pub warnings: Vec<String>,
}

impl EmailClassification {
    /// Create a new classification
    pub fn new(category: EmailCategory, confidence: f32) -> Self {
        Self {
            category,
            confidence,
            warnings: Vec::new(),
        }
    }

    /// Add a policy warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

/// Email classifier (stub for future implementation)
pub struct EmailClassifier;

impl EmailClassifier {
    /// Create a new email classifier
    pub fn new() -> Self {
        Self
    }

    /// Classify an email draft (stub implementation)
    ///
    /// This is a placeholder. Future implementation will use ML models
    /// or rule-based systems for classification.
    pub fn classify(&self, _draft: &EmailDraft) -> EmailClassification {
        // Stub: Always returns FormalBusiness with medium confidence
        EmailClassification::new(EmailCategory::FormalBusiness, 0.5)
    }

    /// Check policy compliance (stub implementation)
    ///
    /// Future implementation will check against configurable policies.
    pub fn check_policy(&self, _draft: &EmailDraft) -> Vec<String> {
        // Stub: No policy violations
        Vec::new()
    }
}

impl Default for EmailClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_classification_creation() {
        let classification = EmailClassification::new(EmailCategory::FormalBusiness, 0.9);

        assert_eq!(classification.category, EmailCategory::FormalBusiness);
        assert_eq!(classification.confidence, 0.9);
        assert_eq!(classification.warnings.len(), 0);
    }

    #[test]
    fn test_email_classification_add_warning() {
        let mut classification = EmailClassification::new(EmailCategory::Personal, 0.7);
        classification.add_warning("May contain sensitive information");
        classification.add_warning("Check recipient address");

        assert_eq!(classification.warnings.len(), 2);
        assert_eq!(
            classification.warnings[0],
            "May contain sensitive information"
        );
    }

    #[test]
    fn test_email_classifier_creation() {
        let classifier = EmailClassifier::new();
        let draft = EmailDraft::new("Test", "Body");

        let classification = classifier.classify(&draft);
        assert_eq!(classification.category, EmailCategory::FormalBusiness);
    }

    #[test]
    fn test_email_classifier_default() {
        let classifier = EmailClassifier;
        let draft = EmailDraft::new("Test", "Body");

        let classification = classifier.classify(&draft);
        assert_eq!(classification.confidence, 0.5);
    }

    #[test]
    fn test_email_classifier_check_policy() {
        let classifier = EmailClassifier::new();
        let draft = EmailDraft::new("Test", "Body");

        let warnings = classifier.check_policy(&draft);
        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_email_category_variants() {
        let categories = [
            EmailCategory::FormalBusiness,
            EmailCategory::InformalBusiness,
            EmailCategory::Personal,
            EmailCategory::Other,
        ];

        assert_eq!(categories.len(), 4);
    }

    #[test]
    fn test_classification_with_multiple_warnings() {
        let mut classification = EmailClassification::new(EmailCategory::Other, 0.3);

        for i in 1..=5 {
            classification.add_warning(format!("Warning {}", i));
        }

        assert_eq!(classification.warnings.len(), 5);
        assert_eq!(classification.warnings[4], "Warning 5");
    }
}
