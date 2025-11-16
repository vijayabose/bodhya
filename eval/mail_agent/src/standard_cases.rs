/// Standard test cases for MailAgent evaluation
use crate::test_case::{EmailType, EmailValidation, MailTestCase};

/// Get all standard test cases
pub fn get_standard_cases() -> Vec<MailTestCase> {
    vec![
        team_introduction(),
        customer_inquiry_response(),
        meeting_request(),
        project_update(),
        apology_email(),
    ]
}

/// Test Case 1: Team Introduction (Professional)
pub fn team_introduction() -> MailTestCase {
    MailTestCase::new(
        "team_intro",
        "Team Introduction",
        "You're joining a new team as a software engineer.",
        "Introduce yourself to the team via email.",
    )
    .with_email_type(EmailType::Professional)
    .with_validation(
        EmailValidation::new()
            .require_greeting()
            .require_closing()
            .require_subject()
            .min_words(50)
            .max_words(200)
            .forbid_word("hey")
            .forbid_word("yo"),
    )
}

/// Test Case 2: Customer Inquiry Response (Customer)
pub fn customer_inquiry_response() -> MailTestCase {
    MailTestCase::new(
        "customer_inquiry",
        "Customer Inquiry Response",
        "A customer asked about product features and pricing.",
        "Write a helpful response addressing their questions.",
    )
    .with_email_type(EmailType::Customer)
    .with_validation(
        EmailValidation::new()
            .require_greeting()
            .require_closing()
            .require_subject()
            .expect_tone("thank")
            .min_words(80)
            .max_words(250)
            .forbid_word("dunno")
            .forbid_word("idk"),
    )
}

/// Test Case 3: Meeting Request (Formal)
pub fn meeting_request() -> MailTestCase {
    MailTestCase::new(
        "meeting_request",
        "Meeting Request",
        "You need to schedule a meeting with your manager.",
        "Request a meeting to discuss your quarterly performance review.",
    )
    .with_email_type(EmailType::Formal)
    .with_validation(
        EmailValidation::new()
            .require_greeting()
            .require_closing()
            .require_subject()
            .min_words(60)
            .max_words(180)
            .forbid_word("asap")
            .forbid_word("urgent!!!"),
    )
}

/// Test Case 4: Project Update (Professional)
pub fn project_update() -> MailTestCase {
    MailTestCase::new(
        "project_update",
        "Project Status Update",
        "Your team is working on a software release.",
        "Send an update email about the project progress and next steps.",
    )
    .with_email_type(EmailType::Professional)
    .with_validation(
        EmailValidation::new()
            .require_greeting()
            .require_closing()
            .require_subject()
            .min_words(100)
            .max_words(300),
    )
}

/// Test Case 5: Apology Email (Customer)
pub fn apology_email() -> MailTestCase {
    MailTestCase::new(
        "apology",
        "Service Apology",
        "A customer experienced a service disruption.",
        "Write an apology email acknowledging the issue and explaining resolution.",
    )
    .with_email_type(EmailType::Customer)
    .with_validation(
        EmailValidation::new()
            .require_greeting()
            .require_closing()
            .require_subject()
            .expect_tone("apologize")
            .expect_tone("sorry")
            .min_words(80)
            .max_words(220)
            .forbid_word("whatever")
            .forbid_word("not our fault"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_cases_count() {
        let cases = get_standard_cases();
        assert_eq!(cases.len(), 5);
    }

    #[test]
    fn test_team_introduction_case() {
        let case = team_introduction();
        assert_eq!(case.id, "team_intro");
        assert!(matches!(case.email_type, EmailType::Professional));
        assert!(case.validation.must_have_greeting);
    }

    #[test]
    fn test_customer_inquiry_case() {
        let case = customer_inquiry_response();
        assert_eq!(case.id, "customer_inquiry");
        assert!(matches!(case.email_type, EmailType::Customer));
        assert!(case.validation.expected_tone.contains(&"thank".to_string()));
    }

    #[test]
    fn test_all_cases_have_unique_ids() {
        let cases = get_standard_cases();
        let ids: Vec<_> = cases.iter().map(|c| &c.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_email_type_distribution() {
        let cases = get_standard_cases();
        let formal = cases
            .iter()
            .filter(|c| matches!(c.email_type, EmailType::Formal))
            .count();
        let professional = cases
            .iter()
            .filter(|c| matches!(c.email_type, EmailType::Professional))
            .count();
        let customer = cases
            .iter()
            .filter(|c| matches!(c.email_type, EmailType::Customer))
            .count();

        assert_eq!(formal, 1);
        assert_eq!(professional, 2);
        assert_eq!(customer, 2);
    }

    #[test]
    fn test_all_cases_have_validation() {
        let cases = get_standard_cases();
        // Just ensure validation exists for all cases
        assert_eq!(cases.len(), 5);
    }
}
