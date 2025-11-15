/// Model registry for role-based model lookup
///
/// This module provides a registry that maps (role, domain, engagement) tuples
/// to appropriate model backends, handling model selection logic.
use bodhya_core::{EngagementMode, Error, ModelBackend, ModelRole, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::manifest::{ModelDefinition, ModelManifest};

/// Model registry for looking up and managing models
pub struct ModelRegistry {
    /// Loaded manifest
    manifest: ModelManifest,
    /// Cached backend instances
    backends: HashMap<String, Arc<dyn ModelBackend>>,
    /// Models directory path
    models_dir: PathBuf,
}

impl ModelRegistry {
    /// Create a new model registry from a manifest file
    pub fn from_manifest_file(
        manifest_path: impl Into<PathBuf>,
        models_dir: impl Into<PathBuf>,
    ) -> Result<Self> {
        let manifest_path = manifest_path.into();
        let manifest = ModelManifest::from_file(&manifest_path)?;

        Ok(Self {
            manifest,
            backends: HashMap::new(),
            models_dir: models_dir.into(),
        })
    }

    /// Create a registry from an already loaded manifest
    pub fn from_manifest(manifest: ModelManifest, models_dir: impl Into<PathBuf>) -> Self {
        Self {
            manifest,
            backends: HashMap::new(),
            models_dir: models_dir.into(),
        }
    }

    /// Get a model backend for the given role, domain, and engagement mode
    ///
    /// This is the primary API for agents to obtain models.
    pub fn get_model(
        &self,
        role: &ModelRole,
        domain: &str,
        engagement: &EngagementMode,
    ) -> Result<ModelInfo> {
        // In v1, we only support local models with Minimum engagement
        if *engagement != EngagementMode::Minimum {
            return Err(Error::EngagementViolation(format!(
                "Only Minimum engagement mode is supported in v1, requested: {:?}",
                engagement
            )));
        }

        // Find models matching role and domain
        let candidates = self.manifest.find_models(role, domain);

        if candidates.is_empty() {
            return Err(Error::ModelNotFound(format!(
                "No model found for role={} domain={}",
                role, domain
            )));
        }

        // For now, take the first matching model
        // Future: could implement preference ordering, fallbacks, etc.
        let (model_id, model_def) = candidates[0];

        Ok(ModelInfo {
            id: model_id.clone(),
            definition: model_def.clone(),
            installed: self.is_model_installed(model_id),
            model_path: self.get_model_path(model_id),
        })
    }

    /// Check if a model is installed
    pub fn is_model_installed(&self, model_id: &str) -> bool {
        let path = self.get_model_path(model_id);
        path.exists()
    }

    /// Get the filesystem path for a model
    pub fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.models_dir.join(format!("{}.gguf", model_id))
    }

    /// List all models in the manifest
    pub fn list_models(&self) -> Vec<ModelListEntry> {
        self.manifest
            .models
            .iter()
            .map(|(id, def)| ModelListEntry {
                id: id.clone(),
                role: def.role.clone(),
                domain: def.domain.clone(),
                display_name: def.display_name.clone(),
                size_gb: def.size_gb,
                installed: self.is_model_installed(id),
            })
            .collect()
    }

    /// Get the manifest
    pub fn manifest(&self) -> &ModelManifest {
        &self.manifest
    }

    /// Register a backend instance for a model ID
    pub fn register_backend(&mut self, model_id: String, backend: Arc<dyn ModelBackend>) {
        self.backends.insert(model_id, backend);
    }

    /// Get a registered backend
    pub fn get_backend(&self, model_id: &str) -> Option<Arc<dyn ModelBackend>> {
        self.backends.get(model_id).cloned()
    }
}

/// Information about a model resolved from the registry
#[derive(Clone, Debug)]
pub struct ModelInfo {
    /// Model ID
    pub id: String,
    /// Model definition from manifest
    pub definition: ModelDefinition,
    /// Whether the model is installed locally
    pub installed: bool,
    /// Path to the model file
    pub model_path: PathBuf,
}

/// Entry in the model list
#[derive(Clone, Debug)]
pub struct ModelListEntry {
    /// Model ID
    pub id: String,
    /// Model role
    pub role: ModelRole,
    /// Domain
    pub domain: String,
    /// Display name
    pub display_name: String,
    /// Size in GB
    pub size_gb: f64,
    /// Whether installed
    pub installed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manifest() -> ModelManifest {
        let yaml = r#"
models:
  test_planner:
    role: planner
    domain: code
    display_name: "Test Planner"
    description: "Test model"
    source_url: "https://example.com/model.gguf"
    size_gb: 4.0
    checksum: "sha256:abc123"
    backend: local

  test_coder:
    role: coder
    domain: code
    display_name: "Test Coder"
    source_url: "https://example.com/coder.gguf"
    size_gb: 3.5
    checksum: "sha256:def456"
    backend: local

  test_writer:
    role: writer
    domain: mail
    display_name: "Test Writer"
    source_url: "https://example.com/writer.gguf"
    size_gb: 4.2
    checksum: "sha256:ghi789"
    backend: local

backends:
  local:
    type: mistral_rs
    enabled: true
"#;
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_registry_creation() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();

        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());
        assert_eq!(registry.manifest.models.len(), 3);
    }

    #[test]
    fn test_get_model_by_role_and_domain() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        let model = registry
            .get_model(&ModelRole::Planner, "code", &EngagementMode::Minimum)
            .unwrap();

        assert_eq!(model.id, "test_planner");
        assert_eq!(model.definition.display_name, "Test Planner");
        assert!(!model.installed);
    }

    #[test]
    fn test_get_model_not_found() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        let result = registry.get_model(&ModelRole::Summarizer, "code", &EngagementMode::Minimum);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ModelNotFound(_)));
    }

    #[test]
    fn test_engagement_mode_validation() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        // Medium and Maximum should fail in v1
        let result = registry.get_model(&ModelRole::Planner, "code", &EngagementMode::Medium);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::EngagementViolation(_)));

        let result = registry.get_model(&ModelRole::Planner, "code", &EngagementMode::Maximum);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_model_installed() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();

        // Create a fake installed model
        std::fs::write(
            temp_dir.path().join("test_planner.gguf"),
            b"fake model data",
        )
        .unwrap();

        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        assert!(registry.is_model_installed("test_planner"));
        assert!(!registry.is_model_installed("test_coder"));
    }

    #[test]
    fn test_get_model_path() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        let path = registry.get_model_path("test_planner");
        assert!(path.to_str().unwrap().contains("test_planner.gguf"));
    }

    #[test]
    fn test_list_models() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        let models = registry.list_models();
        assert_eq!(models.len(), 3);

        let planner = models.iter().find(|m| m.id == "test_planner").unwrap();
        assert_eq!(planner.role, ModelRole::Planner);
        assert_eq!(planner.domain, "code");
        assert!(!planner.installed);
    }

    #[test]
    fn test_list_models_with_installed() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();

        // Create an installed model
        std::fs::write(temp_dir.path().join("test_coder.gguf"), b"fake model data").unwrap();

        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());
        let models = registry.list_models();

        let coder = models.iter().find(|m| m.id == "test_coder").unwrap();
        assert!(coder.installed);

        let planner = models.iter().find(|m| m.id == "test_planner").unwrap();
        assert!(!planner.installed);
    }

    #[test]
    fn test_get_model_info_includes_path() {
        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        let model = registry
            .get_model(&ModelRole::Writer, "mail", &EngagementMode::Minimum)
            .unwrap();

        assert!(model
            .model_path
            .to_str()
            .unwrap()
            .contains("test_writer.gguf"));
    }
}
