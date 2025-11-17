/// EditTool - Advanced file editing with line-based operations
///
/// Provides precise file editing capabilities including:
/// - String replacement
/// - Line-based insertion/deletion
/// - Patch application
/// - Dry-run validation
use async_trait::async_trait;
use bodhya_core::{Result, Tool, ToolRequest, ToolResponse};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// EditTool provides advanced file editing capabilities
pub struct EditTool;

/// Edit operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum EditOperation {
    /// Replace all occurrences of a string
    Replace {
        old: String,
        new: String,
        #[serde(default)]
        count: Option<usize>, // None = replace all
    },
    /// Insert content at a specific line number (1-indexed)
    InsertAtLine { line_number: usize, content: String },
    /// Delete a range of lines (1-indexed, inclusive)
    DeleteLines { start: usize, end: usize },
    /// Apply a unified diff patch
    Patch { patch: String },
}

/// Edit result containing the modified content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditResult {
    pub success: bool,
    pub modified_content: String,
    pub changes_made: usize,
    pub dry_run: bool,
    pub error: Option<String>,
}

impl EditTool {
    pub fn new() -> Self {
        Self
    }

    /// Perform edit operation with optional dry-run
    pub async fn edit(
        &self,
        path: impl AsRef<Path>,
        operation: EditOperation,
        dry_run: bool,
    ) -> Result<EditResult> {
        let path = path.as_ref();

        // Read current content
        let original_content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| bodhya_core::Error::Tool(format!("Failed to read file: {}", e)))?;

        // Apply operation
        let (modified_content, changes_made) = match operation {
            EditOperation::Replace { old, new, count } => {
                self.apply_replace(&original_content, &old, &new, count)
            }
            EditOperation::InsertAtLine {
                line_number,
                content,
            } => self.apply_insert(&original_content, line_number, &content)?,
            EditOperation::DeleteLines { start, end } => {
                self.apply_delete(&original_content, start, end)?
            }
            EditOperation::Patch { patch } => self.apply_patch(&original_content, &patch)?,
        };

        // If not dry-run, write the changes
        if !dry_run && changes_made > 0 {
            tokio::fs::write(path, &modified_content)
                .await
                .map_err(|e| bodhya_core::Error::Tool(format!("Failed to write file: {}", e)))?;
        }

        Ok(EditResult {
            success: true,
            modified_content,
            changes_made,
            dry_run,
            error: None,
        })
    }

    /// Apply string replacement
    fn apply_replace(
        &self,
        content: &str,
        old: &str,
        new: &str,
        count: Option<usize>,
    ) -> (String, usize) {
        if old.is_empty() {
            return (content.to_string(), 0);
        }

        let matches: Vec<_> = content.match_indices(old).collect();
        let changes = if let Some(limit) = count {
            matches.len().min(limit)
        } else {
            matches.len()
        };

        if changes == 0 {
            return (content.to_string(), 0);
        }

        let result = if let Some(limit) = count {
            let mut result = content.to_string();
            for _ in 0..limit {
                if let Some(pos) = result.find(old) {
                    result.replace_range(pos..pos + old.len(), new);
                } else {
                    break;
                }
            }
            result
        } else {
            content.replace(old, new)
        };

        (result, changes)
    }

    /// Insert content at a specific line
    fn apply_insert(
        &self,
        content: &str,
        line_number: usize,
        insert_content: &str,
    ) -> Result<(String, usize)> {
        if line_number == 0 {
            return Err(bodhya_core::Error::Tool(
                "Line numbers are 1-indexed, cannot insert at line 0".to_string(),
            ));
        }

        let mut lines: Vec<&str> = content.lines().collect();
        let insert_idx = line_number - 1;

        if insert_idx > lines.len() {
            return Err(bodhya_core::Error::Tool(format!(
                "Line number {} exceeds file length {}",
                line_number,
                lines.len()
            )));
        }

        lines.insert(insert_idx, insert_content);
        let result = lines.join("\n");

        // Preserve trailing newline if original had one
        let result = if content.ends_with('\n') && !result.ends_with('\n') {
            format!("{}\n", result)
        } else {
            result
        };

        Ok((result, 1))
    }

    /// Delete a range of lines
    fn apply_delete(&self, content: &str, start: usize, end: usize) -> Result<(String, usize)> {
        if start == 0 || end == 0 {
            return Err(bodhya_core::Error::Tool(
                "Line numbers are 1-indexed, cannot use 0".to_string(),
            ));
        }

        if start > end {
            return Err(bodhya_core::Error::Tool(format!(
                "Start line {} must be <= end line {}",
                start, end
            )));
        }

        let lines: Vec<&str> = content.lines().collect();
        let start_idx = start - 1;
        let end_idx = end - 1;

        if end_idx >= lines.len() {
            return Err(bodhya_core::Error::Tool(format!(
                "End line {} exceeds file length {}",
                end,
                lines.len()
            )));
        }

        let changes = end - start + 1;
        let mut result_lines = Vec::new();
        result_lines.extend_from_slice(&lines[..start_idx]);
        result_lines.extend_from_slice(&lines[end_idx + 1..]);

        let result = result_lines.join("\n");

        // Preserve trailing newline if original had one
        let result = if content.ends_with('\n') && !result.is_empty() && !result.ends_with('\n') {
            format!("{}\n", result)
        } else {
            result
        };

        Ok((result, changes))
    }

    /// Apply a unified diff patch (simplified implementation)
    fn apply_patch(&self, _content: &str, _patch: &str) -> Result<(String, usize)> {
        // Simplified patch implementation for now
        // Full unified diff parsing would require a dedicated library
        Err(bodhya_core::Error::Tool(
            "Patch operation not yet implemented - use replace, insert, or delete instead"
                .to_string(),
        ))
    }
}

impl Default for EditTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for EditTool {
    fn id(&self) -> &'static str {
        "edit"
    }

    fn description(&self) -> &'static str {
        "Advanced file editing with replace, insert, delete, and patch operations"
    }

    fn supported_operations(&self) -> Vec<String> {
        vec!["edit".to_string()]
    }

    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse> {
        let path = request
            .params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| bodhya_core::Error::Tool("Missing 'path' parameter".to_string()))?;

        let operation: EditOperation = request
            .params
            .get("operation")
            .ok_or_else(|| bodhya_core::Error::Tool("Missing 'operation' parameter".to_string()))
            .and_then(|v| {
                serde_json::from_value(v.clone())
                    .map_err(|e| bodhya_core::Error::Tool(format!("Invalid operation: {}", e)))
            })?;

        let dry_run = request
            .params
            .get("dry_run")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let result = self.edit(path, operation, dry_run).await?;

        let data = serde_json::to_value(result)
            .map_err(|e| bodhya_core::Error::Tool(format!("Failed to serialize result: {}", e)))?;

        Ok(ToolResponse::success(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        tokio::fs::write(&path, content).await.unwrap();
        path
    }

    #[tokio::test]
    async fn test_edit_tool_creation() {
        let tool = EditTool::new();
        assert_eq!(tool.id(), "edit");
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_replace_operation() {
        let tool = EditTool::new();
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.txt", "hello world\nhello rust\n").await;

        let operation = EditOperation::Replace {
            old: "hello".to_string(),
            new: "goodbye".to_string(),
            count: None,
        };

        let result = tool.edit(&path, operation, false).await.unwrap();

        assert!(result.success);
        assert_eq!(result.changes_made, 2);
        assert!(result.modified_content.contains("goodbye world"));
        assert!(result.modified_content.contains("goodbye rust"));

        // Verify file was actually modified
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, result.modified_content);
    }

    #[tokio::test]
    async fn test_replace_with_count() {
        let tool = EditTool::new();
        let content = "hello world\nhello rust\nhello bodhya\n";

        let (result, changes) = tool.apply_replace(content, "hello", "hi", Some(2));

        assert_eq!(changes, 2);
        assert!(result.contains("hi world"));
        assert!(result.contains("hi rust"));
        assert!(result.contains("hello bodhya")); // Third occurrence unchanged
    }

    #[tokio::test]
    async fn test_insert_at_line() {
        let tool = EditTool::new();
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.txt", "line 1\nline 2\nline 3\n").await;

        let operation = EditOperation::InsertAtLine {
            line_number: 2,
            content: "inserted line".to_string(),
        };

        let result = tool.edit(&path, operation, false).await.unwrap();

        assert!(result.success);
        assert_eq!(result.changes_made, 1);
        let lines: Vec<&str> = result.modified_content.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[1], "inserted line");
    }

    #[tokio::test]
    async fn test_delete_lines() {
        let tool = EditTool::new();
        let temp_dir = TempDir::new().unwrap();
        let path =
            create_test_file(&temp_dir, "test.txt", "line 1\nline 2\nline 3\nline 4\n").await;

        let operation = EditOperation::DeleteLines { start: 2, end: 3 };

        let result = tool.edit(&path, operation, false).await.unwrap();

        assert!(result.success);
        assert_eq!(result.changes_made, 2);
        let lines: Vec<&str> = result.modified_content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line 1");
        assert_eq!(lines[1], "line 4");
    }

    #[tokio::test]
    async fn test_dry_run_mode() {
        let tool = EditTool::new();
        let temp_dir = TempDir::new().unwrap();
        let original_content = "hello world\n";
        let path = create_test_file(&temp_dir, "test.txt", original_content).await;

        let operation = EditOperation::Replace {
            old: "hello".to_string(),
            new: "goodbye".to_string(),
            count: None,
        };

        let result = tool.edit(&path, operation, true).await.unwrap();

        assert!(result.success);
        assert!(result.dry_run);
        assert_eq!(result.changes_made, 1);

        // Verify file was NOT modified
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, original_content);
    }

    #[tokio::test]
    async fn test_insert_at_invalid_line() {
        let tool = EditTool::new();
        let content = "line 1\nline 2\n";

        let result = tool.apply_insert(content, 0, "test");
        assert!(result.is_err());

        let result = tool.apply_insert(content, 100, "test");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_invalid_range() {
        let tool = EditTool::new();
        let content = "line 1\nline 2\nline 3\n";

        // Start > end
        let result = tool.apply_delete(content, 3, 1);
        assert!(result.is_err());

        // Line 0
        let result = tool.apply_delete(content, 0, 2);
        assert!(result.is_err());

        // Beyond file length
        let result = tool.apply_delete(content, 1, 100);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_replace_empty_string() {
        let tool = EditTool::new();
        let content = "hello world";

        let (result, changes) = tool.apply_replace(content, "", "test", None);

        assert_eq!(changes, 0);
        assert_eq!(result, content);
    }

    #[tokio::test]
    async fn test_replace_no_matches() {
        let tool = EditTool::new();
        let content = "hello world";

        let (result, changes) = tool.apply_replace(content, "goodbye", "hi", None);

        assert_eq!(changes, 0);
        assert_eq!(result, content);
    }
}
