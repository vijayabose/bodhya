/// Standard test cases for CodeAgent evaluation
use crate::test_case::{CodeTestCase, Difficulty, ValidationCriteria};

/// Get all standard test cases
pub fn get_standard_cases() -> Vec<CodeTestCase> {
    vec![
        hello_world_case(),
        fibonacci_case(),
        file_reader_case(),
        calculator_case(),
        hash_map_case(),
    ]
}

/// Test Case 1: Hello World (Easy)
pub fn hello_world_case() -> CodeTestCase {
    CodeTestCase::new(
        "hello_world",
        "Hello World Program",
        "Write a simple Rust program that prints 'Hello, World!' to the console",
    )
    .with_difficulty(Difficulty::Easy)
    .with_validation(
        ValidationCriteria::new()
            .expect_pattern("fn main")
            .expect_pattern("println!")
            .forbid_pattern("unsafe")
            .min_lines(3)
            .max_lines(20),
    )
}

/// Test Case 2: Fibonacci Sequence (Medium)
pub fn fibonacci_case() -> CodeTestCase {
    CodeTestCase::new(
        "fibonacci",
        "Fibonacci Calculator",
        "Write a Rust function that calculates the nth Fibonacci number. \
         Include error handling for invalid inputs and add unit tests.",
    )
    .with_difficulty(Difficulty::Medium)
    .with_validation(
        ValidationCriteria::new()
            .expect_pattern("fn fib")
            .expect_pattern("Result")
            .expect_pattern("#[test]")
            .forbid_pattern("panic")
            .forbid_pattern("unwrap")
            .min_lines(20)
            .max_lines(100),
    )
}

/// Test Case 3: File Reader (Medium)
pub fn file_reader_case() -> CodeTestCase {
    CodeTestCase::new(
        "file_reader",
        "File Content Reader",
        "Write a Rust function that reads a file and returns its contents as a String. \
         Handle file not found and permission errors gracefully. Include documentation.",
    )
    .with_difficulty(Difficulty::Medium)
    .with_validation(
        ValidationCriteria::new()
            .expect_pattern("fn read_file")
            .expect_pattern("Result")
            .expect_pattern("std::fs")
            .expect_pattern("///")
            .forbid_pattern("panic!")
            .min_lines(15)
            .max_lines(80),
    )
}

/// Test Case 4: Simple Calculator (Medium)
pub fn calculator_case() -> CodeTestCase {
    CodeTestCase::new(
        "calculator",
        "Basic Calculator",
        "Write a Rust calculator that can add, subtract, multiply, and divide two numbers. \
         Use an enum for operations. Include error handling for division by zero and tests.",
    )
    .with_difficulty(Difficulty::Medium)
    .with_validation(
        ValidationCriteria::new()
            .expect_pattern("enum")
            .expect_pattern("match")
            .expect_pattern("Result")
            .expect_pattern("#[test]")
            .forbid_pattern("panic")
            .min_lines(30)
            .max_lines(150),
    )
}

/// Test Case 5: HashMap Operations (Hard)
pub fn hash_map_case() -> CodeTestCase {
    CodeTestCase::new(
        "hashmap_ops",
        "HashMap Word Counter",
        "Write a Rust program that reads text and counts word frequencies using a HashMap. \
         Handle edge cases like empty input, punctuation, and case insensitivity. \
         Include comprehensive tests and documentation.",
    )
    .with_difficulty(Difficulty::Hard)
    .with_validation(
        ValidationCriteria::new()
            .expect_pattern("HashMap")
            .expect_pattern("fn count_words")
            .expect_pattern("Result")
            .expect_pattern("#[test]")
            .expect_pattern("///")
            .forbid_pattern("unwrap")
            .min_lines(40)
            .max_lines(200),
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
    fn test_hello_world_case() {
        let case = hello_world_case();
        assert_eq!(case.id, "hello_world");
        assert!(matches!(case.difficulty, Difficulty::Easy));
        assert!(!case.validation.must_compile);
        assert!(case
            .validation
            .expected_patterns
            .contains(&"fn main".to_string()));
    }

    #[test]
    fn test_fibonacci_case() {
        let case = fibonacci_case();
        assert_eq!(case.id, "fibonacci");
        assert!(matches!(case.difficulty, Difficulty::Medium));
        assert!(case
            .validation
            .expected_patterns
            .contains(&"Result".to_string()));
    }

    #[test]
    fn test_all_cases_have_unique_ids() {
        let cases = get_standard_cases();
        let ids: Vec<_> = cases.iter().map(|c| &c.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }

    #[test]
    fn test_difficulty_distribution() {
        let cases = get_standard_cases();
        let easy = cases
            .iter()
            .filter(|c| matches!(c.difficulty, Difficulty::Easy))
            .count();
        let medium = cases
            .iter()
            .filter(|c| matches!(c.difficulty, Difficulty::Medium))
            .count();
        let hard = cases
            .iter()
            .filter(|c| matches!(c.difficulty, Difficulty::Hard))
            .count();

        assert_eq!(easy, 1);
        assert_eq!(medium, 3);
        assert_eq!(hard, 1);
    }
}
