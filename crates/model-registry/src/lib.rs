/// Bodhya Model Registry
///
/// This crate provides model manifest parsing, model lookup/selection,
/// and model backend implementations (local and remote).
pub use local_mistral::{LocalBackendConfig, LocalMistralBackend};
pub use manifest::{BackendConfig, ModelDefinition, ModelManifest};
pub use registry::{ModelInfo, ModelListEntry, ModelRegistry};
pub use remote_stub::{RemoteBackend, RemoteBackendConfig};

pub mod local_mistral;
pub mod manifest;
pub mod registry;
pub mod remote_stub;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use bodhya_core::{EngagementMode, ModelBackend, ModelRequest, ModelRole};
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    /// Integration test: Full workflow from manifest to model lookup
    #[test]
    fn test_full_manifest_to_registry_workflow() {
        let manifest_yaml = r#"
models:
  test_planner:
    role: planner
    domain: code
    display_name: "Test Planner"
    description: "Test planning model"
    source_url: "https://example.com/planner.gguf"
    size_gb: 4.0
    checksum: "sha256:abc123"
    backend: local

backends:
  local:
    type: mistral_rs
    enabled: true
"#;

        // Create temporary manifest file
        let mut manifest_file = NamedTempFile::new().unwrap();
        write!(manifest_file, "{}", manifest_yaml).unwrap();

        // Create temporary models directory
        let models_dir = TempDir::new().unwrap();

        // Load registry from manifest
        let registry =
            ModelRegistry::from_manifest_file(manifest_file.path(), models_dir.path()).unwrap();

        // Lookup model by role and domain
        let model_info = registry
            .get_model(&ModelRole::Planner, "code", &EngagementMode::Minimum)
            .unwrap();

        assert_eq!(model_info.id, "test_planner");
        assert_eq!(model_info.definition.display_name, "Test Planner");
        assert!(!model_info.installed);
    }

    /// Integration test: Model listing
    #[test]
    fn test_list_all_models() {
        let manifest_yaml = r#"
models:
  planner_1:
    role: planner
    domain: code
    display_name: "Planner 1"
    source_url: "https://example.com/p1.gguf"
    size_gb: 4.0
    checksum: "sha256:abc"
    backend: local

  coder_1:
    role: coder
    domain: code
    display_name: "Coder 1"
    source_url: "https://example.com/c1.gguf"
    size_gb: 3.5
    checksum: "sha256:def"
    backend: local

backends:
  local:
    type: mistral_rs
"#;

        let manifest: ModelManifest = serde_yaml::from_str(manifest_yaml).unwrap();
        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        let models = registry.list_models();
        assert_eq!(models.len(), 2);

        let ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
        assert!(ids.contains(&"planner_1".to_string()));
        assert!(ids.contains(&"coder_1".to_string()));
    }

    /// Integration test: Backend instantiation and generation
    #[tokio::test]
    async fn test_backend_integration() {
        use std::path::PathBuf;

        let backend =
            LocalMistralBackend::from_model_path(PathBuf::from("/models/test.gguf"), "test_model");

        assert_eq!(backend.id(), "test_model");

        let request = ModelRequest::new(ModelRole::Planner, "code", "Test prompt");
        let response = backend.generate(request).await.unwrap();

        assert!(!response.text.is_empty());
        assert!(response.text.contains("STUB"));
    }

    /// Integration test: Engagement mode enforcement
    #[test]
    fn test_engagement_mode_enforcement() {
        let manifest = ModelManifest {
            models: vec![(
                "test".to_string(),
                ModelDefinition {
                    role: ModelRole::Planner,
                    domain: "code".to_string(),
                    display_name: "Test".to_string(),
                    description: "".to_string(),
                    source_url: "https://example.com/model.gguf".to_string(),
                    size_gb: 4.0,
                    quantization: "".to_string(),
                    checksum: "sha256:abc".to_string(),
                    backend: "local".to_string(),
                },
            )]
            .into_iter()
            .collect(),
            backends: Default::default(),
        };

        let temp_dir = TempDir::new().unwrap();
        let registry = ModelRegistry::from_manifest(manifest, temp_dir.path());

        // Minimum should work
        let result = registry.get_model(&ModelRole::Planner, "code", &EngagementMode::Minimum);
        assert!(result.is_ok());

        // Medium should fail
        let result = registry.get_model(&ModelRole::Planner, "code", &EngagementMode::Medium);
        assert!(result.is_err());

        // Maximum should fail
        let result = registry.get_model(&ModelRole::Planner, "code", &EngagementMode::Maximum);
        assert!(result.is_err());
    }
}
