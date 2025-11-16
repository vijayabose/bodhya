/// CodeAgent Evaluation Harness - Main Entry Point
use bodhya_agent_code::CodeAgent;
use bodhya_eval_code_agent::{get_standard_cases, EvaluationRunner};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Bodhya CodeAgent Evaluation Harness\n");

    // Create agent (without model registry - will use static responses)
    let agent = CodeAgent::new();

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
