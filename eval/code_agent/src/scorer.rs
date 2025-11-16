/// Quality scoring for CodeAgent outputs
use crate::test_case::{CodeTestCase, Difficulty};
use serde::{Deserialize, Serialize};

/// Maximum score possible
pub const MAX_SCORE: f64 = 100.0;

/// Quality score breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Correctness score (0-40): Does it meet requirements?
    pub correctness: f64,

    /// Style score (0-30): Is it idiomatic and clean?
    pub style: f64,

    /// Coverage score (0-30): Are edge cases handled?
    pub coverage: f64,

    /// Total score (0-100)
    pub total: f64,

    /// Detailed feedback
    pub feedback: Vec<String>,
}

impl QualityScore {
    pub fn new(correctness: f64, style: f64, coverage: f64) -> Self {
        let total = correctness + style + coverage;
        Self {
            correctness,
            style,
            coverage,
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
        self.total >= 85.0
    }
}

/// Score a code output against a test case
pub struct CodeScorer;

impl CodeScorer {
    /// Score a generated code output
    pub fn score(test_case: &CodeTestCase, output: &str) -> QualityScore {
        let mut feedback = Vec::new();

        // 1. Correctness (0-40 points)
        let correctness = Self::score_correctness(test_case, output, &mut feedback);

        // 2. Style (0-30 points)
        let style = Self::score_style(output, &mut feedback);

        // 3. Coverage (0-30 points)
        let coverage = Self::score_coverage(test_case, output, &mut feedback);

        QualityScore::new(correctness, style, coverage).with_feedback(feedback)
    }

    fn score_correctness(
        test_case: &CodeTestCase,
        output: &str,
        feedback: &mut Vec<String>,
    ) -> f64 {
        let validation = &test_case.validation;
        let mut score = 0.0;
        let max_correctness = 40.0;

        // Expected patterns (15 points)
        if !validation.expected_patterns.is_empty() {
            let found = validation
                .expected_patterns
                .iter()
                .filter(|p| output.contains(p.as_str()))
                .count();
            let pattern_score = (found as f64 / validation.expected_patterns.len() as f64) * 15.0;
            score += pattern_score;

            if found == validation.expected_patterns.len() {
                feedback.push("✓ All expected patterns found".to_string());
            } else {
                feedback.push(format!(
                    "✗ Only {}/{} expected patterns found",
                    found,
                    validation.expected_patterns.len()
                ));
            }
        } else {
            // If no patterns specified, give partial credit
            score += 7.5;
        }

        // Forbidden patterns (10 points deduction if found)
        let forbidden_found = validation
            .forbidden_patterns
            .iter()
            .filter(|p| output.contains(p.as_str()))
            .count();
        if forbidden_found > 0 {
            score -= 10.0;
            feedback.push(format!("✗ {} forbidden patterns found", forbidden_found));
        } else if !validation.forbidden_patterns.is_empty() {
            score += 10.0;
            feedback.push("✓ No forbidden patterns found".to_string());
        } else {
            score += 5.0;
        }

        // Length check (15 points)
        let line_count = output.lines().count();
        let length_ok = match (validation.min_lines, validation.max_lines) {
            (Some(min), Some(max)) => line_count >= min && line_count <= max,
            (Some(min), None) => line_count >= min,
            (None, Some(max)) => line_count <= max,
            (None, None) => true,
        };

        if length_ok {
            score += 15.0;
            feedback.push(format!("✓ Length appropriate ({} lines)", line_count));
        } else {
            feedback.push(format!("✗ Length out of range ({} lines)", line_count));
        }

        score.min(max_correctness).max(0.0)
    }

    fn score_style(output: &str, feedback: &mut Vec<String>) -> f64 {
        let mut score: f64 = 0.0;
        let max_style = 30.0;

        // Check for common Rust style indicators

        // 1. Uses proper naming conventions (10 points)
        if output.contains("fn ") || output.contains("struct ") {
            score += 5.0;
            feedback.push("✓ Contains function/struct definitions".to_string());
        }

        // Snake case for functions (heuristic)
        if output.contains("fn ") && !output.contains("fn ALLCAPS") {
            score += 5.0;
        }

        // 2. Has comments or documentation (10 points)
        let has_docs = output.contains("///") || output.contains("//!");
        let has_comments = output.contains("//");

        if has_docs {
            score += 10.0;
            feedback.push("✓ Contains documentation comments".to_string());
        } else if has_comments {
            score += 5.0;
            feedback.push("✓ Contains comments".to_string());
        }

        // 3. Proper formatting (10 points)
        // Check for consistent indentation (heuristic: no tabs, proper spaces)
        if !output.contains('\t') {
            score += 5.0;
        }

        // Not overly long lines (heuristic)
        let long_lines = output.lines().filter(|l| l.len() > 100).count();
        if long_lines < output.lines().count() / 10 {
            score += 5.0;
        } else {
            feedback.push(format!("⚠ {} lines exceed 100 characters", long_lines));
        }

        score.min(max_style).max(0.0)
    }

    fn score_coverage(test_case: &CodeTestCase, output: &str, feedback: &mut Vec<String>) -> f64 {
        let mut score: f64 = 0.0;
        let max_coverage = 30.0;

        // Adjust for difficulty
        let difficulty_factor = match test_case.difficulty {
            Difficulty::Easy => 1.2, // Easier to get full marks
            Difficulty::Medium => 1.0,
            Difficulty::Hard => 0.8, // Harder to get full marks
        };

        // 1. Error handling (15 points)
        let has_error_handling = output.contains("Result")
            || output.contains("Option")
            || output.contains("?")
            || output.contains("unwrap")
            || output.contains("expect");

        if has_error_handling {
            score += 15.0 * difficulty_factor;
            feedback.push("✓ Contains error handling".to_string());
        } else {
            feedback.push("⚠ No error handling detected".to_string());
        }

        // 2. Tests or validation (15 points)
        let has_tests = output.contains("#[test]")
            || output.contains("#[cfg(test)]")
            || output.contains("assert");

        if has_tests {
            score += 15.0 * difficulty_factor;
            feedback.push("✓ Contains tests or assertions".to_string());
        } else {
            feedback.push("⚠ No tests detected".to_string());
        }

        score.min(max_coverage).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_case::ValidationCriteria;

    #[test]
    fn test_quality_score_creation() {
        let score = QualityScore::new(35.0, 25.0, 28.0);
        assert_eq!(score.correctness, 35.0);
        assert_eq!(score.style, 25.0);
        assert_eq!(score.coverage, 28.0);
        assert_eq!(score.total, 88.0);
        assert!(score.is_passing());
    }

    #[test]
    fn test_quality_score_failing() {
        let score = QualityScore::new(30.0, 20.0, 20.0);
        assert_eq!(score.total, 70.0);
        assert!(!score.is_passing());
    }

    #[test]
    fn test_score_simple_hello_world() {
        let test_case = CodeTestCase::new("hello", "Hello World", "Write hello world")
            .with_validation(
                ValidationCriteria::new()
                    .expect_pattern("fn main")
                    .expect_pattern("println!"),
            );

        let output = r#"
fn main() {
    println!("Hello, World!");
}
"#;

        let score = CodeScorer::score(&test_case, output);
        assert!(score.correctness > 0.0);
        assert!(score.total > 0.0);
    }

    #[test]
    fn test_score_with_forbidden_patterns() {
        let test_case = CodeTestCase::new("safe_code", "Safe Code", "Write safe code")
            .with_validation(ValidationCriteria::new().forbid_pattern("unsafe"));

        let unsafe_output = "unsafe { let x = 5; }";
        let score = CodeScorer::score(&test_case, unsafe_output);
        assert!(score.correctness < 40.0); // Penalty applied

        let safe_output = "fn safe() { let x = 5; }";
        let score2 = CodeScorer::score(&test_case, safe_output);
        assert!(score2.correctness > score.correctness);
    }

    #[test]
    fn test_score_style_with_comments() {
        let test_case = CodeTestCase::new("test", "Test", "desc");

        let with_docs = r#"
/// This is a documented function
fn main() {
    println!("Hello");
}
"#;

        let without_docs = r#"
fn main() {
    println!("Hello");
}
"#;

        let score_with = CodeScorer::score(&test_case, with_docs);
        let score_without = CodeScorer::score(&test_case, without_docs);

        assert!(score_with.style > score_without.style);
    }

    #[test]
    fn test_difficulty_affects_coverage() {
        let easy = CodeTestCase::new("e", "Easy", "desc").with_difficulty(Difficulty::Easy);
        let hard = CodeTestCase::new("h", "Hard", "desc").with_difficulty(Difficulty::Hard);

        let output = r#"
fn main() -> Result<(), Error> {
    assert!(true);
    Ok(())
}
"#;

        let score_easy = CodeScorer::score(&easy, output);
        let score_hard = CodeScorer::score(&hard, output);

        // Easy tasks get bonus on coverage
        assert!(score_easy.coverage >= score_hard.coverage);
    }
}
