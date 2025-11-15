/// CLI utility functions
///
/// This module provides common utilities for path management,
/// directory creation, and file operations.
use bodhya_core::{Error, Result};
use std::path::PathBuf;

/// Get the Bodhya home directory (~/.bodhya)
pub fn bodhya_home() -> Result<PathBuf> {
    let home = home::home_dir()
        .ok_or_else(|| Error::Config("Could not determine home directory".to_string()))?;
    Ok(home.join(".bodhya"))
}

/// Get the config directory (~/.bodhya/config)
pub fn config_dir() -> Result<PathBuf> {
    Ok(bodhya_home()?.join("config"))
}

/// Get the default config file path (~/.bodhya/config/default.yaml)
pub fn default_config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("default.yaml"))
}

/// Get the models directory (~/.bodhya/models)
pub fn models_dir() -> Result<PathBuf> {
    Ok(bodhya_home()?.join("models"))
}

/// Get the models manifest path (~/.bodhya/models.yaml)
pub fn models_manifest_path() -> Result<PathBuf> {
    Ok(bodhya_home()?.join("models.yaml"))
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| {
            Error::Config(format!(
                "Failed to create directory {}: {}",
                path.display(),
                e
            ))
        })?;
    }
    Ok(())
}

/// Check if Bodhya is initialized (config directory exists)
pub fn is_initialized() -> bool {
    config_dir().ok().map(|p| p.exists()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bodhya_home() {
        let home = bodhya_home().unwrap();
        assert!(home.to_str().unwrap().ends_with(".bodhya"));
    }

    #[test]
    fn test_config_dir() {
        let dir = config_dir().unwrap();
        assert!(dir.to_str().unwrap().contains(".bodhya"));
        assert!(dir.to_str().unwrap().ends_with("config"));
    }

    #[test]
    fn test_default_config_path() {
        let path = default_config_path().unwrap();
        assert!(path.to_str().unwrap().contains(".bodhya"));
        assert!(path.to_str().unwrap().ends_with("default.yaml"));
    }

    #[test]
    fn test_models_dir() {
        let dir = models_dir().unwrap();
        assert!(dir.to_str().unwrap().contains(".bodhya"));
        assert!(dir.to_str().unwrap().ends_with("models"));
    }

    #[test]
    fn test_models_manifest_path() {
        let path = models_manifest_path().unwrap();
        assert!(path.to_str().unwrap().contains(".bodhya"));
        assert!(path.to_str().unwrap().ends_with("models.yaml"));
    }

    #[test]
    fn test_ensure_dir() {
        let temp = TempDir::new().unwrap();
        let test_dir = temp.path().join("test").join("nested").join("dir");

        // Directory doesn't exist yet
        assert!(!test_dir.exists());

        // Create it
        ensure_dir(&test_dir).unwrap();

        // Now it exists
        assert!(test_dir.exists());

        // Calling again should not error
        ensure_dir(&test_dir).unwrap();
    }

    #[test]
    fn test_is_initialized_returns_bool() {
        // This test assumes ~/.bodhya/config may or may not exist
        // For safety, we'll just check it returns a boolean value
        let result = is_initialized();
        // Just verify the function doesn't panic and returns a bool
        let _ = result;
    }
}
