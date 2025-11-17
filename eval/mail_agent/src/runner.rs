/// Evaluation runner for MailAgent
use crate::scorer::{EmailRating, EmailScorer, MAX_RATING};
use crate::test_case::MailTestCase;
use bodhya_agent_mail::MailAgent;
use bodhya_core::{Agent, AgentContext, Task};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Results from running an evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub test_case_id: String,
    pub test_case_name: String,
    pub rating: EmailRating,
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
    pub average_rating: f64,
    pub results: Vec<EvaluationResult>,
}

impl EvaluationSummary {
    pub fn new(results: Vec<EvaluationResult>) -> Self {
        let total_cases = results.len();
        let passed = results.iter().filter(|r| r.rating.is_passing()).count();
        let failed = total_cases - passed;

        let average_rating = if total_cases > 0 {
            results.iter().map(|r| r.rating.total).sum::<f64>() / total_cases as f64
        } else {
            0.0
        };

        Self {
            total_cases,
            passed,
            failed,
            average_rating,
            results,
        }
    }

    pub fn is_passing(&self) -> bool {
        self.average_rating >= 4.5
    }

    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(80).bright_blue());
        println!("{}", "MailAgent Evaluation Summary".bright_cyan().bold());
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
            "\n{}: {:.2}/{:.1} ★",
            "Average Rating".bold(),
            self.average_rating,
            MAX_RATING
        );

        if self.is_passing() {
            println!(
                "\n{}",
                "✓ EVALUATION PASSED (≥4.5/5.0)".bright_green().bold()
            );
        } else {
            println!("\n{}", "✗ EVALUATION FAILED (<4.5/5.0)".bright_red().bold());
        }

        println!("\n{}", "Individual Results:".bold());
        println!("{}", "-".repeat(80).bright_blue());

        for result in &self.results {
            let status = if result.rating.is_passing() {
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
                "  Rating: {:.2}/{:.1} ★ (Tone:{:.2} Clarity:{:.2} Length:{:.2} Complete:{:.2})",
                result.rating.total,
                MAX_RATING,
                result.rating.tone,
                result.rating.clarity,
                result.rating.length,
                result.rating.completeness
            );
            println!("  Duration: {:?}", result.duration);

            if !result.rating.feedback.is_empty() {
                println!("  Feedback:");
                for fb in &result.rating.feedback {
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
    agent: MailAgent,
}

impl EvaluationRunner {
    pub fn new(agent: MailAgent) -> Self {
        Self { agent }
    }

    /// Run a single test case
    pub async fn run_test_case(&self, test_case: &MailTestCase) -> EvaluationResult {
        let start = Instant::now();

        println!(
            "\nRunning: {} ({})",
            test_case.name.bright_cyan(),
            test_case.id
        );

        // Create task from test case
        let task = Task {
            id: test_case.id.clone(),
            domain_hint: Some("mail".to_string()),
            description: format!("{} {}", test_case.context, test_case.purpose),
            payload: serde_json::json!({
                "context": test_case.context,
                "purpose": test_case.purpose,
            }),
            created_at: chrono::Utc::now(),
        };

        // Create minimal agent context
        let context = AgentContext {
            config: bodhya_core::AppConfig::default(),
            metadata: serde_json::Value::Null,
            working_dir: None,
            execution_limits: bodhya_core::ExecutionLimits::default(),
            tools: None,
        };

        // Run the agent
        let (output, error) = match self.agent.handle(task, context).await {
            Ok(result) => (result.content, None),
            Err(e) => (String::new(), Some(e.to_string())),
        };

        // Score the output
        let rating = EmailScorer::score(test_case, &output);

        let duration = start.elapsed();

        println!("  Rating: {:.2}/{:.1} ★", rating.total, MAX_RATING);

        EvaluationResult {
            test_case_id: test_case.id.clone(),
            test_case_name: test_case.name.clone(),
            rating,
            output,
            duration,
            error,
        }
    }

    /// Run all test cases
    pub async fn run_all(&self, test_cases: &[MailTestCase]) -> EvaluationSummary {
        println!("{}", "Starting MailAgent Evaluation".bright_cyan().bold());
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
        let rating = EmailRating::new(1.2, 1.3, 0.9, 0.8);
        let result = EvaluationResult {
            test_case_id: "test1".to_string(),
            test_case_name: "Test 1".to_string(),
            rating,
            output: "email content".to_string(),
            duration: Duration::from_secs(1),
            error: None,
        };

        assert_eq!(result.test_case_id, "test1");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_evaluation_summary() {
        let passing_rating = EmailRating::new(1.4, 1.4, 0.9, 0.9); // 4.6 total
        let failing_rating = EmailRating::new(1.0, 1.0, 0.8, 0.8); // 3.6 total

        let results = vec![
            EvaluationResult {
                test_case_id: "test1".to_string(),
                test_case_name: "Test 1".to_string(),
                rating: passing_rating,
                output: String::new(),
                duration: Duration::from_secs(1),
                error: None,
            },
            EvaluationResult {
                test_case_id: "test2".to_string(),
                test_case_name: "Test 2".to_string(),
                rating: failing_rating,
                output: String::new(),
                duration: Duration::from_secs(1),
                error: None,
            },
        ];

        let summary = EvaluationSummary::new(results);

        assert_eq!(summary.total_cases, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.average_rating, 4.1);
        assert!(!summary.is_passing());
    }

    #[test]
    fn test_summary_all_passing() {
        let passing_rating = EmailRating::new(1.4, 1.4, 0.9, 0.9); // 4.6 total

        let results = vec![
            EvaluationResult {
                test_case_id: "test1".to_string(),
                test_case_name: "Test 1".to_string(),
                rating: passing_rating.clone(),
                output: String::new(),
                duration: Duration::from_secs(1),
                error: None,
            },
            EvaluationResult {
                test_case_id: "test2".to_string(),
                test_case_name: "Test 2".to_string(),
                rating: passing_rating,
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
