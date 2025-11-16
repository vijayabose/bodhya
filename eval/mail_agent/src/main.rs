/// MailAgent Evaluation Harness - Main Entry Point
use bodhya_agent_mail::MailAgent;
use bodhya_eval_mail_agent::{get_standard_cases, EvaluationRunner};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Bodhya MailAgent Evaluation Harness\n");

    // Create agent (without model registry - will use static responses)
    let agent = MailAgent::new();

    // Create runner
    let runner = EvaluationRunner::new(agent);

    // Get standard test cases
    let test_cases = get_standard_cases();

    // Run evaluation
    let summary = runner.run_all(&test_cases).await;

    // Print results
    summary.print_summary();

    // Exit with appropriate code
    if summary.is_passing() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
