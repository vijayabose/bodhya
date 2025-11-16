/// Model lifecycle management
///
/// This module handles model installation, updates, and removal,
/// including user consent and download progress.
use bodhya_core::Result;
use std::path::PathBuf;

use crate::downloader::{DownloadResult, ModelDownloader};
use crate::manifest::ModelDefinition;

/// Model manager for handling model lifecycle
pub struct ModelManager {
    /// Model downloader
    downloader: ModelDownloader,
    /// Models directory
    models_dir: PathBuf,
}

impl ModelManager {
    /// Create a new model manager
    pub fn new(models_dir: impl Into<PathBuf>) -> Self {
        Self {
            downloader: ModelDownloader::new(),
            models_dir: models_dir.into(),
        }
    }

    /// Install a model from its definition
    ///
    /// Downloads the model from the source URL and verifies its checksum.
    ///
    /// # Arguments
    /// * `model_id` - Unique identifier for the model
    /// * `definition` - Model definition containing source URL and checksum
    ///
    /// # Returns
    /// DownloadResult with installation details
    pub async fn install_model(
        &self,
        model_id: &str,
        definition: &ModelDefinition,
    ) -> Result<DownloadResult> {
        tracing::info!("Installing model: {}", model_id);
        tracing::info!("  Display name: {}", definition.display_name);
        tracing::info!("  Size: {:.2} GB", definition.size_gb);
        tracing::info!("  Source: {}", definition.source_url);

        // Determine destination path
        let dest_path = self.models_dir.join(format!("{}.gguf", model_id));

        // Download and verify
        let result = self
            .downloader
            .download(
                &definition.source_url,
                &dest_path,
                Some(&definition.checksum),
            )
            .await?;

        tracing::info!("Model installed successfully: {}", model_id);

        Ok(result)
    }

    /// Check if a model is installed
    pub fn is_installed(&self, model_id: &str) -> bool {
        let path = self.get_model_path(model_id);
        path.exists()
    }

    /// Get the filesystem path for a model
    pub fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.models_dir.join(format!("{}.gguf", model_id))
    }

    /// Remove an installed model
    pub async fn remove_model(&self, model_id: &str) -> Result<()> {
        let path = self.get_model_path(model_id);

        if !path.exists() {
            tracing::warn!("Model {} is not installed, nothing to remove", model_id);
            return Ok(());
        }

        tokio::fs::remove_file(&path).await?;

        tracing::info!("Model {} removed from {:?}", model_id, path);

        Ok(())
    }

    /// Get information about an installed model
    pub async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo> {
        let path = self.get_model_path(model_id);

        if !path.exists() {
            return Err(bodhya_core::Error::ModelNotFound(format!(
                "Model {} is not installed",
                model_id
            )));
        }

        let metadata = tokio::fs::metadata(&path).await?;
        let size_bytes = metadata.len();

        Ok(ModelInfo {
            model_id: model_id.to_string(),
            path,
            size_bytes,
            installed: true,
        })
    }
}

/// Information about an installed model
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID
    pub model_id: String,
    /// Filesystem path
    pub path: PathBuf,
    /// Size in bytes
    pub size_bytes: u64,
    /// Whether the model is installed
    pub installed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_model_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        assert_eq!(manager.models_dir, temp_dir.path());
    }

    #[test]
    fn test_is_installed_false() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        assert!(!manager.is_installed("nonexistent_model"));
    }

    #[test]
    fn test_is_installed_true() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        // Create a dummy model file
        let model_path = temp_dir.path().join("test_model.gguf");
        std::fs::write(&model_path, b"dummy content").unwrap();

        assert!(manager.is_installed("test_model"));
    }

    #[test]
    fn test_get_model_path() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        let path = manager.get_model_path("test_model");
        assert_eq!(path, temp_dir.path().join("test_model.gguf"));
    }

    #[tokio::test]
    async fn test_remove_model() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        // Create a dummy model file
        let model_path = temp_dir.path().join("test_model.gguf");
        std::fs::write(&model_path, b"dummy content").unwrap();

        assert!(model_path.exists());
        assert!(manager.is_installed("test_model"));

        // Remove the model
        let result = manager.remove_model("test_model").await;
        assert!(result.is_ok());
        assert!(!model_path.exists());
        assert!(!manager.is_installed("test_model"));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_model() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        // Removing a non-existent model should not error
        let result = manager.remove_model("nonexistent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_model_info() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        // Create a dummy model file
        let model_path = temp_dir.path().join("test_model.gguf");
        let content = b"dummy content for testing size";
        std::fs::write(&model_path, content).unwrap();

        // Get model info
        let info = manager.get_model_info("test_model").await.unwrap();
        assert_eq!(info.model_id, "test_model");
        assert_eq!(info.size_bytes, content.len() as u64);
        assert!(info.installed);
    }

    #[tokio::test]
    async fn test_get_model_info_not_installed() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());

        // Try to get info for non-existent model
        let result = manager.get_model_info("nonexistent").await;
        assert!(result.is_err());
    }
}
