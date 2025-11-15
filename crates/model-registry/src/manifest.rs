/// Model manifest parsing and validation
///
/// This module handles parsing and validation of the models.yaml manifest file,
/// which defines available models, their roles, and download sources.
use bodhya_core::{Error, ModelRole, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Model manifest loaded from models.yaml
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelManifest {
    /// Map of model ID to model definition
    pub models: HashMap<String, ModelDefinition>,
    /// Backend configurations
    #[serde(default)]
    pub backends: HashMap<String, BackendConfig>,
}

impl ModelManifest {
    /// Load manifest from a YAML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            Error::Config(format!(
                "Failed to read manifest file {}: {}",
                path.display(),
                e
            ))
        })?;

        let manifest: ModelManifest = serde_yaml::from_str(&content).map_err(|e| {
            Error::Config(format!(
                "Failed to parse manifest file {}: {}",
                path.display(),
                e
            ))
        })?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        if self.models.is_empty() {
            return Err(Error::Config("Manifest contains no models".to_string()));
        }

        for (id, model) in &self.models {
            model.validate(id)?;
        }

        Ok(())
    }

    /// Get a model definition by ID
    pub fn get_model(&self, id: &str) -> Option<&ModelDefinition> {
        self.models.get(id)
    }

    /// Find models by role and domain
    pub fn find_models(&self, role: &ModelRole, domain: &str) -> Vec<(&String, &ModelDefinition)> {
        self.models
            .iter()
            .filter(|(_, def)| def.role == *role && def.domain.eq_ignore_ascii_case(domain))
            .collect()
    }

    /// Get all model IDs
    pub fn model_ids(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }
}

/// Definition of a single model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelDefinition {
    /// Model role (planner, coder, reviewer, etc.)
    pub role: ModelRole,
    /// Domain (code, mail, general, etc.)
    pub domain: String,
    /// Human-readable display name
    pub display_name: String,
    /// Model description
    #[serde(default)]
    pub description: String,
    /// URL to download the model
    pub source_url: String,
    /// Model size in GB
    pub size_gb: f64,
    /// Quantization level (e.g., "Q4_K_M")
    #[serde(default)]
    pub quantization: String,
    /// Checksum for verification (format: "sha256:...")
    pub checksum: String,
    /// Backend type (local, remote)
    pub backend: String,
}

impl ModelDefinition {
    /// Validate this model definition
    pub fn validate(&self, id: &str) -> Result<()> {
        if self.display_name.is_empty() {
            return Err(Error::Config(format!(
                "Model '{}' has empty display_name",
                id
            )));
        }

        if self.source_url.is_empty() {
            return Err(Error::Config(format!(
                "Model '{}' has empty source_url",
                id
            )));
        }

        if self.size_gb <= 0.0 {
            return Err(Error::Config(format!(
                "Model '{}' has invalid size_gb: {}",
                id, self.size_gb
            )));
        }

        if !self.checksum.starts_with("sha256:") {
            return Err(Error::Config(format!(
                "Model '{}' has invalid checksum format (must start with 'sha256:')",
                id
            )));
        }

        Ok(())
    }

    /// Get the expected file size in bytes
    pub fn size_bytes(&self) -> u64 {
        (self.size_gb * 1_000_000_000.0) as u64
    }

    /// Extract the checksum hash (without the "sha256:" prefix)
    pub fn checksum_hash(&self) -> &str {
        self.checksum
            .strip_prefix("sha256:")
            .unwrap_or(&self.checksum)
    }
}

/// Backend configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend type (mistral_rs, openai_compatible, etc.)
    #[serde(rename = "type")]
    pub backend_type: String,
    /// Whether this backend is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Backend-specific configuration
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_manifest() -> String {
        r#"
models:
  test_planner:
    role: planner
    domain: code
    display_name: "Test Planner"
    description: "Test model for planning"
    source_url: "https://example.com/model.gguf"
    size_gb: 4.0
    quantization: "Q4_K_M"
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

backends:
  local:
    type: mistral_rs
    enabled: true
    config:
      device: cpu
"#
        .to_string()
    }

    #[test]
    fn test_parse_manifest() {
        let yaml = create_test_manifest();
        let manifest: ModelManifest = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(manifest.models.len(), 2);
        assert_eq!(manifest.backends.len(), 1);
    }

    #[test]
    fn test_manifest_from_file() {
        let yaml = create_test_manifest();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", yaml).unwrap();

        let manifest = ModelManifest::from_file(temp_file.path()).unwrap();
        assert_eq!(manifest.models.len(), 2);
    }

    #[test]
    fn test_manifest_validation() {
        let yaml = create_test_manifest();
        let manifest: ModelManifest = serde_yaml::from_str(&yaml).unwrap();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_empty_manifest_fails_validation() {
        let yaml = "models: {}\nbackends: {}";
        let manifest: ModelManifest = serde_yaml::from_str(yaml).unwrap();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_get_model() {
        let yaml = create_test_manifest();
        let manifest: ModelManifest = serde_yaml::from_str(&yaml).unwrap();

        let model = manifest.get_model("test_planner").unwrap();
        assert_eq!(model.display_name, "Test Planner");
        assert_eq!(model.role, ModelRole::Planner);

        assert!(manifest.get_model("nonexistent").is_none());
    }

    #[test]
    fn test_find_models_by_role_and_domain() {
        let yaml = create_test_manifest();
        let manifest: ModelManifest = serde_yaml::from_str(&yaml).unwrap();

        let planners = manifest.find_models(&ModelRole::Planner, "code");
        assert_eq!(planners.len(), 1);
        assert_eq!(planners[0].0, "test_planner");

        let coders = manifest.find_models(&ModelRole::Coder, "code");
        assert_eq!(coders.len(), 1);

        let writers = manifest.find_models(&ModelRole::Writer, "mail");
        assert_eq!(writers.len(), 0);
    }

    #[test]
    fn test_model_ids() {
        let yaml = create_test_manifest();
        let manifest: ModelManifest = serde_yaml::from_str(&yaml).unwrap();

        let ids = manifest.model_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"test_planner".to_string()));
        assert!(ids.contains(&"test_coder".to_string()));
    }

    #[test]
    fn test_model_definition_validation() {
        let model = ModelDefinition {
            role: ModelRole::Planner,
            domain: "code".to_string(),
            display_name: "Test".to_string(),
            description: "Test model".to_string(),
            source_url: "https://example.com/model.gguf".to_string(),
            size_gb: 4.0,
            quantization: "Q4_K_M".to_string(),
            checksum: "sha256:abc123".to_string(),
            backend: "local".to_string(),
        };

        assert!(model.validate("test_id").is_ok());
    }

    #[test]
    fn test_model_validation_empty_display_name() {
        let model = ModelDefinition {
            role: ModelRole::Planner,
            domain: "code".to_string(),
            display_name: "".to_string(),
            description: "".to_string(),
            source_url: "https://example.com/model.gguf".to_string(),
            size_gb: 4.0,
            quantization: "".to_string(),
            checksum: "sha256:abc".to_string(),
            backend: "local".to_string(),
        };

        assert!(model.validate("test").is_err());
    }

    #[test]
    fn test_model_validation_invalid_checksum() {
        let model = ModelDefinition {
            role: ModelRole::Planner,
            domain: "code".to_string(),
            display_name: "Test".to_string(),
            description: "".to_string(),
            source_url: "https://example.com/model.gguf".to_string(),
            size_gb: 4.0,
            quantization: "".to_string(),
            checksum: "invalid_checksum".to_string(),
            backend: "local".to_string(),
        };

        assert!(model.validate("test").is_err());
    }

    #[test]
    fn test_model_size_bytes() {
        let model = ModelDefinition {
            role: ModelRole::Planner,
            domain: "code".to_string(),
            display_name: "Test".to_string(),
            description: "".to_string(),
            source_url: "https://example.com/model.gguf".to_string(),
            size_gb: 4.4,
            quantization: "".to_string(),
            checksum: "sha256:abc".to_string(),
            backend: "local".to_string(),
        };

        assert_eq!(model.size_bytes(), 4_400_000_000);
    }

    #[test]
    fn test_checksum_hash() {
        let model = ModelDefinition {
            role: ModelRole::Planner,
            domain: "code".to_string(),
            display_name: "Test".to_string(),
            description: "".to_string(),
            source_url: "https://example.com/model.gguf".to_string(),
            size_gb: 4.0,
            quantization: "".to_string(),
            checksum: "sha256:abc123def456".to_string(),
            backend: "local".to_string(),
        };

        assert_eq!(model.checksum_hash(), "abc123def456");
    }

    #[test]
    fn test_backend_config() {
        let yaml = create_test_manifest();
        let manifest: ModelManifest = serde_yaml::from_str(&yaml).unwrap();

        let backend = manifest.backends.get("local").unwrap();
        assert_eq!(backend.backend_type, "mistral_rs");
        assert!(backend.enabled);
    }
}
