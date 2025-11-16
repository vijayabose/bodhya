/// Evaluation runner for CodeAgent
use crate::scorer::{CodeScorer, QualityScore, MAX_SCORE};
use crate::test_case::CodeTestCase;
use bodhya_agent_code::CodeAgent;
use bodhya_core::{Agent, AgentContext, Task};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Results from running an evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub test_case_id: String,
    pub test_case_name: String,
    pub score: QualityScore,
    pub output: String,
    pub duration: Duration,
    pub error: Option<String>,
}

/// Summary of evaluation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSummary {
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub average_score: f64,
    pub results: Vec<EvaluationResult>,
}

impl EvaluationSummary {
    pub fn new(results: Vec<EvaluationResult>) -> Self {
        let total_cases = results.len();
        let passed = results.iter().filter(|r| r.score.is_passing()).count();
        let failed = total_cases - passed;

        let average_score = if total_cases > 0 {
            results.iter().map(|r| r.score.total).sum::<f64>() / total_cases as f64
        } else {
            0.0
        };

        Self {
            total_cases,
            passed,
            failed,
            average_score,
            results,
        }
    }

    pub fn is_passing(&self) -> bool {
        self.average_score >= 85.0
    }

    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(80).bright_blue());
        println!("{}", "CodeAgent Evaluation Summary".bright_cyan().bold());
        println!("{}", "=".repeat(80).bright_blue());

        println!("\n{}: {}", "Total Cases".bold(), self.total_cases);
        let passed_pct = (self.passed as f64 / self.total_cases as f64) * 100.0;
        println!(
            "{}: {} ({:.1}%)",
            "Passed".bold(),
            self.passed.to_string().bright_green(),
            passed_pct
        );
        let failed_pct = (self.failed as f64 / self.total_cases as f64) * 100.0;
        println!(
            "{}: {} ({:.1}%)",
            "Failed".bold(),
            self.failed.to_string().bright_red(),
            failed_pct
        );

        println!(
            "\n{}: {:.2}/{:.0}",
            "Average Score".bold(),
            self.average_score,
            MAX_SCORE
        );

        if self.is_passing() {
            println!(
                "\n{}",
                "✓ EVALUATION PASSED (≥85/100)".bright_green().bold()
            );
        } else {
            println!("\n{}", "✗ EVALUATION FAILED (<85/100)".bright_red().bold());
        }

        println!("\n{}", "Individual Results:".bold());
        println!("{}", "-".repeat(80).bright_blue());

        for result in &self.results {
            let status = if result.score.is_passing() {
                "PASS".bright_green()
            } else {
                "FAIL".bright_red()
            };

            println!(
                "\n[{}] {} ({})",
                status,
                result.test_case_name.bold(),
                result.test_case_id
            );
            println!(
                "  Score: {:.2}/{:.0} (C:{:.1} S:{:.1} Cov:{:.1})",
                result.score.total,
                MAX_SCORE,
                result.score.correctness,
                result.score.style,
                result.score.coverage
            );
            println!("  Duration: {:?}", result.duration);

            if !result.score.feedback.is_empty() {
                println!("  Feedback:");
                for fb in &result.score.feedback {
                    println!("    {}", fb);
                }
            }

            if let Some(ref error) = result.error {
                println!("  {}: {}", "Error".bright_red(), error);
            }
        }

        println!("\n{}", "=".repeat(80).bright_blue());
    }
}

/// Run evaluations on a set of test cases
pub struct EvaluationRunner {
    agent: CodeAgent,
}

impl EvaluationRunner {
    pub fn new(agent: CodeAgent) -> Self {
        Self { agent }
    }

    /// Run a single test case
    pub async fn run_test_case(&self, test_case: &CodeTestCase) -> EvaluationResult {
        let start = Instant::now();

        println!(
            "\nRunning: {} ({})",
            test_case.name.bright_cyan(),
            test_case.id
        );

        // Create task from test case
        let task = Task {
            id: test_case.id.clone(),
            domain_hint: Some("code".to_string()),
            description: test_case.description.clone(),
            payload: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };

        // Create minimal agent context
        let context = AgentContext {
            config: bodhya_core::AppConfig::default(),
            metadata: serde_json::Value::Null,
        };

        // Run the agent
        let (output, error) = match self.agent.handle(task, context).await {
            Ok(result) => (result.content, None),
            Err(e) => (String::new(), Some(e.to_string())),
        };

        // Score the output
        let score = CodeScorer::score(test_case, &output);

        let duration = start.elapsed();

        println!("  Score: {:.2}/{:.0}", score.total, MAX_SCORE);

        EvaluationResult {
            test_case_id: test_case.id.clone(),
            test_case_name: test_case.name.clone(),
            score,
            output,
            duration,
            error,
        }
    }

    /// Run all test cases
    pub async fn run_all(&self, test_cases: &[CodeTestCase]) -> EvaluationSummary {
        println!("{}", "Starting CodeAgent Evaluation".bright_cyan().bold());
        println!("Test cases: {}\n", test_cases.len());

        let mut results = Vec::new();

        for test_case in test_cases {
            let result = self.run_test_case(test_case).await;
            results.push(result);
        }

        EvaluationSummary::new(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluation_result_creation() {
        let score = QualityScore::new(35.0, 25.0, 28.0);
        let result = EvaluationResult {
            test_case_id: "test1".to_string(),
            test_case_name: "Test 1".to_string(),
            score,
            output: "fn main() {}".to_string(),
            duration: Duration::from_secs(1),
            error: None,
        };

        assert_eq!(result.test_case_id, "test1");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_evaluation_summary() {
        let passing_score = QualityScore::new(35.0, 28.0, 27.0); // 90 total
        let failing_score = QualityScore::new(25.0, 20.0, 20.0); // 65 total

        let results = vec![
            EvaluationResult {
                test_case_id: "test1".to_string(),
                test_case_name: "Test 1".to_string(),
                score: passing_score.clone(),
                output: String::new(),
                duration: Duration::from_secs(1),
                error: None,
            },
            EvaluationResult {
                test_case_id: "test2".to_string(),
                test_case_name: "Test 2".to_string(),
                score: failing_score,
                output: String::new(),
                duration: Duration::from_secs(1),
                error: None,
            },
        ];

        let summary = EvaluationSummary::new(results);

        assert_eq!(summary.total_cases, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.average_score, 77.5);
        assert!(!summary.is_passing());
    }

    #[test]
    fn test_summary_all_passing() {
        let passing_score = QualityScore::new(35.0, 28.0, 27.0); // 90 total

        let results = vec![
            EvaluationResult {
                test_case_id: "test1".to_string(),
                test_case_name: "Test 1".to_string(),
                score: passing_score.clone(),
                output: String::new(),
                duration: Duration::from_secs(1),
                error: None,
            },
            EvaluationResult {
                test_case_id: "test2".to_string(),
                test_case_name: "Test 2".to_string(),
                score: passing_score,
                output: String::new(),
                duration: Duration::from_secs(1),
                error: None,
            },
        ];

        let summary = EvaluationSummary::new(results);

        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
        assert!(summary.is_passing());
    }
}
