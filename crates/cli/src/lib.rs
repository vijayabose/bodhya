/// Bodhya CLI Library
///
/// This module provides the command-line interface for Bodhya,
/// including initialization, model management, and task execution.
pub mod config_templates;
pub mod history_cmd;
pub mod init_cmd;
pub mod models_cmd;
pub mod run_cmd;
pub mod utils;

pub use config_templates::ConfigTemplate;
