/// Configuration types for Bodhya platform
///
/// This module defines the configuration structures for the application,
/// agents, and models. Configurations are typically loaded from YAML files.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::model::{EngagementMode, ModelRole};

/// Main application configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Active profile (code, mail, full)
    #[serde(default = "default_profile")]
    pub profile: String,

    /// Engagement mode for remote models
    #[serde(default)]
    pub engagement_mode: EngagementMode,

    /// Configuration for each agent
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,

    /// Model configurations
    #[serde(default)]
    pub models: ModelConfigs,

    /// Paths configuration
    #[serde(default)]
    pub paths: PathsConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

fn default_profile() -> String {
    "full".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            profile: default_profile(),
            engagement_mode: EngagementMode::default(),
            agents: HashMap::new(),
            models: ModelConfigs::default(),
            paths: PathsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl AppConfig {
    /// Load configuration from a YAML file
    pub fn from_file(path: impl Into<PathBuf>) -> crate::errors::Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a YAML file
    pub fn to_file(&self, path: impl Into<PathBuf>) -> crate::errors::Result<()> {
        let path = path.into();
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get configuration for a specific agent
    pub fn get_agent_config(&self, agent_id: &str) -> Option<&AgentConfig> {
        self.agents.get(agent_id)
    }

    /// Check if an agent is enabled
    pub fn is_agent_enabled(&self, agent_id: &str) -> bool {
        self.agents
            .get(agent_id)
            .map(|cfg| cfg.enabled)
            .unwrap_or(false)
    }
}

/// Configuration for a specific agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Whether this agent is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Model assignments for different roles
    #[serde(default)]
    pub models: HashMap<ModelRole, String>,

    /// Agent-specific settings
    #[serde(default)]
    pub settings: serde_json::Value,
}

fn default_true() -> bool {
    true
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            models: HashMap::new(),
            settings: serde_json::Value::Null,
        }
    }
}

impl AgentConfig {
    /// Create a new agent configuration
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            models: HashMap::new(),
            settings: serde_json::Value::Null,
        }
    }

    /// Add a model assignment for a role
    pub fn with_model(mut self, role: ModelRole, model_id: impl Into<String>) -> Self {
        self.models.insert(role, model_id.into());
        self
    }

    /// Get the model ID for a role
    pub fn get_model(&self, role: &ModelRole) -> Option<&String> {
        self.models.get(role)
    }
}

/// Model configurations and manifest
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModelConfigs {
    /// Path to models.yaml manifest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<PathBuf>,

    /// Default model settings
    #[serde(default)]
    pub defaults: ModelDefaults,
}

/// Default model settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelDefaults {
    /// Default temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Default max tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> usize {
    2048
}

impl Default for ModelDefaults {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}

/// Paths configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Bodhya home directory
    #[serde(default = "default_bodhya_home")]
    pub home: PathBuf,

    /// Config directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<PathBuf>,

    /// Models directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<PathBuf>,

    /// Logs directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<PathBuf>,

    /// Cache directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<PathBuf>,
}

fn default_bodhya_home() -> PathBuf {
    home::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".bodhya")
}

impl Default for PathsConfig {
    fn default() -> Self {
        let home = default_bodhya_home();
        Self {
            home: home.clone(),
            config: Some(home.join("config")),
            models: Some(home.join("models")),
            logs: Some(home.join("logs")),
            cache: Some(home.join("cache")),
        }
    }
}

impl PathsConfig {
    /// Get the config directory, using default if not set
    pub fn config_dir(&self) -> PathBuf {
        self.config
            .clone()
            .unwrap_or_else(|| self.home.join("config"))
    }

    /// Get the models directory, using default if not set
    pub fn models_dir(&self) -> PathBuf {
        self.models
            .clone()
            .unwrap_or_else(|| self.home.join("models"))
    }

    /// Get the logs directory, using default if not set
    pub fn logs_dir(&self) -> PathBuf {
        self.logs.clone().unwrap_or_else(|| self.home.join("logs"))
    }

    /// Get the cache directory, using default if not set
    pub fn cache_dir(&self) -> PathBuf {
        self.cache
            .clone()
            .unwrap_or_else(|| self.home.join("cache"))
    }
}

/// Logging configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json, pretty, compact)
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "compact".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.profile, "full");
        assert_eq!(config.engagement_mode, EngagementMode::Minimum);
        assert!(config.agents.is_empty());
    }

    #[test]
    fn test_agent_config_creation() {
        let agent_config = AgentConfig::new(true)
            .with_model(ModelRole::Planner, "planner-model")
            .with_model(ModelRole::Coder, "coder-model");

        assert!(agent_config.enabled);
        assert_eq!(
            agent_config.get_model(&ModelRole::Planner),
            Some(&"planner-model".to_string())
        );
        assert_eq!(
            agent_config.get_model(&ModelRole::Coder),
            Some(&"coder-model".to_string())
        );
    }

    #[test]
    fn test_app_config_agent_enabled() {
        let mut config = AppConfig::default();
        config
            .agents
            .insert("code".to_string(), AgentConfig::new(true));
        config
            .agents
            .insert("mail".to_string(), AgentConfig::new(false));

        assert!(config.is_agent_enabled("code"));
        assert!(!config.is_agent_enabled("mail"));
        assert!(!config.is_agent_enabled("nonexistent"));
    }

    #[test]
    fn test_paths_config_default() {
        let paths = PathsConfig::default();
        assert!(paths.home.to_str().unwrap().contains(".bodhya"));
        assert!(paths.config_dir().to_str().unwrap().contains("config"));
        assert!(paths.models_dir().to_str().unwrap().contains("models"));
    }

    #[test]
    fn test_model_defaults() {
        let defaults = ModelDefaults::default();
        assert_eq!(defaults.temperature, 0.7);
        assert_eq!(defaults.max_tokens, 2048);
    }

    #[test]
    fn test_logging_config_default() {
        let logging = LoggingConfig::default();
        assert_eq!(logging.level, "info");
        assert_eq!(logging.format, "compact");
    }

    #[test]
    fn test_config_serialization() {
        let mut config = AppConfig {
            profile: "code".to_string(),
            ..Default::default()
        };
        config.agents.insert(
            "code".to_string(),
            AgentConfig::new(true).with_model(ModelRole::Planner, "test-model"),
        );

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("profile:"));
        assert!(yaml.contains("code"));

        let deserialized: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.profile, "code");
        assert!(deserialized.is_agent_enabled("code"));
    }

    #[test]
    fn test_config_file_io() {
        let config = AppConfig {
            profile: "test".to_string(),
            ..Default::default()
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save config
        config.to_file(path).unwrap();

        // Load config
        let loaded = AppConfig::from_file(path).unwrap();
        assert_eq!(loaded.profile, "test");
    }

    #[test]
    fn test_agent_config_get_model() {
        let agent_config = AgentConfig::new(true).with_model(ModelRole::Planner, "planner-1");

        assert_eq!(
            agent_config.get_model(&ModelRole::Planner),
            Some(&"planner-1".to_string())
        );
        assert_eq!(agent_config.get_model(&ModelRole::Coder), None);
    }
}
