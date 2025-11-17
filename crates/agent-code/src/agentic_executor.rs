/// Agentic Executor - Error Analysis and Retry Logic
///
/// This module implements the observe-retry-fix workflow for agentic code generation.
/// When tests fail or compilation errors occur, the executor analyzes the errors
/// and generates refinements to fix them.
use bodhya_core::Result;
use bodhya_model_registry::ModelRegistry;
use std::sync::Arc;

use crate::impl_gen::ImplCode;
use crate::planner::CodePlan;
use crate::tdd::TestCode;
use crate::tools::{CodeAgentTools, CommandOutput};

/// Error analysis result
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    /// Error category (compilation, test_failure, runtime, etc.)
    pub category: ErrorCategory,
    /// Specific error messages extracted
    pub messages: Vec<String>,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Root cause analysis
    pub root_cause: Option<String>,
}

/// Categories of errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Compilation errors (syntax, type errors, etc.)
    Compilation,
    /// Test failures (assertions failed)
    TestFailure,
    /// Runtime errors (panics, exceptions)
    Runtime,
    /// Unknown error type
    Unknown,
}

/// Analyzes error output and extracts structured information
pub struct ErrorAnalyzer {
    #[allow(dead_code)]
    registry: Arc<ModelRegistry>,
}

impl ErrorAnalyzer {
    /// Create a new error analyzer
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        Ok(Self { registry })
    }

    /// Analyze command output and extract error information
    pub async fn analyze(&self, output: &CommandOutput) -> Result<ErrorAnalysis> {
        // Combine stderr and stdout for analysis
        let error_text = if !output.stderr.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        };

        // Determine error category
        let category = self.categorize_error(error_text);

        // Extract error messages using simple parsing
        let messages = self.extract_error_messages(error_text);

        // For now, use simple heuristics for suggestions
        // In a real implementation, this would use an LLM
        let suggestions = self.generate_suggestions(&category, &messages);

        let root_cause = self.identify_root_cause(error_text, &category);

        Ok(ErrorAnalysis {
            category,
            messages,
            suggestions,
            root_cause,
        })
    }

    /// Categorize error based on error text
    fn categorize_error(&self, error_text: &str) -> ErrorCategory {
        let text_lower = error_text.to_lowercase();

        if text_lower.contains("error[e") || text_lower.contains("could not compile") {
            ErrorCategory::Compilation
        } else if text_lower.contains("test result: failed")
            || text_lower.contains("assertion")
            || text_lower.contains("expected")
        {
            ErrorCategory::TestFailure
        } else if text_lower.contains("panic") || text_lower.contains("thread") {
            ErrorCategory::Runtime
        } else {
            ErrorCategory::Unknown
        }
    }

    /// Extract specific error messages from output
    fn extract_error_messages(&self, error_text: &str) -> Vec<String> {
        let mut messages = Vec::new();

        for line in error_text.lines() {
            let trimmed = line.trim();
            // Check if line contains any error indicators
            if trimmed.starts_with("error[E")
                || trimmed.starts_with("error:")
                || line.contains("assertion")
                || line.contains("expected")
                || line.contains("panicked at")
            {
                messages.push(trimmed.to_string());
            }
        }

        // If no specific errors found, take first 5 non-empty lines
        if messages.is_empty() {
            messages = error_text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(5)
                .map(|l| l.trim().to_string())
                .collect();
        }

        messages
    }

    /// Generate fix suggestions based on error category and messages
    fn generate_suggestions(&self, category: &ErrorCategory, messages: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();

        match category {
            ErrorCategory::Compilation => {
                suggestions.push("Check for syntax errors and type mismatches".to_string());
                suggestions.push("Ensure all required imports are present".to_string());
                suggestions.push("Verify function signatures match their usage".to_string());
            }
            ErrorCategory::TestFailure => {
                suggestions.push("Review test assertions and expected values".to_string());
                suggestions
                    .push("Check if implementation logic matches test expectations".to_string());
                suggestions.push("Verify edge cases are handled correctly".to_string());
            }
            ErrorCategory::Runtime => {
                suggestions.push("Add proper error handling for potential panics".to_string());
                suggestions.push("Check for division by zero or array bounds".to_string());
            }
            ErrorCategory::Unknown => {
                suggestions.push("Review the full error output for clues".to_string());
            }
        }

        // Add message-specific suggestions
        for message in messages {
            if message.contains("cannot find") {
                suggestions.push(format!("Add missing import or definition: {}", message));
            } else if message.contains("mismatched types") {
                suggestions.push(format!("Fix type mismatch: {}", message));
            }
        }

        suggestions.truncate(5); // Limit to 5 suggestions
        suggestions
    }

    /// Identify root cause from error text
    fn identify_root_cause(&self, error_text: &str, category: &ErrorCategory) -> Option<String> {
        match category {
            ErrorCategory::Compilation => {
                if error_text.contains("cannot find") {
                    Some("Missing definition or import".to_string())
                } else if error_text.contains("mismatched types") {
                    Some("Type incompatibility".to_string())
                } else {
                    Some("Compilation error in generated code".to_string())
                }
            }
            ErrorCategory::TestFailure => {
                if error_text.contains("assertion") {
                    Some(
                        "Test assertion failed - implementation doesn't match expected behavior"
                            .to_string(),
                    )
                } else {
                    Some("Test execution failed".to_string())
                }
            }
            ErrorCategory::Runtime => Some("Runtime panic or exception".to_string()),
            ErrorCategory::Unknown => None,
        }
    }
}

/// Code refiner - generates fixed code based on error analysis
pub struct CodeRefiner {
    _registry: Arc<ModelRegistry>,
}

impl CodeRefiner {
    /// Create a new code refiner
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        Ok(Self {
            _registry: registry,
        })
    }

    /// Generate refined implementation based on error analysis
    pub async fn refine(
        &self,
        original_impl: &ImplCode,
        _test_code: &TestCode,
        error_analysis: &ErrorAnalysis,
        _plan: &CodePlan,
    ) -> Result<ImplCode> {
        // For now, return a simple refinement
        // In a real implementation, this would use an LLM to generate fixes

        let mut refined_code = original_impl.code.clone();

        // Apply simple heuristic fixes based on error category
        match error_analysis.category {
            ErrorCategory::Compilation => {
                // Add common imports if missing
                if error_analysis
                    .messages
                    .iter()
                    .any(|m| m.contains("cannot find"))
                    && !refined_code.contains("use std::")
                {
                    refined_code =
                        format!("use std::fmt;\nuse std::error::Error;\n\n{}", refined_code);
                }
            }
            ErrorCategory::TestFailure => {
                // This would need LLM analysis to fix logic errors
                // For now, just return original
            }
            _ => {}
        }

        Ok(ImplCode {
            code: refined_code,

            loc: original_impl.loc,
        })
    }
}

/// Agentic executor - orchestrates the observe-retry-fix loop
pub struct AgenticExecutor {
    analyzer: ErrorAnalyzer,
    refiner: CodeRefiner,
    max_iterations: usize,
}

impl AgenticExecutor {
    /// Create a new agentic executor
    pub fn new(registry: Arc<ModelRegistry>, max_iterations: usize) -> Result<Self> {
        Ok(Self {
            analyzer: ErrorAnalyzer::new(Arc::clone(&registry))?,
            refiner: CodeRefiner::new(registry)?,
            max_iterations,
        })
    }

    /// Execute the observe-retry-fix loop
    ///
    /// Returns the final implementation and a summary of the execution
    pub async fn execute_with_retry(
        &self,
        initial_impl: ImplCode,
        test_code: &TestCode,
        plan: &CodePlan,
        tools: &CodeAgentTools,
        _test_path: &str,
        impl_path: &str,
    ) -> Result<(ImplCode, ExecutionSummary)> {
        let mut current_impl = initial_impl;
        let mut iteration = 0;
        let mut attempts = Vec::new();

        while iteration < self.max_iterations {
            iteration += 1;

            // Write current implementation
            tools.write_file(impl_path, &current_impl.code).await?;

            // Run tests
            let test_result = tools.run_cargo("test", &[]).await?;

            let attempt = AttemptSummary {
                iteration,
                success: test_result.success,
                error_category: if test_result.success {
                    None
                } else {
                    Some(self.analyzer.categorize_error(&test_result.stderr))
                },
                error_count: if test_result.success {
                    0
                } else {
                    self.analyzer
                        .extract_error_messages(&test_result.stderr)
                        .len()
                },
            };

            attempts.push(attempt);

            if test_result.success {
                // Success! Return the working implementation
                return Ok((
                    current_impl,
                    ExecutionSummary {
                        total_iterations: iteration,
                        successful: true,
                        attempts,
                    },
                ));
            }

            // Analyze errors
            let error_analysis = self.analyzer.analyze(&test_result).await?;

            // Check if we've reached max iterations
            if iteration >= self.max_iterations {
                return Ok((
                    current_impl,
                    ExecutionSummary {
                        total_iterations: iteration,
                        successful: false,
                        attempts,
                    },
                ));
            }

            // Refine the implementation
            current_impl = self
                .refiner
                .refine(&current_impl, test_code, &error_analysis, plan)
                .await?;
        }

        // Shouldn't reach here, but just in case
        Ok((
            current_impl,
            ExecutionSummary {
                total_iterations: iteration,
                successful: false,
                attempts,
            },
        ))
    }
}

/// Summary of a single attempt in the retry loop
#[derive(Debug, Clone)]
pub struct AttemptSummary {
    /// Iteration number
    pub iteration: usize,
    /// Whether this attempt succeeded
    pub success: bool,
    /// Error category (if failed)
    pub error_category: Option<ErrorCategory>,
    /// Number of errors encountered
    pub error_count: usize,
}

/// Summary of the entire execution
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    /// Total number of iterations
    pub total_iterations: usize,
    /// Whether the execution ultimately succeeded
    pub successful: bool,
    /// Details of each attempt
    pub attempts: Vec<AttemptSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_equality() {
        // Test error category enum equality
        assert_eq!(ErrorCategory::Compilation, ErrorCategory::Compilation);
        assert_eq!(ErrorCategory::TestFailure, ErrorCategory::TestFailure);
        assert_eq!(ErrorCategory::Runtime, ErrorCategory::Runtime);
        assert_eq!(ErrorCategory::Unknown, ErrorCategory::Unknown);
    }

    #[test]
    fn test_error_analysis_structure() {
        let analysis = ErrorAnalysis {
            category: ErrorCategory::Compilation,
            messages: vec!["error: cannot find function".to_string()],
            suggestions: vec!["Add missing import".to_string()],
            root_cause: Some("Missing definition".to_string()),
        };

        assert_eq!(analysis.category, ErrorCategory::Compilation);
        assert_eq!(analysis.messages.len(), 1);
        assert_eq!(analysis.suggestions.len(), 1);
        assert!(analysis.root_cause.is_some());
    }

    #[test]
    fn test_attempt_summary() {
        let attempt = AttemptSummary {
            iteration: 1,
            success: false,
            error_category: Some(ErrorCategory::Compilation),
            error_count: 2,
        };

        assert_eq!(attempt.iteration, 1);
        assert!(!attempt.success);
        assert_eq!(attempt.error_category, Some(ErrorCategory::Compilation));
        assert_eq!(attempt.error_count, 2);
    }

    #[test]
    fn test_execution_summary() {
        let summary = ExecutionSummary {
            total_iterations: 3,
            successful: true,
            attempts: vec![
                AttemptSummary {
                    iteration: 1,
                    success: false,
                    error_category: Some(ErrorCategory::Compilation),
                    error_count: 2,
                },
                AttemptSummary {
                    iteration: 2,
                    success: false,
                    error_category: Some(ErrorCategory::TestFailure),
                    error_count: 1,
                },
                AttemptSummary {
                    iteration: 3,
                    success: true,
                    error_category: None,
                    error_count: 0,
                },
            ],
        };

        assert_eq!(summary.total_iterations, 3);
        assert!(summary.successful);
        assert_eq!(summary.attempts.len(), 3);
        assert!(summary.attempts[2].success);
    }
}
