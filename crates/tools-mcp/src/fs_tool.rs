/// Filesystem tool for file operations
///
/// This module provides filesystem operations (read, write, list) as a Tool implementation.
use async_trait::async_trait;
use bodhya_core::{Result, Tool, ToolRequest, ToolResponse};
use std::path::PathBuf;

/// Filesystem tool for file operations
pub struct FilesystemTool {
    /// Base directory for sandboxing (optional)
    base_dir: Option<PathBuf>,
}

impl FilesystemTool {
    /// Create a new filesystem tool
    pub fn new() -> Self {
        Self { base_dir: None }
    }

    /// Create a filesystem tool with a base directory for sandboxing
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
        }
    }

    /// Resolve a path relative to the base directory (if set)
    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let path = PathBuf::from(path);

        if let Some(base) = &self.base_dir {
            // Resolve relative to base directory
            let resolved = if path.is_absolute() {
                path
            } else {
                base.join(&path)
            };

            // Ensure the resolved path is within base_dir (security check)
            let canonical_base = base.canonicalize().map_err(|e| {
                bodhya_core::Error::Tool(format!("Failed to canonicalize base dir: {}", e))
            })?;

            // For non-existent paths, we can't canonicalize, so check the parent
            let canonical_resolved = if resolved.exists() {
                resolved.canonicalize().map_err(|e| {
                    bodhya_core::Error::Tool(format!("Failed to canonicalize path: {}", e))
                })?
            } else {
                // Check parent directory if file doesn't exist
                if let Some(parent) = resolved.parent() {
                    if parent.exists() {
                        parent.canonicalize().map_err(|e| {
                            bodhya_core::Error::Tool(format!(
                                "Failed to canonicalize parent: {}",
                                e
                            ))
                        })?;
                    }
                }
                resolved
            };

            // Security check: ensure path is within base_dir
            if !canonical_resolved.starts_with(&canonical_base) {
                return Err(bodhya_core::Error::Tool(format!(
                    "Path '{}' is outside base directory",
                    canonical_resolved.display()
                )));
            }

            Ok(canonical_resolved)
        } else {
            Ok(path)
        }
    }

    /// Read a file
    async fn read_file(&self, path: &str) -> Result<ToolResponse> {
        let resolved = self.resolve_path(path)?;

        match tokio::fs::read_to_string(&resolved).await {
            Ok(content) => Ok(ToolResponse::success(serde_json::json!({
                "path": path,
                "content": content,
                "size": content.len()
            }))),
            Err(e) => Ok(ToolResponse::failure(format!(
                "Failed to read file '{}': {}",
                path, e
            ))),
        }
    }

    /// Write to a file
    async fn write_file(&self, path: &str, content: &str) -> Result<ToolResponse> {
        let resolved = self.resolve_path(path)?;

        // Create parent directories if they don't exist
        if let Some(parent) = resolved.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return Ok(ToolResponse::failure(format!(
                    "Failed to create parent directories: {}",
                    e
                )));
            }
        }

        match tokio::fs::write(&resolved, content).await {
            Ok(_) => Ok(ToolResponse::success(serde_json::json!({
                "path": path,
                "size": content.len(),
                "written": true
            }))),
            Err(e) => Ok(ToolResponse::failure(format!(
                "Failed to write file '{}': {}",
                path, e
            ))),
        }
    }

    /// List files in a directory
    async fn list_dir(&self, path: &str) -> Result<ToolResponse> {
        let resolved = self.resolve_path(path)?;

        match tokio::fs::read_dir(&resolved).await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                let mut dirs = Vec::new();

                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(name) = entry.file_name().into_string() {
                        if let Ok(file_type) = entry.file_type().await {
                            if file_type.is_dir() {
                                dirs.push(name);
                            } else {
                                files.push(name);
                            }
                        }
                    }
                }

                files.sort();
                dirs.sort();

                Ok(ToolResponse::success(serde_json::json!({
                    "path": path,
                    "files": files,
                    "directories": dirs,
                    "total": files.len() + dirs.len()
                })))
            }
            Err(e) => Ok(ToolResponse::failure(format!(
                "Failed to list directory '{}': {}",
                path, e
            ))),
        }
    }

    /// Check if a path exists
    async fn exists(&self, path: &str) -> Result<ToolResponse> {
        let resolved = self.resolve_path(path)?;

        Ok(ToolResponse::success(serde_json::json!({
            "path": path,
            "exists": resolved.exists()
        })))
    }
}

impl Default for FilesystemTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FilesystemTool {
    fn id(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "Filesystem operations: read, write, list, and check existence of files and directories"
    }

    fn supported_operations(&self) -> Vec<String> {
        vec![
            "read".to_string(),
            "write".to_string(),
            "list".to_string(),
            "exists".to_string(),
        ]
    }

    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse> {
        match request.operation.as_str() {
            "read" => {
                let path = request.params["path"].as_str().ok_or_else(|| {
                    bodhya_core::Error::Tool("Missing 'path' parameter".to_string())
                })?;
                self.read_file(path).await
            }
            "write" => {
                let path = request.params["path"].as_str().ok_or_else(|| {
                    bodhya_core::Error::Tool("Missing 'path' parameter".to_string())
                })?;
                let content = request.params["content"].as_str().ok_or_else(|| {
                    bodhya_core::Error::Tool("Missing 'content' parameter".to_string())
                })?;
                self.write_file(path, content).await
            }
            "list" => {
                let path = request.params["path"].as_str().ok_or_else(|| {
                    bodhya_core::Error::Tool("Missing 'path' parameter".to_string())
                })?;
                self.list_dir(path).await
            }
            "exists" => {
                let path = request.params["path"].as_str().ok_or_else(|| {
                    bodhya_core::Error::Tool("Missing 'path' parameter".to_string())
                })?;
                self.exists(path).await
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
    async fn test_filesystem_tool_creation() {
        let tool = FilesystemTool::new();
        assert_eq!(tool.id(), "filesystem");
        assert!(tool.description().contains("Filesystem"));
    }

    #[tokio::test]
    async fn test_filesystem_tool_supported_operations() {
        let tool = FilesystemTool::new();
        let ops = tool.supported_operations();

        assert!(ops.contains(&"read".to_string()));
        assert!(ops.contains(&"write".to_string()));
        assert!(ops.contains(&"list".to_string()));
        assert!(ops.contains(&"exists".to_string()));
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FilesystemTool::with_base_dir(temp_dir.path());

        // Write a file
        let write_req = ToolRequest::new(
            "filesystem",
            "write",
            serde_json::json!({
                "path": "test.txt",
                "content": "Hello, World!"
            }),
        );

        let write_resp = tool.execute(write_req).await.unwrap();
        assert!(write_resp.success);

        // Read the file back
        let read_req = ToolRequest::new(
            "filesystem",
            "read",
            serde_json::json!({
                "path": "test.txt"
            }),
        );

        let read_resp = tool.execute(read_req).await.unwrap();
        assert!(read_resp.success);
        assert_eq!(read_resp.data["content"], "Hello, World!");
    }

    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FilesystemTool::with_base_dir(temp_dir.path());

        // Create some files
        std::fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        std::fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        // List directory
        let list_req = ToolRequest::new(
            "filesystem",
            "list",
            serde_json::json!({
                "path": "."
            }),
        );

        let list_resp = tool.execute(list_req).await.unwrap();
        assert!(list_resp.success);
        assert_eq!(list_resp.data["total"], 3);

        let files = list_resp.data["files"].as_array().unwrap();
        assert_eq!(files.len(), 2);

        let dirs = list_resp.data["directories"].as_array().unwrap();
        assert_eq!(dirs.len(), 1);
    }

    #[tokio::test]
    async fn test_exists_operation() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FilesystemTool::with_base_dir(temp_dir.path());

        // Create a file
        std::fs::write(temp_dir.path().join("exists.txt"), "content").unwrap();

        // Check existing file
        let exists_req = ToolRequest::new(
            "filesystem",
            "exists",
            serde_json::json!({
                "path": "exists.txt"
            }),
        );

        let exists_resp = tool.execute(exists_req).await.unwrap();
        assert!(exists_resp.success);
        assert_eq!(exists_resp.data["exists"], true);

        // Check non-existing file
        let not_exists_req = ToolRequest::new(
            "filesystem",
            "exists",
            serde_json::json!({
                "path": "not_exists.txt"
            }),
        );

        let not_exists_resp = tool.execute(not_exists_req).await.unwrap();
        assert!(not_exists_resp.success);
        assert_eq!(not_exists_resp.data["exists"], false);
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FilesystemTool::with_base_dir(temp_dir.path());

        let read_req = ToolRequest::new(
            "filesystem",
            "read",
            serde_json::json!({
                "path": "nonexistent.txt"
            }),
        );

        let read_resp = tool.execute(read_req).await.unwrap();
        assert!(!read_resp.success);
        assert!(read_resp.error.unwrap().contains("Failed to read file"));
    }

    #[tokio::test]
    async fn test_write_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FilesystemTool::with_base_dir(temp_dir.path());

        let write_req = ToolRequest::new(
            "filesystem",
            "write",
            serde_json::json!({
                "path": "subdir/nested/file.txt",
                "content": "nested content"
            }),
        );

        let write_resp = tool.execute(write_req).await.unwrap();
        assert!(write_resp.success);

        // Verify the file was created
        let read_req = ToolRequest::new(
            "filesystem",
            "read",
            serde_json::json!({
                "path": "subdir/nested/file.txt"
            }),
        );

        let read_resp = tool.execute(read_req).await.unwrap();
        assert!(read_resp.success);
        assert_eq!(read_resp.data["content"], "nested content");
    }

    #[tokio::test]
    async fn test_unsupported_operation() {
        let tool = FilesystemTool::new();

        let req = ToolRequest::new(
            "filesystem",
            "delete",
            serde_json::json!({
                "path": "test.txt"
            }),
        );

        let resp = tool.execute(req).await.unwrap();
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Unsupported operation"));
    }

    #[tokio::test]
    async fn test_missing_path_parameter() {
        let tool = FilesystemTool::new();

        let req = ToolRequest::new("filesystem", "read", serde_json::json!({}));

        let result = tool.execute(req).await;
        assert!(result.is_err());
    }
}
