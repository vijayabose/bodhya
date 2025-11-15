/// Task execution command
///
/// This module implements the `bodhya run` command for executing tasks.
/// In Phase 4, this is a minimal stub. Phase 5 will add full integration
/// with the controller and agents.
use bodhya_core::Result;

use crate::utils;

/// Run a task (stub implementation for Phase 4)
pub fn run_task(domain: Option<String>, task: String) -> Result<()> {
    // Check if initialized
    if !utils::is_initialized() {
        return Err(bodhya_core::Error::Config(
            "Bodhya is not initialized. Run 'bodhya init' first.".to_string(),
        ));
    }

    // Load config
    let config_path = utils::default_config_path()?;
    if !config_path.exists() {
        return Err(bodhya_core::Error::Config(
            "Configuration file not found. Run 'bodhya init' first.".to_string(),
        ));
    }

    println!("Task execution:");
    println!("  Domain: {}", domain.as_deref().unwrap_or("auto"));
    println!("  Task: {}", task);
    println!();

    // Phase 4 stub - just show what would happen
    println!("[STUB - Phase 4]");
    println!("Full task execution will be implemented in Phase 5.");
    println!("\nWhat would happen:");
    println!("  1. Load configuration from: {}", config_path.display());
    println!("  2. Initialize controller and register agents");
    println!("  3. Route task to appropriate agent");
    if let Some(d) = domain {
        println!("     - Using explicit domain: {}", d);
    } else {
        println!("     - Using keyword-based routing");
    }
    println!("  4. Execute task through agent");
    println!("  5. Display results");

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
            let result = run_task(None, "test task".to_string());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not initialized"));
        });
    }

    #[test]
    #[ignore]
    fn test_run_task_no_config_file() {
        with_temp_home(|temp_home| {
            // Create directory but no config file
            let bodhya_home = temp_home.path().join(".bodhya");
            let config_dir = bodhya_home.join("config");
            std::fs::create_dir_all(&config_dir).unwrap();

            let result = run_task(None, "test task".to_string());
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    // TODO: Fix HOME env var mocking for concurrent tests
    #[test]
    #[ignore]
    fn test_run_task_with_config() {
        with_temp_home(|temp_home| {
            setup_initialized_env(temp_home);

            let result = run_task(None, "Generate a hello world function".to_string());
            assert!(result.is_ok());
        });
    }

    // TODO: Fix HOME env var mocking for concurrent tests
    #[test]
    #[ignore]
    fn test_run_task_with_domain() {
        with_temp_home(|temp_home| {
            setup_initialized_env(temp_home);

            let result = run_task(Some("code".to_string()), "Generate code".to_string());
            assert!(result.is_ok());
        });
    }

    #[test]
    #[ignore]
    fn test_run_task_with_different_tasks() {
        with_temp_home(|temp_home| {
            setup_initialized_env(temp_home);

            let tasks = vec!["Write an email", "Generate Rust code", "Create a plan"];

            for task in tasks {
                let result = run_task(None, task.to_string());
                assert!(result.is_ok());
            }
        });
    }
}
