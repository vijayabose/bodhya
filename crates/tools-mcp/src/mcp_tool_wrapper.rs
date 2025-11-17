/// MCP Tool Wrapper
///
/// This module provides a wrapper that implements the Tool trait for tools exposed by MCP servers.
use async_trait::async_trait;
use bodhya_core::{McpClient, Result, Tool, ToolRequest, ToolResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Wrapper that adapts an MCP tool to the Tool trait
pub struct McpToolWrapper {
    /// Tool name from MCP server
    tool_name: String,
    /// MCP client (shared across multiple tools from same server)
    client: Arc<Mutex<Box<dyn McpClient>>>,
    /// Server name (for debugging/logging)
    #[allow(dead_code)]
    server_name: String,
}

impl McpToolWrapper {
    /// Create a new MCP tool wrapper
    pub fn new(
        tool_name: String,
        client: Arc<Mutex<Box<dyn McpClient>>>,
        server_name: String,
    ) -> Self {
        Self {
            tool_name,
            client,
            server_name,
        }
    }

    /// Get the fully qualified tool ID (server:tool)
    #[allow(dead_code)]
    fn qualified_id(&self) -> String {
        format!("{}:{}", self.server_name, self.tool_name)
    }
}

#[async_trait]
impl Tool for McpToolWrapper {
    fn id(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        "Tool from MCP server"
    }

    fn supported_operations(&self) -> Vec<String> {
        // MCP tools typically support a single "call" operation
        vec!["call".to_string()]
    }

    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse> {
        // Forward request to MCP client
        let client_guard = self.client.lock().await;

        // Create new request with tool name
        let mcp_request = ToolRequest::new(&self.tool_name, &request.operation, request.params);

        client_guard.call_tool(mcp_request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bodhya_core::{Error, McpServerConfig};

    // Mock MCP client for testing
    struct MockMcpClient {
        connected: bool,
        tools: Vec<String>,
    }

    impl MockMcpClient {
        fn new() -> Self {
            Self {
                connected: false,
                tools: vec!["test-tool".to_string()],
            }
        }
    }

    #[async_trait]
    impl McpClient for MockMcpClient {
        async fn connect(&mut self, _config: &McpServerConfig) -> Result<()> {
            self.connected = true;
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        async fn list_tools(&self) -> Result<Vec<String>> {
            if !self.connected {
                return Err(Error::Tool("Not connected".to_string()));
            }
            Ok(self.tools.clone())
        }

        async fn call_tool(&self, request: ToolRequest) -> Result<ToolResponse> {
            if !self.connected {
                return Err(Error::Tool("Not connected".to_string()));
            }

            if !self.tools.contains(&request.tool) {
                return Ok(ToolResponse::failure("Tool not found"));
            }

            Ok(ToolResponse::success(serde_json::json!({
                "tool": request.tool,
                "operation": request.operation,
                "result": "success"
            })))
        }
    }

    #[tokio::test]
    async fn test_mcp_tool_wrapper_creation() {
        let client: Box<dyn McpClient> = Box::new(MockMcpClient::new());
        let client_arc = Arc::new(Mutex::new(client));

        let wrapper = McpToolWrapper::new(
            "test-tool".to_string(),
            client_arc,
            "test-server".to_string(),
        );

        assert_eq!(wrapper.id(), "test-tool");
        assert_eq!(wrapper.qualified_id(), "test-server:test-tool");
    }

    #[tokio::test]
    async fn test_mcp_tool_wrapper_execute() {
        let mut client = MockMcpClient::new();
        client.connected = true;

        let client_box: Box<dyn McpClient> = Box::new(client);
        let client_arc = Arc::new(Mutex::new(client_box));

        let wrapper = McpToolWrapper::new(
            "test-tool".to_string(),
            client_arc,
            "test-server".to_string(),
        );

        let request = ToolRequest::new("test-tool", "call", serde_json::json!({}));
        let response = wrapper.execute(request).await.unwrap();

        assert!(response.success);
        assert_eq!(response.data["tool"], "test-tool");
    }

    #[tokio::test]
    async fn test_mcp_tool_wrapper_not_connected() {
        let client = MockMcpClient::new(); // Not connected

        let client_box: Box<dyn McpClient> = Box::new(client);
        let client_arc = Arc::new(Mutex::new(client_box));

        let wrapper = McpToolWrapper::new(
            "test-tool".to_string(),
            client_arc,
            "test-server".to_string(),
        );

        let request = ToolRequest::new("test-tool", "call", serde_json::json!({}));
        let result = wrapper.execute(request).await;

        assert!(result.is_err());
    }
}
