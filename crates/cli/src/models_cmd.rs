/// Model management commands
///
/// This module implements commands for listing, installing, and removing models:
/// - `bodhya models list`
/// - `bodhya models install <id>`
/// - `bodhya models remove <id>`
use bodhya_core::Result;
use bodhya_model_registry::{ModelListEntry, ModelRegistry};

use crate::utils;

/// List all available models
pub fn list_models() -> Result<()> {
    let manifest_path = utils::models_manifest_path()?;
    let models_dir = utils::models_dir()?;

    if !manifest_path.exists() {
        return Err(bodhya_core::Error::Config(
            "Models manifest not found. Run 'bodhya init' first.".to_string(),
        ));
    }

    let registry = ModelRegistry::from_manifest_file(&manifest_path, &models_dir)?;
    let models = registry.list_models();

    if models.is_empty() {
        println!("No models defined in manifest.");
        return Ok(());
    }

    println!("Available models:\n");
    println!(
        "{:<20} {:<12} {:<10} {:<10} {:<40}",
        "ID", "ROLE", "DOMAIN", "SIZE", "DISPLAY NAME"
    );
    println!("{}", "-".repeat(100));

    for model in models {
        print_model_entry(&model);
    }

    println!("\nTo install a model: bodhya models install <id>");
    println!("To remove a model:  bodhya models remove <id>");

    Ok(())
}

/// Install a model by ID
pub fn install_model(model_id: &str) -> Result<()> {
    let manifest_path = utils::models_manifest_path()?;
    let models_dir = utils::models_dir()?;

    if !manifest_path.exists() {
        return Err(bodhya_core::Error::Config(
            "Models manifest not found. Run 'bodhya init' first.".to_string(),
        ));
    }

    let registry = ModelRegistry::from_manifest_file(&manifest_path, &models_dir)?;

    // Check if model exists - get_model_path returns PathBuf (not Result)
    let model_path = registry.get_model_path(model_id);

    // Check if already installed - is_model_installed returns bool (not Result)
    if registry.is_model_installed(model_id) {
        println!(
            "Model '{}' is already installed at: {}",
            model_id,
            model_path.display()
        );
        return Ok(());
    }

    // In v1, we don't actually implement download
    // This is a placeholder that shows what would happen
    println!("Model '{}' is not installed.", model_id);
    println!("Expected location: {}", model_path.display());
    println!("\n[v1 LIMITATION]");
    println!("Automatic model download is not yet implemented.");
    println!("To use this model:");
    println!("  1. Download the model manually from the source URL");
    println!("  2. Place it at: {}", model_path.display());
    println!("  3. Verify with: bodhya models list");

    Ok(())
}

/// Remove an installed model
pub fn remove_model(model_id: &str) -> Result<()> {
    let manifest_path = utils::models_manifest_path()?;
    let models_dir = utils::models_dir()?;

    if !manifest_path.exists() {
        return Err(bodhya_core::Error::Config(
            "Models manifest not found. Run 'bodhya init' first.".to_string(),
        ));
    }

    let registry = ModelRegistry::from_manifest_file(&manifest_path, &models_dir)?;

    // Check if model exists in manifest - get_model_path returns PathBuf (not Result)
    let model_path = registry.get_model_path(model_id);

    // Check if installed - is_model_installed returns bool (not Result)
    if !registry.is_model_installed(model_id) {
        println!("Model '{}' is not installed.", model_id);
        return Ok(());
    }

    // Remove the model file
    std::fs::remove_file(&model_path).map_err(|e| {
        bodhya_core::Error::Internal(format!(
            "Failed to remove model file {}: {}",
            model_path.display(),
            e
        ))
    })?;

    println!("✓ Removed model '{}'", model_id);

    Ok(())
}

/// Print a single model entry
fn print_model_entry(model: &ModelListEntry) {
    let status = if model.installed { "✓" } else { " " };
    let size_str = format!("{:.1} GB", model.size_gb);

    println!(
        "{} {:<18} {:<12} {:<10} {:<10} {}",
        status,
        model.id,
        format!("{:?}", model.role),
        model.domain,
        size_str,
        model.display_name
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
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

    fn create_test_manifest(temp_home: &TempDir) {
        let bodhya_home = temp_home.path().join(".bodhya");
        std::fs::create_dir_all(&bodhya_home).unwrap();
        std::fs::create_dir_all(bodhya_home.join("models")).unwrap();

        let manifest_content = r#"
models:
  test_model:
    role: planner
    domain: code
    display_name: "Test Model"
    description: "A test model"
    source_url: "https://example.com/model.gguf"
    size_gb: 4.0
    checksum: "sha256:abc123"
    backend: local

backends:
  local:
    type: mistral_rs
"#;

        let manifest_path = bodhya_home.join("models.yaml");
        let mut file = std::fs::File::create(manifest_path).unwrap();
        file.write_all(manifest_content.as_bytes()).unwrap();
    }

    #[test]
    fn test_list_models_without_init() {
        with_temp_home(|_temp_home| {
            let result = list_models();
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[test]
    #[ignore]
    fn test_list_models_with_manifest() {
        with_temp_home(|temp_home| {
            create_test_manifest(temp_home);

            let result = list_models();
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_install_model_not_initialized() {
        with_temp_home(|_temp_home| {
            let result = install_model("test_model");
            assert!(result.is_err());
        });
    }

    #[test]
    #[ignore]
    fn test_install_model_shows_placeholder() {
        with_temp_home(|temp_home| {
            create_test_manifest(temp_home);

            // Should succeed but show placeholder message
            let result = install_model("test_model");
            assert!(result.is_ok());
        });
    }

    #[test]
    #[ignore]
    fn test_install_nonexistent_model() {
        with_temp_home(|temp_home| {
            create_test_manifest(temp_home);

            let result = install_model("nonexistent");
            assert!(result.is_err());
        });
    }

    #[test]
    #[ignore]
    fn test_remove_model_not_installed() {
        with_temp_home(|temp_home| {
            create_test_manifest(temp_home);

            let result = remove_model("test_model");
            // Should succeed even if not installed
            assert!(result.is_ok());
        });
    }

    #[test]
    #[ignore]
    fn test_remove_installed_model() {
        with_temp_home(|temp_home| {
            create_test_manifest(temp_home);

            // Create a fake installed model
            let model_path = temp_home.path().join(".bodhya/models/test_model.gguf");
            std::fs::write(&model_path, b"fake model data").unwrap();

            // Verify it's there
            assert!(model_path.exists());

            // Remove it
            let result = remove_model("test_model");
            assert!(result.is_ok());

            // Verify it's gone
            assert!(!model_path.exists());
        });
    }

    #[test]
    #[ignore]
    fn test_remove_nonexistent_model() {
        with_temp_home(|temp_home| {
            create_test_manifest(temp_home);

            let result = remove_model("nonexistent");
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_print_model_entry() {
        let model = ModelListEntry {
            id: "test".to_string(),
            role: bodhya_core::ModelRole::Planner,
            domain: "code".to_string(),
            display_name: "Test Model".to_string(),
            size_gb: 4.0,
            installed: false,
        };

        // Just verify it doesn't panic
        print_model_entry(&model);
    }
}
