/// Code validation using cargo commands
///
/// This module handles running cargo check, test, and clippy on generated code.
use std::process::{Command, Stdio};

/// Validation result for a single command
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code
    pub exit_code: Option<i32>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(success: bool, stdout: String, stderr: String, exit_code: Option<i32>) -> Self {
        Self {
            success,
            stdout,
            stderr,
            exit_code,
        }
    }

    /// Check if there are errors in the output
    pub fn has_errors(&self) -> bool {
        !self.success || self.stderr.contains("error:")
    }

    /// Get error messages from output
    pub fn get_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for line in self.stderr.lines() {
            if line.contains("error:") {
                errors.push(line.to_string());
            }
        }

        errors
    }
}

/// Code validator using cargo
pub struct CodeValidator {
    /// Working directory for cargo commands
    pub working_dir: std::path::PathBuf,
}

impl CodeValidator {
    /// Create a new validator for a given directory
    pub fn new(working_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }

    /// Run cargo check
    pub fn check(&self) -> ValidationResult {
        self.run_cargo_command(&["check"])
    }

    /// Run cargo test
    pub fn test(&self) -> ValidationResult {
        self.run_cargo_command(&["test", "--", "--nocapture"])
    }

    /// Run cargo clippy
    pub fn clippy(&self) -> ValidationResult {
        self.run_cargo_command(&["clippy", "--", "-D", "warnings"])
    }

    /// Run a cargo command and return the result
    fn run_cargo_command(&self, args: &[&str]) -> ValidationResult {
        let output = Command::new("cargo")
            .args(args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) => {
                let success = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();

                ValidationResult::new(success, stdout, stderr, exit_code)
            }
            Err(e) => {
                // Command failed to execute (e.g., cargo not found)
                ValidationResult::new(
                    false,
                    String::new(),
                    format!("Failed to execute cargo: {}", e),
                    None,
                )
            }
        }
    }
}

/// Summary of all validation results
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationSummary {
    /// Result of cargo check
    pub check: ValidationResult,
    /// Result of cargo test
    pub test: ValidationResult,
    /// Result of cargo clippy
    pub clippy: ValidationResult,
}

impl ValidationSummary {
    /// Check if all validations passed
    pub fn all_passed(&self) -> bool {
        self.check.success && self.test.success && self.clippy.success
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str("## Validation Summary\n\n");

        summary.push_str(&format!(
            "- **cargo check**: {}\n",
            if self.check.success {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        ));

        summary.push_str(&format!(
            "- **cargo test**: {}\n",
            if self.test.success {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        ));

        summary.push_str(&format!(
            "- **cargo clippy**: {}\n",
            if self.clippy.success {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        ));

        if !self.all_passed() {
            summary.push_str("\n### Errors Found:\n\n");

            if self.check.has_errors() {
                summary.push_str("**cargo check errors:**\n");
                for error in self.check.get_errors() {
                    summary.push_str(&format!("- {}\n", error));
                }
                summary.push('\n');
            }

            if self.test.has_errors() {
                summary.push_str("**cargo test errors:**\n");
                for error in self.test.get_errors() {
                    summary.push_str(&format!("- {}\n", error));
                }
                summary.push('\n');
            }

            if self.clippy.has_errors() {
                summary.push_str("**cargo clippy errors:**\n");
                for error in self.clippy.get_errors() {
                    summary.push_str(&format!("- {}\n", error));
                }
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_creation() {
        let result =
            ValidationResult::new(true, "Build successful".to_string(), String::new(), Some(0));

        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
        assert!(!result.has_errors());
    }

    #[test]
    fn test_validation_result_with_errors() {
        let result = ValidationResult::new(
            false,
            String::new(),
            "error: could not compile `example`".to_string(),
            Some(101),
        );

        assert!(!result.success);
        assert!(result.has_errors());
        let errors = result.get_errors();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("could not compile"));
    }

    #[test]
    fn test_get_errors_multiple() {
        let stderr = r#"
error: unused variable: `x`
  --> src/main.rs:5:9
   |
5  |     let x = 10;
   |         ^ help: if this is intentional, prefix it with an underscore: `_x`
   |
error: unused variable: `y`
  --> src/main.rs:6:9
"#;

        let result = ValidationResult::new(false, String::new(), stderr.to_string(), Some(1));

        let errors = result.get_errors();
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("unused variable: `x`"));
        assert!(errors[1].contains("unused variable: `y`"));
    }

    #[test]
    fn test_validation_summary_all_passed() {
        let check = ValidationResult::new(true, "OK".to_string(), String::new(), Some(0));
        let test = ValidationResult::new(true, "OK".to_string(), String::new(), Some(0));
        let clippy = ValidationResult::new(true, "OK".to_string(), String::new(), Some(0));

        let summary = ValidationSummary {
            check,
            test,
            clippy,
        };

        assert!(summary.all_passed());
        let text = summary.summary();
        assert!(text.contains("✓ PASS"));
        assert!(!text.contains("✗ FAIL"));
    }

    #[test]
    fn test_validation_summary_with_failures() {
        let check = ValidationResult::new(true, "OK".to_string(), String::new(), Some(0));
        let test = ValidationResult::new(
            false,
            String::new(),
            "error: test failed".to_string(),
            Some(101),
        );
        let clippy = ValidationResult::new(true, "OK".to_string(), String::new(), Some(0));

        let summary = ValidationSummary {
            check,
            test,
            clippy,
        };

        assert!(!summary.all_passed());
        let text = summary.summary();
        assert!(text.contains("✗ FAIL"));
        assert!(text.contains("cargo test errors"));
    }

    #[test]
    fn test_validator_creation() {
        let validator = CodeValidator::new("/tmp/test_project");
        assert_eq!(
            validator.working_dir,
            std::path::PathBuf::from("/tmp/test_project")
        );
    }

    #[test]
    fn test_validation_result_no_errors_when_success() {
        let result =
            ValidationResult::new(true, "Build successful".to_string(), String::new(), Some(0));
        assert!(!result.has_errors());
        assert_eq!(result.get_errors().len(), 0);
    }

    #[test]
    fn test_summary_format() {
        let check = ValidationResult::new(true, "".to_string(), "".to_string(), Some(0));
        let test = ValidationResult::new(
            false,
            "".to_string(),
            "error: test failed".to_string(),
            Some(1),
        );
        let clippy = ValidationResult::new(true, "".to_string(), "".to_string(), Some(0));

        let summary = ValidationSummary {
            check,
            test,
            clippy,
        };
        let text = summary.summary();

        assert!(text.contains("## Validation Summary"));
        assert!(text.contains("cargo check"));
        assert!(text.contains("cargo test"));
        assert!(text.contains("cargo clippy"));
        assert!(text.contains("Errors Found"));
    }
}
