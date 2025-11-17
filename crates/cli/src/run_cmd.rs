/// Task execution command
///
/// This module implements the `bodhya run` command for executing tasks.
/// Phase 5 adds full integration with the controller and code agent.
use bodhya_agent_code::CodeAgent;
use bodhya_controller::TaskOrchestrator;
use bodhya_core::{AppConfig, Result, Task};
use std::sync::Arc;

use crate::utils;

/// Run a task through the controller
pub async fn run_task(
    domain: Option<String>,
    working_dir: Option<String>,
    task_description: String,
) -> Result<()> {
    // Check if initialized
    if !utils::is_initialized() {
        return Err(bodhya_core::Error::Config(
            "Bodhya is not initialized. Run 'bodhya init' first.".to_string(),
        ));
    }

    // Validate and resolve working directory
    let working_dir_path = if let Some(dir) = working_dir {
        let path = std::path::PathBuf::from(&dir);
        if !path.exists() {
            return Err(bodhya_core::Error::Config(format!(
                "Working directory does not exist: {}",
                dir
            )));
        }
        if !path.is_dir() {
            return Err(bodhya_core::Error::Config(format!(
                "Path is not a directory: {}",
                dir
            )));
        }
        Some(path)
    } else {
        // Default to current directory
        std::env::current_dir().ok()
    };

    // Load config
    let config_path = utils::default_config_path()?;
    if !config_path.exists() {
        return Err(bodhya_core::Error::Config(
            "Configuration file not found. Run 'bodhya init' first.".to_string(),
        ));
    }

    let config_content = std::fs::read_to_string(&config_path).map_err(|e| {
        bodhya_core::Error::Config(format!(
            "Failed to read config from {}: {}",
            config_path.display(),
            e
        ))
    })?;

    let config: AppConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| bodhya_core::Error::Config(format!("Failed to parse config: {}", e)))?;

    // Initialize orchestrator with code agent
    // Note: TaskOrchestrator::new() already creates ToolRegistry with defaults
    let mut orchestrator = TaskOrchestrator::new(config);

    // Set working directory if specified
    if let Some(wd) = working_dir_path {
        orchestrator.set_working_dir(wd);
    }

    // Register CodeAgent (Phase 5: only code agent)
    let code_agent = Arc::new(CodeAgent::new());
    orchestrator.router_mut().register(code_agent);

    // Create task
    let mut task = Task::new(&task_description);
    if let Some(d) = domain {
        task = task.with_domain(&d);
    }

    // Execute task
    println!("Executing task: {}", task_description);
    if task.domain_hint.is_some() {
        println!("Domain: {}", task.domain_hint.as_ref().unwrap());
    }
    println!();

    let result = orchestrator.execute(task).await?;

    // Display result
    if result.success {
        println!("✓ Task completed successfully\n");
        println!("{}", result.content);
    } else {
        println!("✗ Task failed\n");
        println!("{}", result.content);
        if let Some(error) = &result.error {
            println!("\nError: {}", error);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_templates::{ConfigTemplate, Profile};
    use std::env;
    use tempfile::TempDir;

    fn with_temp_home<F>(f: F)
    where
        F: FnOnce(&TempDir),
    {
        let temp_home = TempDir::new().unwrap();
        let old_home = env::var("HOME").ok();

        env::set_var("HOME", temp_home.path());

        f(&temp_home);

        if let Some(home) = old_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    fn setup_initialized_env(temp_home: &TempDir) {
        let bodhya_home = temp_home.path().join(".bodhya");
        let config_dir = bodhya_home.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        let config = ConfigTemplate::for_profile(Profile::Code);
        let config_yaml = serde_yaml::to_string(&config).unwrap();
        let config_path = config_dir.join("default.yaml");
        std::fs::write(config_path, config_yaml).unwrap();
    }

    #[test]
    fn test_run_task_not_initialized() {
        with_temp_home(|_temp_home| {
            // Verify that bodhya is not initialized in a temp home
            // The actual run_task call would fail with "not initialized" error
            assert!(!utils::is_initialized());
        });
    }

    #[tokio::test]
    #[ignore]
    async fn test_run_task_no_config_file() {
        with_temp_home(|temp_home| {
            // Create directory but no config file
            let bodhya_home = temp_home.path().join(".bodhya");
            let config_dir = bodhya_home.join("config");
            std::fs::create_dir_all(&config_dir).unwrap();

            let rt = tokio::runtime::Handle::current();
            let result = rt.block_on(run_task(None, None, "test task".to_string()));
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    // TODO: Fix HOME env var mocking for concurrent tests
    #[tokio::test]
    #[ignore]
    async fn test_run_task_with_config() {
        with_temp_home(|temp_home| {
            setup_initialized_env(temp_home);

            let rt = tokio::runtime::Handle::current();
            let result = rt.block_on(run_task(
                None,
                None,
                "Generate a hello world function".to_string(),
            ));
            assert!(result.is_ok());
        });
    }

    // TODO: Fix HOME env var mocking for concurrent tests
    #[tokio::test]
    #[ignore]
    async fn test_run_task_with_domain() {
        with_temp_home(|temp_home| {
            setup_initialized_env(temp_home);

            let rt = tokio::runtime::Handle::current();
            let result = rt.block_on(run_task(
                Some("code".to_string()),
                None,
                "Generate code".to_string(),
            ));
            assert!(result.is_ok());
        });
    }

    #[tokio::test]
    #[ignore]
    async fn test_run_task_with_different_tasks() {
        with_temp_home(|temp_home| {
            setup_initialized_env(temp_home);

            let tasks = vec!["Write an email", "Generate Rust code", "Create a plan"];

            let rt = tokio::runtime::Handle::current();
            for task in tasks {
                let result = rt.block_on(run_task(None, None, task.to_string()));
                assert!(result.is_ok());
            }
        });
    }
}
