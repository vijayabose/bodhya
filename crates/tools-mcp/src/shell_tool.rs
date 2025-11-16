/// Shell command execution tool
///
/// This module provides shell command execution as a Tool implementation.
use async_trait::async_trait;
use bodhya_core::{Result, Tool, ToolRequest, ToolResponse};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Shell execution tool for running commands
pub struct ShellTool {
    /// Working directory for command execution
    working_dir: Option<PathBuf>,
    /// Maximum execution time in seconds
    timeout_secs: u64,
}

impl ShellTool {
    /// Create a new shell tool with default settings
    pub fn new() -> Self {
        Self {
            working_dir: None,
            timeout_secs: 300, // 5 minutes default
        }
    }

    /// Create a shell tool with a specific working directory
    pub fn with_working_dir(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: Some(working_dir.into()),
            timeout_secs: 300,
        }
    }

    /// Set the timeout for command execution
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Execute a shell command
    async fn execute_command(&self, command: &str, args: Vec<String>) -> Result<ToolResponse> {
        let mut cmd = Command::new(command);

        // Set arguments
        if !args.is_empty() {
            cmd.args(&args);
        }

        // Set working directory if specified
        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        // Capture output
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Execute with timeout
        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs(self.timeout_secs),
            cmd.output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let success = output.status.success();
                let exit_code = output.status.code();

                let combined_output = if !stderr.is_empty() {
                    format!("{}\n{}", stdout, stderr)
                } else {
                    stdout.clone()
                };

                if success {
                    Ok(ToolResponse::success_with_output(
                        serde_json::json!({
                            "exit_code": exit_code,
                            "stdout": stdout,
                            "stderr": stderr,
                        }),
                        combined_output,
                    ))
                } else {
                    Ok(ToolResponse::failure(format!(
                        "Command failed with exit code {:?}: {}",
                        exit_code,
                        if !stderr.is_empty() { &stderr } else { &stdout }
                    )))
                }
            }
            Ok(Err(e)) => Ok(ToolResponse::failure(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Ok(ToolResponse::failure(format!(
                "Command timed out after {} seconds",
                self.timeout_secs
            ))),
        }
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn id(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute shell commands and capture their output"
    }

    fn supported_operations(&self) -> Vec<String> {
        vec!["exec".to_string(), "run".to_string()]
    }

    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse> {
        match request.operation.as_str() {
            "exec" | "run" => {
                let command = request.params["command"].as_str().ok_or_else(|| {
                    bodhya_core::Error::Tool("Missing 'command' parameter".to_string())
                })?;

                // Parse arguments
                let args = if let Some(args_value) = request.params.get("args") {
                    if let Some(args_array) = args_value.as_array() {
                        args_array
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    } else if let Some(args_str) = args_value.as_str() {
                        // Parse space-separated string
                        shell_words::split(args_str).map_err(|e| {
                            bodhya_core::Error::Tool(format!("Failed to parse args: {}", e))
                        })?
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                self.execute_command(command, args).await
            }
            _ => Ok(ToolResponse::failure(format!(
                "Unsupported operation: {}",
                request.operation
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_shell_tool_creation() {
        let tool = ShellTool::new();
        assert_eq!(tool.id(), "shell");
        assert!(tool.description().contains("shell"));
    }

    #[tokio::test]
    async fn test_shell_tool_supported_operations() {
        let tool = ShellTool::new();
        let ops = tool.supported_operations();

        assert!(ops.contains(&"exec".to_string()));
        assert!(ops.contains(&"run".to_string()));
    }

    #[tokio::test]
    async fn test_execute_echo_command() {
        let tool = ShellTool::new();

        let req = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": "echo",
                "args": ["Hello", "World"]
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(resp.success);
        assert!(resp.data["stdout"]
            .as_str()
            .unwrap()
            .contains("Hello World"));
    }

    #[tokio::test]
    async fn test_execute_with_working_dir() {
        let temp_dir = TempDir::new().unwrap();
        let tool = ShellTool::with_working_dir(temp_dir.path());

        // Create a test file in the temp directory
        std::fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

        let req = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": "ls",
                "args": []
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(resp.success);
        assert!(resp.data["stdout"].as_str().unwrap().contains("test.txt"));
    }

    #[tokio::test]
    async fn test_execute_failing_command() {
        let tool = ShellTool::new();

        let req = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": "false"
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(!resp.success);
        assert!(resp.error.is_some());
    }

    #[tokio::test]
    async fn test_execute_nonexistent_command() {
        let tool = ShellTool::new();

        let req = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": "nonexistent_command_12345"
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(!resp.success);
    }

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let tool = ShellTool::new().with_timeout(1);

        // This command should timeout (tries to sleep for 10 seconds)
        let req = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": "sleep",
                "args": ["10"]
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn test_unsupported_operation() {
        let tool = ShellTool::new();

        let req = ToolRequest::new(
            "shell",
            "invalid",
            serde_json::json!({
                "command": "echo"
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Unsupported operation"));
    }

    #[tokio::test]
    async fn test_missing_command_parameter() {
        let tool = ShellTool::new();

        let req = ToolRequest::new("shell", "exec", serde_json::json!({}));

        let result = tool.execute(req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_captures_stderr() {
        let tool = ShellTool::new();

        // Use a command that writes to stderr (e.g., ls on a non-existent file)
        let req = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": "ls",
                "args": ["/nonexistent_path_12345"]
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(!resp.success);
        let stderr = resp.data["stderr"].as_str().unwrap_or("");
        assert!(!stderr.is_empty() || resp.error.is_some());
    }

    #[tokio::test]
    async fn test_run_operation_alias() {
        let tool = ShellTool::new();

        let req = ToolRequest::new(
            "shell",
            "run",
            serde_json::json!({
                "command": "echo",
                "args": ["test"]
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(resp.success);
    }
}
