/// MailAgent Evaluation Harness
///
/// Provides repeatable quality evaluation for MailAgent with standard test cases,
/// rating system, and comparison framework.
pub mod runner;
pub mod scorer;
pub mod standard_cases;
pub mod test_case;

pub use runner::{EvaluationResult, EvaluationRunner, EvaluationSummary};
pub use scorer::{EmailRating, EmailScorer, MAX_RATING};
pub use standard_cases::get_standard_cases;
pub use test_case::{EmailType, EmailValidation, MailTestCase};
