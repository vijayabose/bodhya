/// CodeAgent Evaluation Harness
///
/// Provides repeatable quality evaluation for CodeAgent with standard test cases,
/// scoring system, and comparison framework.
pub mod runner;
pub mod scorer;
pub mod standard_cases;
pub mod test_case;

pub use runner::{EvaluationResult, EvaluationRunner, EvaluationSummary};
pub use scorer::{CodeScorer, QualityScore, MAX_SCORE};
pub use standard_cases::get_standard_cases;
pub use test_case::{CodeTestCase, Difficulty, ValidationCriteria};
