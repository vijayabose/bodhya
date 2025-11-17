/// Configuration templates for different profiles
///
/// This module provides pre-configured templates for different use cases:
/// - "code": Code generation only
/// - "mail": Email writing only
/// - "full": All domains enabled
use bodhya_core::{AgentConfig, AppConfig, EngagementMode, LoggingConfig, PathsConfig};
use std::collections::HashMap;

/// Profile types supported by Bodhya
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Code,
    Mail,
    Full,
}

impl Profile {
    /// Parse profile from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "code" => Some(Profile::Code),
            "mail" => Some(Profile::Mail),
            "full" => Some(Profile::Full),
            _ => None,
        }
    }

    /// Get profile name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Profile::Code => "code",
            Profile::Mail => "mail",
            Profile::Full => "full",
        }
    }

    /// Get profile description
    pub fn description(&self) -> &'static str {
        match self {
            Profile::Code => "Code generation and development tasks",
            Profile::Mail => "Email drafting and refinement",
            Profile::Full => "All agents enabled (code, mail, and future domains)",
        }
    }
}

/// Configuration template provider
pub struct ConfigTemplate;

impl ConfigTemplate {
    /// Generate configuration for a specific profile
    pub fn for_profile(profile: Profile) -> AppConfig {
        let mut agents = HashMap::new();

        match profile {
            Profile::Code => {
                agents.insert(
                    "code".to_string(),
                    AgentConfig {
                        enabled: true,
                        models: HashMap::new(),
                        settings: serde_json::Value::Null,
                    },
                );
            }
            Profile::Mail => {
                agents.insert(
                    "mail".to_string(),
                    AgentConfig {
                        enabled: true,
                        models: HashMap::new(),
                        settings: serde_json::Value::Null,
                    },
                );
            }
            Profile::Full => {
                agents.insert(
                    "code".to_string(),
                    AgentConfig {
                        enabled: true,
                        models: HashMap::new(),
                        settings: serde_json::Value::Null,
                    },
                );
                agents.insert(
                    "mail".to_string(),
                    AgentConfig {
                        enabled: true,
                        models: HashMap::new(),
                        settings: serde_json::Value::Null,
                    },
                );
            }
        }

        AppConfig {
            profile: profile.as_str().to_string(),
            engagement_mode: EngagementMode::Minimum,
            agents,
            models: Default::default(),
            tools: Default::default(),
            paths: PathsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }

    /// Get all available profiles
    pub fn all_profiles() -> Vec<Profile> {
        vec![Profile::Code, Profile::Mail, Profile::Full]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_parse() {
        assert_eq!(Profile::parse("code"), Some(Profile::Code));
        assert_eq!(Profile::parse("mail"), Some(Profile::Mail));
        assert_eq!(Profile::parse("full"), Some(Profile::Full));
        assert_eq!(Profile::parse("CODE"), Some(Profile::Code));
        assert_eq!(Profile::parse("invalid"), None);
    }

    #[test]
    fn test_profile_as_str() {
        assert_eq!(Profile::Code.as_str(), "code");
        assert_eq!(Profile::Mail.as_str(), "mail");
        assert_eq!(Profile::Full.as_str(), "full");
    }

    #[test]
    fn test_profile_description() {
        assert!(Profile::Code.description().contains("Code"));
        assert!(Profile::Mail.description().contains("Email"));
        assert!(Profile::Full.description().contains("All"));
    }

    #[test]
    fn test_code_profile_config() {
        let config = ConfigTemplate::for_profile(Profile::Code);

        assert_eq!(config.profile, "code");
        assert_eq!(config.engagement_mode, EngagementMode::Minimum);
        assert_eq!(config.agents.len(), 1);
        assert!(config.agents.contains_key("code"));
        assert!(config.agents.get("code").unwrap().enabled);
    }

    #[test]
    fn test_mail_profile_config() {
        let config = ConfigTemplate::for_profile(Profile::Mail);

        assert_eq!(config.profile, "mail");
        assert_eq!(config.agents.len(), 1);
        assert!(config.agents.contains_key("mail"));
    }

    #[test]
    fn test_full_profile_config() {
        let config = ConfigTemplate::for_profile(Profile::Full);

        assert_eq!(config.profile, "full");
        assert_eq!(config.agents.len(), 2);
        assert!(config.agents.contains_key("code"));
        assert!(config.agents.contains_key("mail"));
        assert!(config.agents.get("code").unwrap().enabled);
        assert!(config.agents.get("mail").unwrap().enabled);
    }

    #[test]
    fn test_all_profiles() {
        let profiles = ConfigTemplate::all_profiles();
        assert_eq!(profiles.len(), 3);
        assert!(profiles.contains(&Profile::Code));
        assert!(profiles.contains(&Profile::Mail));
        assert!(profiles.contains(&Profile::Full));
    }

    #[test]
    fn test_profiles_have_minimum_engagement() {
        for profile in ConfigTemplate::all_profiles() {
            let config = ConfigTemplate::for_profile(profile);
            assert_eq!(config.engagement_mode, EngagementMode::Minimum);
        }
    }
}
