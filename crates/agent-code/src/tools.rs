/// CodeAgent tool wrapper for file operations and command execution
///
/// This module provides a high-level, agent-friendly interface to the tool system,
/// wrapping the low-level ToolRegistry with methods tailored for code generation tasks.
use bodhya_core::{Result, ToolRequest};
use bodhya_tools_mcp::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Command execution output
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandOutput {
    /// Command exit code
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Whether command succeeded
    pub success: bool,
}

impl CommandOutput {
    /// Create a successful command output
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            exit_code: Some(0),
            stdout: stdout.into(),
            stderr: String::new(),
            success: true,
        }
    }

    /// Create a failed command output
    pub fn failure(exit_code: Option<i32>, stderr: impl Into<String>) -> Self {
        Self {
            exit_code,
            stdout: String::new(),
            stderr: stderr.into(),
            success: false,
        }
    }
}

/// Execution statistics for tracking tool usage
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Number of files read
    pub files_read: usize,
    /// Number of files written
    pub files_written: usize,
    /// Number of files listed
    pub files_listed: usize,
    /// Number of commands executed
    pub commands_executed: usize,
    /// Total bytes read
    pub bytes_read: usize,
    /// Total bytes written
    pub bytes_written: usize,
}

impl ExecutionStats {
    /// Create new execution statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all statistics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// High-level tool wrapper for CodeAgent
///
/// Provides convenient methods for file operations and command execution,
/// with execution tracking and working directory management.
pub struct CodeAgentTools {
    /// Tool registry for low-level operations
    registry: Arc<ToolRegistry>,
    /// Working directory for file operations
    working_dir: PathBuf,
    /// Execution statistics (thread-safe)
    stats: Arc<Mutex<ExecutionStats>>,
}

impl CodeAgentTools {
    /// Create a new CodeAgentTools instance
    pub fn new(registry: Arc<ToolRegistry>, working_dir: impl Into<PathBuf>) -> Self {
        Self {
            registry,
            working_dir: working_dir.into(),
            stats: Arc::new(Mutex::new(ExecutionStats::new())),
        }
    }

    /// Get current execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.stats.lock().await.clone()
    }

    /// Reset execution statistics
    pub async fn reset_stats(&self) {
        self.stats.lock().await.reset();
    }

    /// Resolve a path relative to the working directory
    fn resolve_path(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    /// Read a file and return its contents
    pub async fn read_file(&self, path: impl AsRef<Path>) -> Result<String> {
        let resolved = self.resolve_path(path);
        let path_str = resolved
            .to_str()
            .ok_or_else(|| bodhya_core::Error::Tool("Invalid path encoding".to_string()))?;

        let request = ToolRequest::new(
            "filesystem",
            "read",
            serde_json::json!({
                "path": path_str
            }),
        );

        let response = self.registry.execute(request).await?;

        if response.success {
            let content = response.data["content"].as_str().unwrap_or("").to_string();

            // Update stats
            let mut stats = self.stats.lock().await;
            stats.files_read += 1;
            stats.bytes_read += content.len();

            Ok(content)
        } else {
            Err(bodhya_core::Error::Tool(
                response
                    .error
                    .unwrap_or_else(|| "Failed to read file".to_string()),
            ))
        }
    }

    /// Write content to a file
    pub async fn write_file(&self, path: impl AsRef<Path>, content: &str) -> Result<()> {
        let resolved = self.resolve_path(path);
        let path_str = resolved
            .to_str()
            .ok_or_else(|| bodhya_core::Error::Tool("Invalid path encoding".to_string()))?;

        let request = ToolRequest::new(
            "filesystem",
            "write",
            serde_json::json!({
                "path": path_str,
                "content": content
            }),
        );

        let response = self.registry.execute(request).await?;

        if response.success {
            // Update stats
            let mut stats = self.stats.lock().await;
            stats.files_written += 1;
            stats.bytes_written += content.len();

            Ok(())
        } else {
            Err(bodhya_core::Error::Tool(
                response
                    .error
                    .unwrap_or_else(|| "Failed to write file".to_string()),
            ))
        }
    }

    /// List files in a directory
    pub async fn list_files(&self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let resolved = self.resolve_path(path);
        let path_str = resolved
            .to_str()
            .ok_or_else(|| bodhya_core::Error::Tool("Invalid path encoding".to_string()))?;

        let request = ToolRequest::new(
            "filesystem",
            "list",
            serde_json::json!({
                "path": path_str
            }),
        );

        let response = self.registry.execute(request).await?;

        if response.success {
            let mut results = Vec::new();

            // Get files
            if let Some(files) = response.data["files"].as_array() {
                for file in files {
                    if let Some(name) = file.as_str() {
                        results.push(resolved.join(name));
                    }
                }
            }

            // Get directories
            if let Some(dirs) = response.data["directories"].as_array() {
                for dir in dirs {
                    if let Some(name) = dir.as_str() {
                        results.push(resolved.join(name));
                    }
                }
            }

            // Update stats
            let mut stats = self.stats.lock().await;
            stats.files_listed += results.len();

            Ok(results)
        } else {
            Err(bodhya_core::Error::Tool(
                response
                    .error
                    .unwrap_or_else(|| "Failed to list directory".to_string()),
            ))
        }
    }

    /// Check if a file or directory exists
    pub async fn file_exists(&self, path: impl AsRef<Path>) -> Result<bool> {
        let resolved = self.resolve_path(path);
        let path_str = resolved
            .to_str()
            .ok_or_else(|| bodhya_core::Error::Tool("Invalid path encoding".to_string()))?;

        let request = ToolRequest::new(
            "filesystem",
            "exists",
            serde_json::json!({
                "path": path_str
            }),
        );

        let response = self.registry.execute(request).await?;

        if response.success {
            Ok(response.data["exists"].as_bool().unwrap_or(false))
        } else {
            Err(bodhya_core::Error::Tool(
                response
                    .error
                    .unwrap_or_else(|| "Failed to check existence".to_string()),
            ))
        }
    }

    /// Execute a shell command
    pub async fn run_command(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        let request = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": command,
                "args": args
            }),
        );

        let response = self.registry.execute(request).await?;

        // Update stats
        let mut stats = self.stats.lock().await;
        stats.commands_executed += 1;

        if response.success {
            Ok(CommandOutput {
                exit_code: response.data["exit_code"].as_i64().map(|v| v as i32),
                stdout: response.data["stdout"].as_str().unwrap_or("").to_string(),
                stderr: response.data["stderr"].as_str().unwrap_or("").to_string(),
                success: true,
            })
        } else {
            // Even on failure, try to extract output
            Ok(CommandOutput {
                exit_code: response.data["exit_code"].as_i64().map(|v| v as i32),
                stdout: response.data["stdout"].as_str().unwrap_or("").to_string(),
                stderr: response
                    .error
                    .unwrap_or_else(|| "Command failed".to_string()),
                success: false,
            })
        }
    }

    /// Execute a cargo command (convenience wrapper)
    pub async fn run_cargo(&self, subcommand: &str, args: &[&str]) -> Result<CommandOutput> {
        let mut cargo_args = vec![subcommand];
        cargo_args.extend_from_slice(args);
        self.run_command("cargo", &cargo_args).await
    }

    /// Edit a file with the specified operation
    ///
    /// # Arguments
    /// * `path` - Path to the file to edit (relative or absolute)
    /// * `operation` - The edit operation to perform (replace, insert, delete, patch)
    /// * `dry_run` - If true, validate the operation without applying changes
    ///
    /// # Returns
    /// A tuple of (success, modified_content, changes_made, error_message)
    pub async fn edit_file(
        &self,
        path: impl AsRef<Path>,
        operation: serde_json::Value,
        dry_run: bool,
    ) -> Result<(bool, String, usize, Option<String>)> {
        let resolved = self.resolve_path(path);
        let path_str = resolved
            .to_str()
            .ok_or_else(|| bodhya_core::Error::Tool("Invalid path encoding".to_string()))?;

        let request = ToolRequest::new(
            "edit",
            "edit",
            serde_json::json!({
                "path": path_str,
                "operation": operation,
                "dry_run": dry_run
            }),
        );

        let response = self.registry.execute(request).await?;

        if response.success {
            let modified_content = response.data["modified_content"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let changes_made = response.data["changes_made"].as_u64().unwrap_or(0) as usize;
            let error = response.data["error"].as_str().map(|s| s.to_string());

            Ok((true, modified_content, changes_made, error))
        } else {
            let error_msg = response
                .error
                .unwrap_or_else(|| "Failed to edit file".to_string());
            Err(bodhya_core::Error::Tool(error_msg))
        }
    }

    /// Search for code patterns in files
    ///
    /// # Arguments
    /// * `path` - Path to search in (file or directory)
    /// * `pattern` - Regular expression pattern to search for
    /// * `recursive` - Whether to search recursively in subdirectories
    /// * `case_sensitive` - Whether the search should be case-sensitive
    /// * `file_pattern` - Optional file pattern to filter (e.g., "*.rs")
    /// * `context_lines` - Number of context lines to show before/after matches
    ///
    /// # Returns
    /// A tuple of (success, matches, total_count, files_searched, error_message)
    pub async fn search_code(
        &self,
        path: impl AsRef<Path>,
        pattern: &str,
        recursive: bool,
        case_sensitive: bool,
        file_pattern: Option<&str>,
        context_lines: usize,
    ) -> Result<(bool, Vec<serde_json::Value>, usize, usize, Option<String>)> {
        let resolved = self.resolve_path(path);
        let path_str = resolved
            .to_str()
            .ok_or_else(|| bodhya_core::Error::Tool("Invalid path encoding".to_string()))?;

        let request = ToolRequest::new(
            "search",
            "grep",
            serde_json::json!({
                "path": path_str,
                "pattern": pattern,
                "recursive": recursive,
                "case_sensitive": case_sensitive,
                "file_pattern": file_pattern,
                "context_lines": context_lines
            }),
        );

        let response = self.registry.execute(request).await?;

        if response.success {
            let matches = response.data["matches"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let total_matches = response.data["total_matches"].as_u64().unwrap_or(0) as usize;
            let files_searched = response.data["files_searched"].as_u64().unwrap_or(0) as usize;
            let error = response.data["error"].as_str().map(|s| s.to_string());

            Ok((true, matches, total_matches, files_searched, error))
        } else {
            let error_msg = response
                .error
                .unwrap_or_else(|| "Failed to search code".to_string());
            Err(bodhya_core::Error::Tool(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_tools(temp_dir: &TempDir) -> CodeAgentTools {
        let registry = Arc::new(ToolRegistry::with_defaults());
        CodeAgentTools::new(registry, temp_dir.path())
    }

    #[test]
    fn test_command_output_success() {
        let output = CommandOutput::success("test output");
        assert!(output.success);
        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.stdout, "test output");
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_command_output_failure() {
        let output = CommandOutput::failure(Some(1), "error message");
        assert!(!output.success);
        assert_eq!(output.exit_code, Some(1));
        assert_eq!(output.stderr, "error message");
    }

    #[test]
    fn test_execution_stats_new() {
        let stats = ExecutionStats::new();
        assert_eq!(stats.files_read, 0);
        assert_eq!(stats.files_written, 0);
        assert_eq!(stats.commands_executed, 0);
    }

    #[test]
    fn test_execution_stats_reset() {
        let mut stats = ExecutionStats::new();
        stats.files_read = 5;
        stats.files_written = 3;
        stats.reset();
        assert_eq!(stats.files_read, 0);
        assert_eq!(stats.files_written, 0);
    }

    #[tokio::test]
    async fn test_code_agent_tools_creation() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        let stats = tools.get_stats().await;
        assert_eq!(stats.files_read, 0);
        assert_eq!(stats.files_written, 0);
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        let test_content = "Hello, World!";
        tools.write_file("test.txt", test_content).await.unwrap();

        let content = tools.read_file("test.txt").await.unwrap();
        assert_eq!(content, test_content);

        let stats = tools.get_stats().await;
        assert_eq!(stats.files_written, 1);
        assert_eq!(stats.files_read, 1);
        assert_eq!(stats.bytes_written, test_content.len());
        assert_eq!(stats.bytes_read, test_content.len());
    }

    #[tokio::test]
    async fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        // File doesn't exist yet
        let exists = tools.file_exists("test.txt").await.unwrap();
        assert!(!exists);

        // Create file
        tools.write_file("test.txt", "content").await.unwrap();

        // File should exist now
        let exists = tools.file_exists("test.txt").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_list_files() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        // Create some files
        tools.write_file("file1.txt", "content1").await.unwrap();
        tools.write_file("file2.txt", "content2").await.unwrap();

        // List files
        let files = tools.list_files(".").await.unwrap();
        assert_eq!(files.len(), 2);

        let stats = tools.get_stats().await;
        assert_eq!(stats.files_listed, 2);
    }

    #[tokio::test]
    async fn test_run_command() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        let output = tools.run_command("echo", &["test"]).await.unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("test"));

        let stats = tools.get_stats().await;
        assert_eq!(stats.commands_executed, 1);
    }

    #[tokio::test]
    async fn test_run_cargo() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        let output = tools.run_cargo("--version", &[]).await.unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("cargo"));
    }

    #[tokio::test]
    async fn test_resolve_path() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        let relative = tools.resolve_path("test.txt");
        assert_eq!(relative, temp_dir.path().join("test.txt"));

        let absolute = tools.resolve_path("/tmp/test.txt");
        assert_eq!(absolute, PathBuf::from("/tmp/test.txt"));
    }

    #[tokio::test]
    async fn test_stats_reset() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        tools.write_file("test.txt", "content").await.unwrap();
        let stats = tools.get_stats().await;
        assert_eq!(stats.files_written, 1);

        tools.reset_stats().await;
        let stats = tools.get_stats().await;
        assert_eq!(stats.files_written, 0);
    }

    #[tokio::test]
    async fn test_nested_directory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let tools = create_test_tools(&temp_dir);

        // Write to nested path (FilesystemTool should create parent dirs)
        tools
            .write_file("nested/dir/test.txt", "nested content")
            .await
            .unwrap();

        // Read it back
        let content = tools.read_file("nested/dir/test.txt").await.unwrap();
        assert_eq!(content, "nested content");

        // Check it exists
        let exists = tools.file_exists("nested/dir/test.txt").await.unwrap();
        assert!(exists);
    }
}
