/// Integration tests for Bodhya CLI
///
/// These tests validate the end-to-end flow from CLI through controller to agents.
use bodhya_agent_code::CodeAgent;
use bodhya_cli::config_templates::{ConfigTemplate, Profile};
use bodhya_controller::TaskOrchestrator;
use bodhya_core::Task;
use std::sync::Arc;
use tempfile::TempDir;

/// Test the full vertical slice: Task → Controller → CodeAgent → Result
#[tokio::test]
async fn test_vertical_slice_code_agent() {
    // Create config
    let config = ConfigTemplate::for_profile(Profile::Code);

    // Initialize orchestrator
    let mut orchestrator = TaskOrchestrator::new(config);

    // Register CodeAgent
    let code_agent = Arc::new(CodeAgent::new());
    orchestrator.router_mut().register(code_agent);

    // Create task
    let task = Task::new("Generate a hello world function");

    // Execute task
    let result = orchestrator.execute(task).await;

    // Verify result
    assert!(result.is_ok());
    let agent_result = result.unwrap();
    assert!(agent_result.success);
    // Tool-based execution should have statistics, or fall back to static content
    assert!(
        agent_result.content.contains("Tool-Based Execution")
            || agent_result.content.contains("Hello, World!")
    );
}

/// Test task routing with explicit domain hint
#[tokio::test]
async fn test_vertical_slice_with_domain_hint() {
    let config = ConfigTemplate::for_profile(Profile::Code);
    let mut orchestrator = TaskOrchestrator::new(config);

    let code_agent = Arc::new(CodeAgent::new());
    orchestrator.router_mut().register(code_agent);

    // Create task with explicit domain
    let task = Task::new("Generate code").with_domain("code");

    let result = orchestrator.execute(task).await;

    assert!(result.is_ok());
    let agent_result = result.unwrap();
    assert!(agent_result.success);
    // Accept either tool-based or static output
    assert!(
        agent_result.content.contains("Tool-Based Execution")
            || agent_result.content.contains("Hello, World!")
    );
}

/// Test task routing via keywords (no explicit domain)
#[tokio::test]
async fn test_vertical_slice_keyword_routing() {
    let config = ConfigTemplate::for_profile(Profile::Code);
    let mut orchestrator = TaskOrchestrator::new(config);

    let code_agent = Arc::new(CodeAgent::new());
    orchestrator.router_mut().register(code_agent);

    // Task with keywords that match CodeAgent
    let task = Task::new("Write a Rust function that says hello");

    let result = orchestrator.execute(task).await;

    assert!(result.is_ok());
    let agent_result = result.unwrap();
    assert!(agent_result.success);
    // Accept either tool-based or static output
    assert!(
        agent_result.content.contains("Tool-Based Execution")
            || agent_result.content.contains("hello")
    );
}

/// Test multiple sequential task executions
#[tokio::test]
async fn test_vertical_slice_multiple_tasks() {
    let config = ConfigTemplate::for_profile(Profile::Code);
    let mut orchestrator = TaskOrchestrator::new(config);

    let code_agent = Arc::new(CodeAgent::new());
    orchestrator.router_mut().register(code_agent);

    let tasks = vec![
        "Generate hello world",
        "Create a Rust function",
        "Write code to print hello",
    ];

    for task_desc in tasks {
        let task = Task::new(task_desc);
        let result = orchestrator.execute(task).await;

        assert!(result.is_ok());
        let agent_result = result.unwrap();
        assert!(agent_result.success);
        // Accept either tool-based or static output
        assert!(
            agent_result.content.contains("Tool-Based Execution")
                || agent_result.content.contains("Hello, World!")
        );
    }
}

/// Test configuration loading and agent registration
#[tokio::test]
async fn test_config_based_agent_setup() {
    // Test code profile
    let code_config = ConfigTemplate::for_profile(Profile::Code);
    assert_eq!(code_config.profile, "code");
    assert!(code_config.agents.contains_key("code"));
    assert!(code_config.agents.get("code").unwrap().enabled);

    // Test full profile
    let full_config = ConfigTemplate::for_profile(Profile::Full);
    assert_eq!(full_config.profile, "full");
    assert!(full_config.agents.contains_key("code"));
    assert!(full_config.agents.contains_key("mail"));
}

/// Test file-based initialization (simulating bodhya init)
#[tokio::test]
async fn test_init_and_run_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();

    // Simulate bodhya init - write config file
    let config = ConfigTemplate::for_profile(Profile::Code);
    let config_yaml = serde_yaml::to_string(&config).unwrap();
    let config_path = config_dir.join("default.yaml");
    std::fs::write(&config_path, config_yaml).unwrap();

    // Load config from file (simulating bodhya run)
    let loaded_config_content = std::fs::read_to_string(&config_path).unwrap();
    let loaded_config: bodhya_core::AppConfig =
        serde_yaml::from_str(&loaded_config_content).unwrap();

    // Verify loaded config
    assert_eq!(loaded_config.profile, "code");
    assert!(loaded_config.agents.contains_key("code"));

    // Use loaded config to execute task
    let mut orchestrator = TaskOrchestrator::new(loaded_config);
    let code_agent = Arc::new(CodeAgent::new());
    orchestrator.router_mut().register(code_agent);

    let task = Task::new("Generate hello world");
    let result = orchestrator.execute(task).await.unwrap();

    assert!(result.success);
    // Accept either tool-based or static output
    assert!(
        result.content.contains("Tool-Based Execution") || result.content.contains("Hello, World!")
    );
}
