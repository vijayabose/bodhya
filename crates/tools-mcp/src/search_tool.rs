/// SearchTool - Code search with grep and pattern matching
///
/// Provides search capabilities including:
/// - Recursive grep with regex support
/// - File pattern filtering
/// - Line number tracking
/// - Context lines (before/after)
use async_trait::async_trait;
use bodhya_core::{Result, Tool, ToolRequest, ToolResponse};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// SearchTool provides code search capabilities
pub struct SearchTool;

/// A single search match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_before: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_after: Option<Vec<String>>,
}

/// Search result containing all matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub success: bool,
    pub matches: Vec<SearchMatch>,
    pub total_matches: usize,
    pub files_searched: usize,
    pub error: Option<String>,
}

impl SearchTool {
    pub fn new() -> Self {
        Self
    }

    /// Perform grep search in a directory
    pub async fn grep(
        &self,
        path: impl AsRef<Path>,
        pattern: &str,
        recursive: bool,
        case_sensitive: bool,
        file_pattern: Option<&str>,
        context_lines: usize,
    ) -> Result<SearchResult> {
        let path = path.as_ref();

        // Compile regex pattern
        let regex_pattern = if case_sensitive {
            pattern
        } else {
            &format!("(?i){}", pattern)
        };

        let regex = Regex::new(regex_pattern)
            .map_err(|e| bodhya_core::Error::Tool(format!("Invalid regex pattern: {}", e)))?;

        let file_filter = file_pattern.map(|p| {
            glob::Pattern::new(p)
                .map_err(|e| bodhya_core::Error::Tool(format!("Invalid file pattern: {}", e)))
        });

        if let Some(Err(e)) = file_filter {
            return Err(e);
        }

        let file_filter = file_filter.map(|r| r.unwrap());

        let mut matches = Vec::new();
        let mut files_searched = 0;

        if path.is_file() {
            if let Ok(file_matches) = self.search_file(path, &regex, context_lines).await {
                matches.extend(file_matches);
                files_searched += 1;
            }
        } else if path.is_dir() {
            if recursive {
                self.search_directory_recursive(
                    path,
                    &regex,
                    &file_filter,
                    context_lines,
                    &mut matches,
                    &mut files_searched,
                )
                .await?;
            } else {
                self.search_directory_shallow(
                    path,
                    &regex,
                    &file_filter,
                    context_lines,
                    &mut matches,
                    &mut files_searched,
                )
                .await?;
            }
        } else {
            return Err(bodhya_core::Error::Tool(format!(
                "Path does not exist: {}",
                path.display()
            )));
        }

        Ok(SearchResult {
            success: true,
            total_matches: matches.len(),
            matches,
            files_searched,
            error: None,
        })
    }

    /// Search a single file
    async fn search_file(
        &self,
        path: &Path,
        regex: &Regex,
        context_lines: usize,
    ) -> Result<Vec<SearchMatch>> {
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| bodhya_core::Error::Tool(format!("Failed to read file: {}", e)))?;

        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            if let Some(mat) = regex.find(line) {
                let context_before = if context_lines > 0 && idx > 0 {
                    let start = idx.saturating_sub(context_lines);
                    Some(lines[start..idx].iter().map(|s| s.to_string()).collect())
                } else {
                    None
                };

                let context_after = if context_lines > 0 && idx + 1 < lines.len() {
                    let end = (idx + 1 + context_lines).min(lines.len());
                    Some(lines[idx + 1..end].iter().map(|s| s.to_string()).collect())
                } else {
                    None
                };

                matches.push(SearchMatch {
                    file_path: path.to_path_buf(),
                    line_number: idx + 1, // 1-indexed
                    line_content: line.to_string(),
                    column: mat.start() + 1, // 1-indexed
                    context_before,
                    context_after,
                });
            }
        }

        Ok(matches)
    }

    /// Search directory recursively
    fn search_directory_recursive<'a>(
        &'a self,
        path: &'a Path,
        regex: &'a Regex,
        file_filter: &'a Option<glob::Pattern>,
        context_lines: usize,
        matches: &'a mut Vec<SearchMatch>,
        files_searched: &'a mut usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut entries = fs::read_dir(path).await.map_err(|e| {
                bodhya_core::Error::Tool(format!("Failed to read directory: {}", e))
            })?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| bodhya_core::Error::Tool(format!("Failed to read entry: {}", e)))?
            {
                let path = entry.path();

                if path.is_dir() {
                    // Skip hidden directories
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with('.') {
                            continue;
                        }
                    }

                    // Recurse into subdirectory
                    self.search_directory_recursive(
                        &path,
                        regex,
                        file_filter,
                        context_lines,
                        matches,
                        files_searched,
                    )
                    .await?;
                } else if path.is_file() {
                    // Check file filter
                    if let Some(filter) = file_filter {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if !filter.matches(name) {
                                continue;
                            }
                        }
                    }

                    // Search file
                    if let Ok(file_matches) = self.search_file(&path, regex, context_lines).await {
                        matches.extend(file_matches);
                        *files_searched += 1;
                    }
                }
            }

            Ok(())
        })
    }

    /// Search directory (non-recursive)
    async fn search_directory_shallow(
        &self,
        path: &Path,
        regex: &Regex,
        file_filter: &Option<glob::Pattern>,
        context_lines: usize,
        matches: &mut Vec<SearchMatch>,
        files_searched: &mut usize,
    ) -> Result<()> {
        let mut entries = fs::read_dir(path)
            .await
            .map_err(|e| bodhya_core::Error::Tool(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| bodhya_core::Error::Tool(format!("Failed to read entry: {}", e)))?
        {
            let path = entry.path();

            if path.is_file() {
                // Check file filter
                if let Some(filter) = file_filter {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !filter.matches(name) {
                            continue;
                        }
                    }
                }

                // Search file
                if let Ok(file_matches) = self.search_file(&path, regex, context_lines).await {
                    matches.extend(file_matches);
                    *files_searched += 1;
                }
            }
        }

        Ok(())
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn id(&self) -> &'static str {
        "search"
    }

    fn description(&self) -> &'static str {
        "Search files with regex patterns, supports recursive search and file filtering"
    }

    fn supported_operations(&self) -> Vec<String> {
        vec!["grep".to_string()]
    }

    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse> {
        let path = request
            .params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| bodhya_core::Error::Tool("Missing 'path' parameter".to_string()))?;

        let pattern = request
            .params
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| bodhya_core::Error::Tool("Missing 'pattern' parameter".to_string()))?;

        let recursive = request
            .params
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let case_sensitive = request
            .params
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let file_pattern = request.params.get("file_pattern").and_then(|v| v.as_str());

        let context_lines = request
            .params
            .get("context_lines")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let result = self
            .grep(
                path,
                pattern,
                recursive,
                case_sensitive,
                file_pattern,
                context_lines,
            )
            .await?;

        let data = serde_json::to_value(result)
            .map_err(|e| bodhya_core::Error::Tool(format!("Failed to serialize result: {}", e)))?;

        Ok(ToolResponse::success(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.unwrap();
        }
        tokio::fs::write(&path, content).await.unwrap();
        path
    }

    #[tokio::test]
    async fn test_search_tool_creation() {
        let tool = SearchTool::new();
        assert_eq!(tool.id(), "search");
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_grep_single_file() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();
        let content = "hello world\ntest line\nhello rust\n";
        let path = create_test_file(&temp_dir, "test.txt", content).await;

        let result = tool
            .grep(&path, "hello", false, true, None, 0)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.total_matches, 2);
        assert_eq!(result.files_searched, 1);
        assert_eq!(result.matches[0].line_number, 1);
        assert_eq!(result.matches[1].line_number, 3);
    }

    #[tokio::test]
    async fn test_grep_case_insensitive() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();
        let content = "Hello world\nHELLO rust\nhello bodhya\n";
        let path = create_test_file(&temp_dir, "test.txt", content).await;

        let result = tool
            .grep(&path, "hello", false, false, None, 0)
            .await
            .unwrap();

        assert_eq!(result.total_matches, 3);
    }

    #[tokio::test]
    async fn test_grep_case_sensitive() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();
        let content = "Hello world\nHELLO rust\nhello bodhya\n";
        let path = create_test_file(&temp_dir, "test.txt", content).await;

        let result = tool
            .grep(&path, "hello", false, true, None, 0)
            .await
            .unwrap();

        assert_eq!(result.total_matches, 1);
        assert_eq!(result.matches[0].line_number, 3);
    }

    #[tokio::test]
    async fn test_grep_with_context() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();
        let content = "line 1\nline 2\nMATCH\nline 4\nline 5\n";
        let path = create_test_file(&temp_dir, "test.txt", content).await;

        let result = tool
            .grep(&path, "MATCH", false, true, None, 2)
            .await
            .unwrap();

        assert_eq!(result.total_matches, 1);
        let match_result = &result.matches[0];
        assert_eq!(match_result.line_number, 3);
        assert_eq!(match_result.context_before.as_ref().unwrap().len(), 2);
        assert_eq!(match_result.context_after.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_grep_recursive() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "file1.txt", "hello world\n").await;
        create_test_file(&temp_dir, "sub/file2.txt", "hello rust\n").await;
        create_test_file(&temp_dir, "sub/nested/file3.txt", "hello bodhya\n").await;

        let result = tool
            .grep(temp_dir.path(), "hello", true, true, None, 0)
            .await
            .unwrap();

        assert_eq!(result.total_matches, 3);
        assert_eq!(result.files_searched, 3);
    }

    #[tokio::test]
    async fn test_grep_with_file_pattern() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();

        create_test_file(&temp_dir, "test.rs", "fn main() {}\n").await;
        create_test_file(&temp_dir, "test.txt", "fn helper() {}\n").await;
        create_test_file(&temp_dir, "lib.rs", "fn lib() {}\n").await;

        let result = tool
            .grep(temp_dir.path(), "fn", false, true, Some("*.rs"), 0)
            .await
            .unwrap();

        assert_eq!(result.total_matches, 2); // Only .rs files
        assert_eq!(result.files_searched, 2);
    }

    #[tokio::test]
    async fn test_grep_regex_pattern() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();
        let content = "test123\ntest456\nhello789\n";
        let path = create_test_file(&temp_dir, "test.txt", content).await;

        let result = tool
            .grep(&path, r"test\d+", false, true, None, 0)
            .await
            .unwrap();

        assert_eq!(result.total_matches, 2);
    }

    #[tokio::test]
    async fn test_grep_no_matches() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();
        let content = "hello world\n";
        let path = create_test_file(&temp_dir, "test.txt", content).await;

        let result = tool
            .grep(&path, "goodbye", false, true, None, 0)
            .await
            .unwrap();

        assert_eq!(result.total_matches, 0);
        assert_eq!(result.files_searched, 1);
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() {
        let tool = SearchTool::new();
        let temp_dir = TempDir::new().unwrap();
        let path = create_test_file(&temp_dir, "test.txt", "test\n").await;

        let result = tool.grep(&path, "[invalid", false, true, None, 0).await;

        assert!(result.is_err());
    }
}
