/// Quality scoring for MailAgent outputs
use crate::test_case::{EmailType, MailTestCase};
use serde::{Deserialize, Serialize};

/// Maximum rating possible (5-star system)
pub const MAX_RATING: f64 = 5.0;

/// Email quality rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRating {
    /// Tone score (0-1.5): Is the tone appropriate?
    pub tone: f64,

    /// Clarity score (0-1.5): Is it clear and well-structured?
    pub clarity: f64,

    /// Length score (0-1.0): Is the length appropriate?
    pub length: f64,

    /// Completeness score (0-1.0): Has all required elements?
    pub completeness: f64,

    /// Total rating (0-5.0)
    pub total: f64,

    /// Detailed feedback
    pub feedback: Vec<String>,
}

impl EmailRating {
    pub fn new(tone: f64, clarity: f64, length: f64, completeness: f64) -> Self {
        let total = tone + clarity + length + completeness;
        Self {
            tone,
            clarity,
            length,
            completeness,
            total,
            feedback: Vec::new(),
        }
    }

    pub fn with_feedback(mut self, feedback: Vec<String>) -> Self {
        self.feedback = feedback;
        self
    }

    pub fn add_feedback(&mut self, message: impl Into<String>) {
        self.feedback.push(message.into());
    }

    pub fn is_passing(&self) -> bool {
        self.total >= 4.5
    }
}

/// Score an email output
pub struct EmailScorer;

impl EmailScorer {
    /// Score a generated email output
    pub fn score(test_case: &MailTestCase, output: &str) -> EmailRating {
        let mut feedback = Vec::new();

        // 1. Tone (0-1.5 points)
        let tone = Self::score_tone(test_case, output, &mut feedback);

        // 2. Clarity (0-1.5 points)
        let clarity = Self::score_clarity(output, &mut feedback);

        // 3. Length (0-1.0 points)
        let length = Self::score_length(test_case, output, &mut feedback);

        // 4. Completeness (0-1.0 points)
        let completeness = Self::score_completeness(test_case, output, &mut feedback);

        EmailRating::new(tone, clarity, length, completeness).with_feedback(feedback)
    }

    fn score_tone(test_case: &MailTestCase, output: &str, feedback: &mut Vec<String>) -> f64 {
        let validation = &test_case.validation;
        let mut score: f64 = 0.0;
        let max_tone = 1.5;

        let output_lower = output.to_lowercase();

        // Check for forbidden words (severe penalty)
        let forbidden_found = validation
            .forbidden_words
            .iter()
            .filter(|w| output_lower.contains(&w.to_lowercase()))
            .count();

        if forbidden_found > 0 {
            score -= 0.5;
            feedback.push(format!(
                "✗ {} forbidden words/phrases found",
                forbidden_found
            ));
        } else if !validation.forbidden_words.is_empty() {
            score += 0.5;
            feedback.push("✓ No forbidden words found".to_string());
        } else {
            score += 0.25;
        }

        // Check for expected tone markers
        if !validation.expected_tone.is_empty() {
            let found = validation
                .expected_tone
                .iter()
                .filter(|t| output_lower.contains(&t.to_lowercase()))
                .count();

            let tone_score = (found as f64 / validation.expected_tone.len() as f64) * 0.5;
            score += tone_score;

            if found == validation.expected_tone.len() {
                feedback.push("✓ All expected tone markers found".to_string());
            } else {
                feedback.push(format!(
                    "⚠ Only {}/{} expected tone markers found",
                    found,
                    validation.expected_tone.len()
                ));
            }
        } else {
            score += 0.25;
        }

        // Type-specific tone check
        match test_case.email_type {
            EmailType::Formal | EmailType::Professional => {
                // Check for formal greetings
                if output_lower.contains("dear") || output_lower.contains("hello") {
                    score += 0.25;
                    feedback.push("✓ Appropriate formal greeting".to_string());
                }
                // Penalize overly casual
                if output_lower.contains("hey") || output_lower.contains("yo") {
                    score -= 0.25;
                    feedback.push("✗ Too casual for formal email".to_string());
                }
            }
            EmailType::Informal => {
                // Allow casual tone
                score += 0.25;
            }
            EmailType::Customer => {
                // Check for politeness
                if output_lower.contains("thank") || output_lower.contains("appreciate") {
                    score += 0.25;
                    feedback.push("✓ Polite and courteous".to_string());
                }
            }
        }

        score.min(max_tone).max(0.0)
    }

    fn score_clarity(output: &str, feedback: &mut Vec<String>) -> f64 {
        let mut score: f64 = 0.0;
        let max_clarity = 1.5;

        // Check for paragraph structure
        let paragraphs: Vec<_> = output
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();

        if paragraphs.len() >= 2 {
            score += 0.5;
            feedback.push(format!(
                "✓ Well-structured ({} paragraphs)",
                paragraphs.len()
            ));
        } else {
            feedback.push("⚠ Consider using multiple paragraphs for clarity".to_string());
        }

        // Check average sentence length (not too long)
        let sentences: Vec<_> = output
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .collect();
        if !sentences.is_empty() {
            let avg_words = sentences
                .iter()
                .map(|s| s.split_whitespace().count())
                .sum::<usize>()
                / sentences.len();

            if avg_words <= 25 {
                score += 0.5;
                feedback.push("✓ Sentences are clear and concise".to_string());
            } else {
                feedback.push("⚠ Some sentences may be too long".to_string());
            }
        }

        // Check for transition words/phrases
        let output_lower = output.to_lowercase();
        let transitions = [
            "however",
            "therefore",
            "additionally",
            "furthermore",
            "meanwhile",
        ];
        let has_transitions = transitions.iter().any(|t| output_lower.contains(t));

        if has_transitions {
            score += 0.25;
            feedback.push("✓ Uses transition words for flow".to_string());
        }

        // Penalize excessive exclamation marks
        let exclaim_count = output.matches('!').count();
        if exclaim_count > 2 {
            score -= 0.25;
            feedback.push("⚠ Too many exclamation marks".to_string());
        } else {
            score += 0.25;
        }

        score.min(max_clarity).max(0.0)
    }

    fn score_length(test_case: &MailTestCase, output: &str, feedback: &mut Vec<String>) -> f64 {
        let validation = &test_case.validation;
        let mut score: f64 = 0.0;
        let max_length = 1.0;

        let word_count = output.split_whitespace().count();

        let length_ok = match (validation.min_words, validation.max_words) {
            (Some(min), Some(max)) => word_count >= min && word_count <= max,
            (Some(min), None) => word_count >= min,
            (None, Some(max)) => word_count <= max,
            (None, None) => true,
        };

        if length_ok {
            score += 1.0;
            feedback.push(format!("✓ Appropriate length ({} words)", word_count));
        } else {
            if let Some(min) = validation.min_words {
                if word_count < min {
                    feedback.push(format!("✗ Too short ({} words, min {})", word_count, min));
                }
            }
            if let Some(max) = validation.max_words {
                if word_count > max {
                    feedback.push(format!("✗ Too long ({} words, max {})", word_count, max));
                }
            }
        }

        score.min(max_length).max(0.0)
    }

    fn score_completeness(
        test_case: &MailTestCase,
        output: &str,
        feedback: &mut Vec<String>,
    ) -> f64 {
        let validation = &test_case.validation;
        let mut score: f64 = 0.0;
        let max_complete = 1.0;

        let output_lower = output.to_lowercase();

        // Check for subject line
        if validation.must_have_subject {
            if output_lower.contains("subject:") {
                score += 0.35;
                feedback.push("✓ Contains subject line".to_string());
            } else {
                feedback.push("✗ Missing subject line".to_string());
            }
        } else {
            score += 0.35;
        }

        // Check for greeting
        if validation.must_have_greeting {
            let greetings = ["dear", "hello", "hi", "greetings"];
            let has_greeting = greetings.iter().any(|g| output_lower.contains(g));

            if has_greeting {
                score += 0.35;
                feedback.push("✓ Contains greeting".to_string());
            } else {
                feedback.push("✗ Missing greeting".to_string());
            }
        } else {
            score += 0.35;
        }

        // Check for closing
        if validation.must_have_closing {
            let closings = ["regards", "sincerely", "best", "thanks", "cheers"];
            let has_closing = closings.iter().any(|c| output_lower.contains(c));

            if has_closing {
                score += 0.3;
                feedback.push("✓ Contains closing".to_string());
            } else {
                feedback.push("✗ Missing closing".to_string());
            }
        } else {
            score += 0.3;
        }

        score.min(max_complete).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_case::EmailValidation;

    #[test]
    fn test_email_rating_creation() {
        let rating = EmailRating::new(1.2, 1.3, 0.9, 0.8);
        assert_eq!(rating.tone, 1.2);
        assert_eq!(rating.clarity, 1.3);
        assert_eq!(rating.length, 0.9);
        assert_eq!(rating.completeness, 0.8);
        assert_eq!(rating.total, 4.2);
        assert!(!rating.is_passing());
    }

    #[test]
    fn test_email_rating_passing() {
        let rating = EmailRating::new(1.4, 1.4, 0.9, 0.9);
        assert_eq!(rating.total, 4.6);
        assert!(rating.is_passing());
    }

    #[test]
    fn test_score_simple_email() {
        let test_case = MailTestCase::new("test", "Test Email", "context", "purpose")
            .with_validation(
                EmailValidation::new()
                    .require_greeting()
                    .require_closing()
                    .require_subject(),
            );

        let email = r#"Subject: Test Email

Dear Team,

This is a test email.

Best regards"#;

        let rating = EmailScorer::score(&test_case, email);
        assert!(rating.total > 0.0);
        assert!(rating.completeness > 0.0);
    }

    #[test]
    fn test_forbidden_words_penalty() {
        let test_case = MailTestCase::new("test", "Test", "c", "p")
            .with_validation(EmailValidation::new().forbid_word("urgent!!!"));

        let bad_email = "Subject: URGENT!!! This is urgent!!!";
        let good_email = "Subject: Important Notice";

        let bad_rating = EmailScorer::score(&test_case, bad_email);
        let good_rating = EmailScorer::score(&test_case, good_email);

        assert!(good_rating.tone > bad_rating.tone);
    }

    #[test]
    fn test_length_scoring() {
        let test_case = MailTestCase::new("test", "Test", "c", "p")
            .with_validation(EmailValidation::new().min_words(10).max_words(20));

        let short_email = "Hi there";
        let good_email =
            "This is a test email with about fifteen words in total for testing purposes.";
        let long_email = format!("{} ", good_email).repeat(10);

        let short_rating = EmailScorer::score(&test_case, short_email);
        let good_rating = EmailScorer::score(&test_case, good_email);
        let long_rating = EmailScorer::score(&test_case, &long_email);

        assert_eq!(short_rating.length, 0.0);
        assert_eq!(good_rating.length, 1.0);
        assert_eq!(long_rating.length, 0.0);
    }
}
