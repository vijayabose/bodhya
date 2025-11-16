/// Test case definition for CodeAgent evaluation
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTestCase {
    /// Unique identifier for this test case
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Task description to give to CodeAgent
    pub description: String,

    /// Expected outputs or validation criteria
    pub validation: ValidationCriteria,

    /// Difficulty level (easy, medium, hard)
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCriteria {
    /// Must compile without errors
    pub must_compile: bool,

    /// Expected keywords/patterns in output
    pub expected_patterns: Vec<String>,

    /// Forbidden patterns (anti-patterns, security issues)
    pub forbidden_patterns: Vec<String>,

    /// Expected file types/extensions
    pub expected_files: Vec<String>,

    /// Minimum lines of code (rough heuristic)
    pub min_lines: Option<usize>,

    /// Maximum lines of code (avoid bloat)
    pub max_lines: Option<usize>,
}

impl CodeTestCase {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            validation: ValidationCriteria::default(),
            difficulty: Difficulty::Medium,
        }
    }

    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    pub fn with_validation(mut self, validation: ValidationCriteria) -> Self {
        self.validation = validation;
        self
    }
}

impl Default for ValidationCriteria {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationCriteria {
    pub fn new() -> Self {
        Self {
            must_compile: false,
            expected_patterns: Vec::new(),
            forbidden_patterns: Vec::new(),
            expected_files: Vec::new(),
            min_lines: None,
            max_lines: None,
        }
    }

    pub fn must_compile(mut self) -> Self {
        self.must_compile = true;
        self
    }

    pub fn expect_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.expected_patterns.push(pattern.into());
        self
    }

    pub fn forbid_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.forbidden_patterns.push(pattern.into());
        self
    }

    pub fn expect_file(mut self, file: impl Into<String>) -> Self {
        self.expected_files.push(file.into());
        self
    }

    pub fn min_lines(mut self, lines: usize) -> Self {
        self.min_lines = Some(lines);
        self
    }

    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_case() {
        let case = CodeTestCase::new(
            "hello_world",
            "Hello World",
            "Write a hello world program in Rust",
        );

        assert_eq!(case.id, "hello_world");
        assert_eq!(case.name, "Hello World");
        assert!(!case.validation.must_compile);
    }

    #[test]
    fn test_validation_builder() {
        let validation = ValidationCriteria::new()
            .must_compile()
            .expect_pattern("fn main")
            .expect_pattern("println!")
            .forbid_pattern("unsafe")
            .min_lines(5)
            .max_lines(20);

        assert!(validation.must_compile);
        assert_eq!(validation.expected_patterns.len(), 2);
        assert_eq!(validation.forbidden_patterns.len(), 1);
        assert_eq!(validation.min_lines, Some(5));
        assert_eq!(validation.max_lines, Some(20));
    }

    #[test]
    fn test_difficulty_levels() {
        let easy = CodeTestCase::new("test1", "Test", "desc").with_difficulty(Difficulty::Easy);
        let hard = CodeTestCase::new("test2", "Test", "desc").with_difficulty(Difficulty::Hard);

        assert!(matches!(easy.difficulty, Difficulty::Easy));
        assert!(matches!(hard.difficulty, Difficulty::Hard));
    }
}
