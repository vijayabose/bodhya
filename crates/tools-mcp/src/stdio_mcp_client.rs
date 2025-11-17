/// Stdio-based MCP client implementation
///
/// This module provides a full-featured MCP client that communicates with
/// MCP servers via stdio (spawning a process and communicating over stdin/stdout).
use async_trait::async_trait;
use bodhya_core::{Error, McpClient, McpServerConfig, Result, ToolRequest, ToolResponse};
use serde_json::Value;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, RequestId};

/// Stdio-based MCP client
pub struct StdioMcpClient {
    /// Server configuration
    config: Option<McpServerConfig>,
    /// Child process
    process: Option<Child>,
    /// Standard input to process
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    /// Standard output from process
    stdout: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
    /// Request ID counter
    request_id: AtomicU64,
    /// Available tools from server
    available_tools: Arc<Mutex<Vec<String>>>,
    /// Connection state
    connected: Arc<Mutex<bool>>,
}

impl StdioMcpClient {
    /// Create a new stdio MCP client
    pub fn new() -> Self {
        Self {
            config: None,
            process: None,
            stdin: None,
            stdout: None,
            request_id: AtomicU64::new(1),
            available_tools: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Get next request ID
    fn next_id(&self) -> RequestId {
        RequestId::Number(self.request_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<JsonRpcResponse> {
        let id = self.next_id();
        let request = JsonRpcRequest::new(id.clone(), method, params);

        // Serialize request
        let mut request_json = serde_json::to_string(&request)
            .map_err(|e| Error::Tool(format!("Failed to serialize request: {}", e)))?;
        request_json.push('\n');

        // Write to stdin
        if let Some(ref stdin) = self.stdin {
            let mut stdin_guard = stdin.lock().await;
            stdin_guard
                .write_all(request_json.as_bytes())
                .await
                .map_err(|e| Error::Tool(format!("Failed to write to MCP server: {}", e)))?;
            stdin_guard
                .flush()
                .await
                .map_err(|e| Error::Tool(format!("Failed to flush stdin: {}", e)))?;
        } else {
            return Err(Error::Tool("MCP client not connected".to_string()));
        }

        // Read response from stdout
        if let Some(ref stdout) = self.stdout {
            let mut stdout_guard = stdout.lock().await;
            let mut line = String::new();
            stdout_guard
                .read_line(&mut line)
                .await
                .map_err(|e| Error::Tool(format!("Failed to read from MCP server: {}", e)))?;

            // Parse response
            let response: JsonRpcResponse = serde_json::from_str(&line)
                .map_err(|e| Error::Tool(format!("Failed to parse response: {}", e)))?;

            // Check for errors
            if let Some(error) = &response.error {
                return Err(Error::Tool(format!(
                    "MCP server error {}: {}",
                    error.code, error.message
                )));
            }

            Ok(response)
        } else {
            Err(Error::Tool("MCP client not connected".to_string()))
        }
    }

    /// Initialize connection with MCP server
    async fn initialize(&self) -> Result<()> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "bodhya",
                "version": "0.1.0"
            }
        });

        let response = self.send_request("initialize", Some(params)).await?;

        if response.result.is_some() {
            *self.connected.lock().await = true;
            Ok(())
        } else {
            Err(Error::Tool("MCP server initialization failed".to_string()))
        }
    }

    /// List available tools from server
    async fn list_tools_internal(&self) -> Result<Vec<String>> {
        let response = self.send_request("tools/list", None).await?;

        if let Some(result) = response.result {
            if let Some(tools_array) = result.get("tools").and_then(|v| v.as_array()) {
                let tools: Vec<String> = tools_array
                    .iter()
                    .filter_map(|tool| tool.get("name").and_then(|n| n.as_str()).map(String::from))
                    .collect();
                return Ok(tools);
            }
        }

        Ok(Vec::new())
    }

    /// Call a tool on the MCP server
    async fn call_tool_internal(&self, tool_name: &str, arguments: Value) -> Result<ToolResponse> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });

        let response = self.send_request("tools/call", Some(params)).await?;

        if let Some(result) = response.result {
            // Extract content from MCP response
            if let Some(content) = result.get("content").and_then(|v| v.as_array()) {
                let data = serde_json::json!({
                    "content": content
                });
                return Ok(ToolResponse::success(data));
            }

            // Fallback: return entire result
            Ok(ToolResponse::success(result))
        } else {
            Ok(ToolResponse::failure("No result from MCP server"))
        }
    }
}

impl Default for StdioMcpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn connect(&mut self, config: &McpServerConfig) -> Result<()> {
        // Expand environment variables in config
        let expanded_config = config.expand_env_vars();

        // Only support stdio type for now
        if expanded_config.server_type != "stdio" {
            return Err(Error::Tool(format!(
                "Unsupported MCP server type: {}. Only 'stdio' is supported.",
                expanded_config.server_type
            )));
        }

        // Get command and args
        let command_vec = expanded_config
            .command
            .as_ref()
            .ok_or_else(|| Error::Tool("No command specified for stdio MCP server".to_string()))?;

        if command_vec.is_empty() {
            return Err(Error::Tool(
                "Empty command for stdio MCP server".to_string(),
            ));
        }

        let program = &command_vec[0];
        let args = &command_vec[1..];

        // Spawn process
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        // Set environment variables
        for (key, value) in &expanded_config.env {
            cmd.env(key, value);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| Error::Tool(format!("Failed to spawn MCP server: {}", e)))?;

        // Get stdin/stdout
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Tool("Failed to get stdin for MCP server".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Tool("Failed to get stdout for MCP server".to_string()))?;

        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.stdout = Some(Arc::new(Mutex::new(BufReader::new(stdout))));
        self.process = Some(child);
        self.config = Some(expanded_config);

        // Initialize connection
        self.initialize().await?;

        // Discover available tools
        let tools = self.list_tools_internal().await?;
        *self.available_tools.lock().await = tools;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        *self.connected.lock().await = false;

        // Kill process if running
        if let Some(mut process) = self.process.take() {
            let _ = process.kill().await;
        }

        self.stdin = None;
        self.stdout = None;
        self.config = None;
        self.available_tools.lock().await.clear();

        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Use try_lock to avoid async in non-async method
        self.connected
            .try_lock()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    async fn list_tools(&self) -> Result<Vec<String>> {
        if !*self.connected.lock().await {
            return Err(Error::Tool("MCP client not connected".to_string()));
        }

        Ok(self.available_tools.lock().await.clone())
    }

    async fn call_tool(&self, request: ToolRequest) -> Result<ToolResponse> {
        if !*self.connected.lock().await {
            return Err(Error::Tool("MCP client not connected".to_string()));
        }

        // Convert ToolRequest params to JSON
        let arguments = request.params;

        self.call_tool_internal(&request.tool, arguments).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = StdioMcpClient::new();
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_client_default() {
        let client = StdioMcpClient::default();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_request_id_generation() {
        let client = StdioMcpClient::new();
        let id1 = client.next_id();
        let id2 = client.next_id();
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let mut client = StdioMcpClient::new();
        let result = client.disconnect().await;
        assert!(result.is_ok());
    }
}
