/// Engagement mode handling and enforcement
///
/// This module manages engagement modes (Minimum, Medium, Maximum) and enforces
/// the local-only constraint in v1 while providing hooks for future remote model usage.
use bodhya_core::{EngagementMode, Error, Result};
use serde::{Deserialize, Serialize};

/// Engagement manager that enforces mode policies
#[derive(Clone, Debug)]
pub struct EngagementManager {
    /// Current engagement mode
    mode: EngagementMode,
    /// Whether to allow remote model calls (v1: always false)
    allow_remote: bool,
}

impl EngagementManager {
    /// Create a new engagement manager with the given mode
    pub fn new(mode: EngagementMode) -> Self {
        Self {
            mode,
            allow_remote: false, // v1 constraint: no remote calls
        }
    }

    /// Get the current engagement mode
    pub fn mode(&self) -> &EngagementMode {
        &self.mode
    }

    /// Check if remote model calls are allowed
    pub fn is_remote_allowed(&self) -> bool {
        self.allow_remote
    }

    /// Validate that an operation is allowed under the current engagement mode
    pub fn validate_operation(&self, operation: EngagementOperation) -> Result<()> {
        match operation {
            EngagementOperation::LocalModelCall => {
                // Local model calls always allowed
                Ok(())
            }
            EngagementOperation::RemoteModelCall => {
                // Remote calls only allowed if explicitly enabled
                if self.allow_remote {
                    Ok(())
                } else {
                    Err(Error::EngagementViolation(
                        "Remote model calls are not allowed in the current engagement mode (v1 = local-only)"
                            .to_string(),
                    ))
                }
            }
            EngagementOperation::RemoteFallback => {
                // Fallback to remote only in Medium or Maximum mode (and if enabled)
                match self.mode {
                    EngagementMode::Minimum => Err(Error::EngagementViolation(
                        "Remote fallback not allowed in Minimum engagement mode".to_string(),
                    )),
                    EngagementMode::Medium | EngagementMode::Maximum => {
                        if self.allow_remote {
                            Ok(())
                        } else {
                            Err(Error::EngagementViolation(
                                "Remote fallback requires remote access to be enabled".to_string(),
                            ))
                        }
                    }
                }
            }
        }
    }

    /// Get recommended strategy for the current mode
    pub fn get_strategy(&self) -> EngagementStrategy {
        match self.mode {
            EngagementMode::Minimum => EngagementStrategy {
                prefer_local: true,
                allow_remote_fallback: false,
                remote_for_complex: false,
            },
            EngagementMode::Medium => EngagementStrategy {
                prefer_local: true,
                allow_remote_fallback: true,
                remote_for_complex: false,
            },
            EngagementMode::Maximum => EngagementStrategy {
                prefer_local: false,
                allow_remote_fallback: true,
                remote_for_complex: true,
            },
        }
    }

    /// Log where remote escalation would be beneficial (design-only in v1)
    pub fn log_remote_opportunity(&self, reason: &str) {
        tracing::debug!(
            mode = ?self.mode,
            reason = reason,
            "Remote model escalation opportunity detected (not used in v1)"
        );
    }
}

impl Default for EngagementManager {
    fn default() -> Self {
        Self::new(EngagementMode::Minimum)
    }
}

/// Types of operations that need engagement validation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngagementOperation {
    /// Local model inference call
    LocalModelCall,
    /// Remote model inference call
    RemoteModelCall,
    /// Fallback to remote when local fails
    RemoteFallback,
}

/// Strategy based on engagement mode
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngagementStrategy {
    /// Prefer local models over remote
    pub prefer_local: bool,
    /// Allow fallback to remote if local fails
    pub allow_remote_fallback: bool,
    /// Use remote for complex tasks proactively
    pub remote_for_complex: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_engagement_manager() {
        let manager = EngagementManager::default();
        assert_eq!(*manager.mode(), EngagementMode::Minimum);
        assert!(!manager.is_remote_allowed());
    }

    #[test]
    fn test_create_with_mode() {
        let manager = EngagementManager::new(EngagementMode::Medium);
        assert_eq!(*manager.mode(), EngagementMode::Medium);
    }

    #[test]
    fn test_local_call_always_allowed() {
        let min_manager = EngagementManager::new(EngagementMode::Minimum);
        assert!(min_manager
            .validate_operation(EngagementOperation::LocalModelCall)
            .is_ok());

        let med_manager = EngagementManager::new(EngagementMode::Medium);
        assert!(med_manager
            .validate_operation(EngagementOperation::LocalModelCall)
            .is_ok());

        let max_manager = EngagementManager::new(EngagementMode::Maximum);
        assert!(max_manager
            .validate_operation(EngagementOperation::LocalModelCall)
            .is_ok());
    }

    #[test]
    fn test_remote_call_blocked_in_v1() {
        let manager = EngagementManager::new(EngagementMode::Minimum);
        let result = manager.validate_operation(EngagementOperation::RemoteModelCall);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::EngagementViolation(_)));
    }

    #[test]
    fn test_remote_fallback_blocked_in_minimum() {
        let manager = EngagementManager::new(EngagementMode::Minimum);
        let result = manager.validate_operation(EngagementOperation::RemoteFallback);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::EngagementViolation(_)));
        assert!(err.to_string().contains("Minimum engagement mode"));
    }

    #[test]
    fn test_remote_fallback_blocked_in_medium_without_access() {
        let manager = EngagementManager::new(EngagementMode::Medium);
        // Even in Medium mode, remote is not allowed because allow_remote is false
        let result = manager.validate_operation(EngagementOperation::RemoteFallback);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_strategy_minimum() {
        let manager = EngagementManager::new(EngagementMode::Minimum);
        let strategy = manager.get_strategy();

        assert!(strategy.prefer_local);
        assert!(!strategy.allow_remote_fallback);
        assert!(!strategy.remote_for_complex);
    }

    #[test]
    fn test_get_strategy_medium() {
        let manager = EngagementManager::new(EngagementMode::Medium);
        let strategy = manager.get_strategy();

        assert!(strategy.prefer_local);
        assert!(strategy.allow_remote_fallback);
        assert!(!strategy.remote_for_complex);
    }

    #[test]
    fn test_get_strategy_maximum() {
        let manager = EngagementManager::new(EngagementMode::Maximum);
        let strategy = manager.get_strategy();

        assert!(!strategy.prefer_local);
        assert!(strategy.allow_remote_fallback);
        assert!(strategy.remote_for_complex);
    }

    #[test]
    fn test_is_remote_allowed_v1() {
        // In v1, remote is never allowed regardless of mode
        let min = EngagementManager::new(EngagementMode::Minimum);
        assert!(!min.is_remote_allowed());

        let med = EngagementManager::new(EngagementMode::Medium);
        assert!(!med.is_remote_allowed());

        let max = EngagementManager::new(EngagementMode::Maximum);
        assert!(!max.is_remote_allowed());
    }

    #[test]
    fn test_log_remote_opportunity() {
        let manager = EngagementManager::new(EngagementMode::Minimum);
        // This should not panic and should log a debug message
        manager.log_remote_opportunity("Complex task detected");
    }

    #[test]
    fn test_engagement_operation_equality() {
        assert_eq!(
            EngagementOperation::LocalModelCall,
            EngagementOperation::LocalModelCall
        );
        assert_ne!(
            EngagementOperation::LocalModelCall,
            EngagementOperation::RemoteModelCall
        );
    }
}
