/// Agentic Executor - Error Analysis and Retry Logic
///
/// This module implements the observe-retry-fix workflow for agentic code generation.
/// When tests fail or compilation errors occur, the executor analyzes the errors
/// and generates refinements to fix them.
use bodhya_core::{EngagementMode, ModelRequest, ModelRole, Result};
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
    registry: Arc<ModelRegistry>,
}

impl ErrorAnalyzer {
    /// Create a new error analyzer
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        Ok(Self { registry })
    }

    /// Load the error analyzer prompt
    fn load_prompt(&self) -> Result<String> {
        // Use embedded prompt from file
        Ok(include_str!("../../../prompts/code/error_analyzer.txt").to_string())
    }

    /// Analyze command output and extract error information using LLM
    pub async fn analyze(&self, output: &CommandOutput) -> Result<ErrorAnalysis> {
        self.analyze_with_context(output, "", "", "").await
    }

    /// Analyze errors with full context (code, tests, plan)
    pub async fn analyze_with_context(
        &self,
        output: &CommandOutput,
        plan_context: &str,
        generated_code: &str,
        test_code: &str,
    ) -> Result<ErrorAnalysis> {
        // Combine stderr and stdout for analysis
        let error_text = if !output.stderr.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        };

        // Quick categorization for fallback
        let fallback_category = self.categorize_error(error_text);

        // Try LLM-based analysis first
        match self
            .llm_analyze(plan_context, generated_code, test_code, error_text)
            .await
        {
            Ok(analysis) => Ok(analysis),
            Err(e) => {
                tracing::warn!(
                    "LLM error analysis failed, falling back to heuristics: {}",
                    e
                );
                // Fall back to heuristic analysis
                self.heuristic_analyze(output, fallback_category).await
            }
        }
    }

    /// LLM-based error analysis
    async fn llm_analyze(
        &self,
        plan_context: &str,
        generated_code: &str,
        test_code: &str,
        error_output: &str,
    ) -> Result<ErrorAnalysis> {
        let prompt_template = self.load_prompt()?;

        // Fill in the template
        let prompt = prompt_template
            .replace("{plan_context}", plan_context)
            .replace("{generated_code}", generated_code)
            .replace("{test_code}", test_code)
            .replace("{error_output}", error_output);

        // Get planner model from registry for reasoning
        let model_info =
            self.registry
                .get_model(&ModelRole::Planner, "code", &EngagementMode::Minimum)?;

        // Create model request
        let request = ModelRequest::new(ModelRole::Planner, "code", prompt);

        // Call the model backend
        let backend = self.registry.get_backend(&model_info.id).ok_or_else(|| {
            bodhya_core::Error::Config(format!(
                "Backend '{}' not found for model '{}'",
                model_info.definition.backend, model_info.id
            ))
        })?;

        let response = backend.generate(request).await?;

        // Parse the LLM response to extract structured information
        self.parse_llm_response(&response.text)
    }

    /// Parse LLM response into ErrorAnalysis structure
    fn parse_llm_response(&self, response: &str) -> Result<ErrorAnalysis> {
        let mut category = ErrorCategory::Unknown;
        let mut messages = Vec::new();
        let mut suggestions = Vec::new();
        let mut root_cause = None;

        // Parse the response sections
        for line in response.lines() {
            let line_trimmed = line.trim();

            // Parse Error Category
            if line_trimmed.contains("COMPILATION") {
                category = ErrorCategory::Compilation;
            } else if line_trimmed.contains("TEST_FAILURE") {
                category = ErrorCategory::TestFailure;
            } else if line_trimmed.contains("RUNTIME") {
                category = ErrorCategory::Runtime;
            }

            // Extract root cause (look for "Root Cause Analysis" section)
            if line_trimmed.starts_with("##") && line_trimmed.contains("Root Cause") {
                // The next non-empty line is the root cause
                continue;
            } else if root_cause.is_none()
                && !line_trimmed.starts_with("##")
                && !line_trimmed.is_empty()
                && response.contains("Root Cause Analysis")
            {
                root_cause = Some(line_trimmed.to_string());
            }

            // Extract error messages (look for lines starting with "- ")
            if line_trimmed.starts_with("- ") && response.contains("Specific Error Messages") {
                messages.push(line_trimmed.trim_start_matches("- ").to_string());
            }

            // Extract fix strategy as suggestions
            if line_trimmed.starts_with("- ") && response.contains("Fix Strategy") {
                suggestions.push(line_trimmed.trim_start_matches("- ").to_string());
            }
        }

        // If we didn't extract specific messages, look for the Fix Strategy section as a whole
        if let Some(strategy_start) = response.find("## Fix Strategy") {
            if let Some(next_section) = response[strategy_start..].find("\n## ") {
                let strategy = &response[strategy_start..strategy_start + next_section];
                if !strategy.trim().is_empty() {
                    suggestions.push(strategy.lines().skip(1).collect::<Vec<_>>().join(" "));
                }
            }
        }

        Ok(ErrorAnalysis {
            category,
            messages,
            suggestions,
            root_cause,
        })
    }

    /// Heuristic-based error analysis (fallback)
    async fn heuristic_analyze(
        &self,
        output: &CommandOutput,
        category: ErrorCategory,
    ) -> Result<ErrorAnalysis> {
        let error_text = if !output.stderr.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        };

        let messages = self.extract_error_messages(error_text);
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
    registry: Arc<ModelRegistry>,
}

impl CodeRefiner {
    /// Create a new code refiner
    pub fn new(registry: Arc<ModelRegistry>) -> Result<Self> {
        Ok(Self { registry })
    }

    /// Load the code refiner prompt
    fn load_prompt(&self) -> Result<String> {
        // Use embedded prompt from file
        Ok(include_str!("../../../prompts/code/code_refiner.txt").to_string())
    }

    /// Generate refined implementation based on error analysis using LLM
    pub async fn refine(
        &self,
        original_impl: &ImplCode,
        test_code: &TestCode,
        error_analysis: &ErrorAnalysis,
        plan: &CodePlan,
    ) -> Result<ImplCode> {
        self.refine_with_iteration(original_impl, test_code, error_analysis, plan, 0)
            .await
    }

    /// Generate refined implementation with iteration context
    pub async fn refine_with_iteration(
        &self,
        original_impl: &ImplCode,
        test_code: &TestCode,
        error_analysis: &ErrorAnalysis,
        plan: &CodePlan,
        iteration: usize,
    ) -> Result<ImplCode> {
        // Try LLM-based refinement first
        match self
            .llm_refine(original_impl, test_code, error_analysis, plan, iteration)
            .await
        {
            Ok(refined) => Ok(refined),
            Err(e) => {
                tracing::warn!(
                    "LLM code refinement failed, falling back to heuristics: {}",
                    e
                );
                // Fall back to heuristic refinement
                self.heuristic_refine(original_impl, error_analysis).await
            }
        }
    }

    /// LLM-based code refinement
    async fn llm_refine(
        &self,
        original_impl: &ImplCode,
        test_code: &TestCode,
        error_analysis: &ErrorAnalysis,
        plan: &CodePlan,
        iteration: usize,
    ) -> Result<ImplCode> {
        let prompt_template = self.load_prompt()?;

        // Format plan context
        let plan_context = format!(
            "Purpose: {}\nRequirements: {}",
            plan.purpose,
            plan.requirements.join(", ")
        );

        // Format error analysis
        let error_analysis_text = format!(
            "Category: {:?}\nRoot Cause: {}\nMessages:\n{}\nSuggestions:\n{}",
            error_analysis.category,
            error_analysis.root_cause.as_deref().unwrap_or("Unknown"),
            error_analysis.messages.join("\n"),
            error_analysis.suggestions.join("\n")
        );

        // Fill in the template
        let prompt = prompt_template
            .replace("{plan_context}", &plan_context)
            .replace("{current_code}", &original_impl.code)
            .replace("{test_code}", &test_code.code)
            .replace("{error_analysis}", &error_analysis_text)
            .replace("{iteration}", &iteration.to_string())
            .replace(
                "{previous_error_category}",
                &format!("{:?}", error_analysis.category),
            );

        // Get coder model from registry
        let model_info =
            self.registry
                .get_model(&ModelRole::Coder, "code", &EngagementMode::Minimum)?;

        // Create model request
        let request = ModelRequest::new(ModelRole::Coder, "code", prompt);

        // Call the model backend
        let backend = self.registry.get_backend(&model_info.id).ok_or_else(|| {
            bodhya_core::Error::Config(format!(
                "Backend '{}' not found for model '{}'",
                model_info.definition.backend, model_info.id
            ))
        })?;

        let response = backend.generate(request).await?;

        // Extract code from response (look for ```rust code blocks)
        let refined_code = self.extract_code_from_response(&response.text)?;

        // Count lines of code
        let loc = refined_code
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count();

        Ok(ImplCode {
            code: refined_code,
            loc,
        })
    }

    /// Extract Rust code from LLM response
    fn extract_code_from_response(&self, response: &str) -> Result<String> {
        // Look for ```rust code blocks
        if let Some(start) = response.find("```rust") {
            let code_start = start + 7; // Skip "```rust"
            if let Some(end) = response[code_start..].find("```") {
                let code = response[code_start..code_start + end].trim();
                return Ok(code.to_string());
            }
        }

        // Try generic code blocks
        if let Some(start) = response.find("```") {
            let code_start = start + 3;
            if let Some(end) = response[code_start..].find("```") {
                let code = response[code_start..code_start + end].trim();
                // Skip language identifier if present
                if let Some(newline) = code.find('\n') {
                    return Ok(code[newline..].trim().to_string());
                }
                return Ok(code.to_string());
            }
        }

        // If no code blocks found, return the whole response (might be plain code)
        Ok(response.trim().to_string())
    }

    /// Heuristic-based code refinement (fallback)
    async fn heuristic_refine(
        &self,
        original_impl: &ImplCode,
        error_analysis: &ErrorAnalysis,
    ) -> Result<ImplCode> {
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
                // Heuristics can't fix logic errors
                // Just return original
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

            // Analyze errors with full context for LLM
            let plan_context = format!(
                "Purpose: {}\nRequirements: {}",
                plan.purpose,
                plan.requirements.join(", ")
            );
            let error_analysis = self
                .analyzer
                .analyze_with_context(
                    &test_result,
                    &plan_context,
                    &current_impl.code,
                    &test_code.code,
                )
                .await?;

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

            // Refine the implementation with iteration context
            current_impl = self
                .refiner
                .refine_with_iteration(&current_impl, test_code, &error_analysis, plan, iteration)
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
