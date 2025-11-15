/// Tool and MCP abstractions for Bodhya
///
/// This module defines the core traits for tools and MCP (Model Context Protocol)
/// integrations, allowing agents to interact with external systems (filesystem,
/// git, shell, etc.) in a uniform way.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::Result;

/// Represents a request to execute a tool
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolRequest {
    /// Tool name/identifier
    pub tool: String,
    /// Operation to perform
    pub operation: String,
    /// Parameters for the operation
    #[serde(default)]
    pub params: serde_json::Value,
}

impl ToolRequest {
    /// Create a new tool request
    pub fn new(
        tool: impl Into<String>,
        operation: impl Into<String>,
        params: serde_json::Value,
    ) -> Self {
        Self {
            tool: tool.into(),
            operation: operation.into(),
            params,
        }
    }
}

/// Response from a tool execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    /// Whether the operation succeeded
    pub success: bool,
    /// Result data
    #[serde(default)]
    pub data: serde_json::Value,
    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Optional stdout/stderr output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl ToolResponse {
    /// Create a successful response
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
            output: None,
        }
    }

    /// Create a successful response with output
    pub fn success_with_output(data: serde_json::Value, output: impl Into<String>) -> Self {
        Self {
            success: true,
            data,
            error: None,
            output: Some(output.into()),
        }
    }

    /// Create a failure response
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error.into()),
            output: None,
        }
    }
}

/// Core trait for tool implementations
///
/// Tools provide specific capabilities like filesystem operations, git commands,
/// shell execution, etc.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool identifier
    fn id(&self) -> &str;

    /// Human-readable tool description
    fn description(&self) -> &str;

    /// List of supported operations
    fn supported_operations(&self) -> Vec<String>;

    /// Execute a tool operation
    async fn execute(&self, request: ToolRequest) -> Result<ToolResponse>;

    /// Check if this tool supports a specific operation
    fn supports_operation(&self, operation: &str) -> bool {
        self.supported_operations().iter().any(|op| op == operation)
    }
}

/// MCP (Model Context Protocol) server configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name/identifier
    pub name: String,
    /// Server type (e.g., "stdio", "http")
    pub server_type: String,
    /// Command to start the server (for stdio type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    /// URL for HTTP-based servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// Trait for MCP client implementations
///
/// MCP clients communicate with MCP servers to provide additional tools
/// and capabilities to agents.
#[async_trait]
pub trait McpClient: Send + Sync {
    /// Connect to an MCP server
    async fn connect(&mut self, config: &McpServerConfig) -> Result<()>;

    /// Disconnect from the MCP server
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// List available tools from the MCP server
    async fn list_tools(&self) -> Result<Vec<String>>;

    /// Call a tool on the MCP server
    async fn call_tool(&self, request: ToolRequest) -> Result<ToolResponse>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_request_creation() {
        let params = serde_json::json!({"path": "/tmp/test"});
        let req = ToolRequest::new("fs", "read", params.clone());

        assert_eq!(req.tool, "fs");
        assert_eq!(req.operation, "read");
        assert_eq!(req.params, params);
    }

    #[test]
    fn test_tool_response_success() {
        let data = serde_json::json!({"content": "file contents"});
        let resp = ToolResponse::success(data.clone());

        assert!(resp.success);
        assert_eq!(resp.data, data);
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_tool_response_success_with_output() {
        let data = serde_json::json!({"status": "ok"});
        let resp = ToolResponse::success_with_output(data.clone(), "Command output");

        assert!(resp.success);
        assert_eq!(resp.data, data);
        assert_eq!(resp.output, Some("Command output".to_string()));
    }

    #[test]
    fn test_tool_response_failure() {
        let resp = ToolResponse::failure("File not found");

        assert!(!resp.success);
        assert_eq!(resp.error, Some("File not found".to_string()));
        assert_eq!(resp.data, serde_json::Value::Null);
    }

    #[test]
    fn test_mcp_server_config() {
        let config = McpServerConfig {
            name: "test-server".to_string(),
            server_type: "stdio".to_string(),
            command: Some(vec!["mcp-server".to_string(), "--verbose".to_string()]),
            url: None,
            env: std::collections::HashMap::new(),
        };

        assert_eq!(config.name, "test-server");
        assert_eq!(config.server_type, "stdio");
        assert!(config.command.is_some());
    }

    // Mock tool for testing
    struct MockTool {
        id: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn id(&self) -> &str {
            &self.id
        }

        fn description(&self) -> &str {
            "Mock tool for testing"
        }

        fn supported_operations(&self) -> Vec<String> {
            vec!["read".to_string(), "write".to_string()]
        }

        async fn execute(&self, request: ToolRequest) -> Result<ToolResponse> {
            if !self.supports_operation(&request.operation) {
                return Ok(ToolResponse::failure(format!(
                    "Unsupported operation: {}",
                    request.operation
                )));
            }

            Ok(ToolResponse::success(serde_json::json!({
                "operation": request.operation,
                "completed": true
            })))
        }
    }

    #[tokio::test]
    async fn test_tool_trait() {
        let tool = MockTool {
            id: "mock-fs".to_string(),
        };

        assert_eq!(tool.id(), "mock-fs");
        assert_eq!(tool.description(), "Mock tool for testing");
        assert!(tool.supports_operation("read"));
        assert!(tool.supports_operation("write"));
        assert!(!tool.supports_operation("delete"));

        let req = ToolRequest::new("mock-fs", "read", serde_json::json!({}));
        let resp = tool.execute(req).await.unwrap();
        assert!(resp.success);
        assert_eq!(resp.data["operation"], "read");
    }

    #[tokio::test]
    async fn test_tool_unsupported_operation() {
        let tool = MockTool {
            id: "mock-fs".to_string(),
        };

        let req = ToolRequest::new("mock-fs", "delete", serde_json::json!({}));
        let resp = tool.execute(req).await.unwrap();
        assert!(!resp.success);
        assert!(resp.error.unwrap().contains("Unsupported operation"));
    }

    #[test]
    fn test_tool_request_serialization() {
        let req = ToolRequest::new("fs", "read", serde_json::json!({"path": "/tmp"}));
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ToolRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.tool, "fs");
        assert_eq!(deserialized.operation, "read");
        assert_eq!(deserialized.params["path"], "/tmp");
    }

    #[test]
    fn test_tool_response_serialization() {
        let resp = ToolResponse::success(serde_json::json!({"data": "test"}));
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ToolResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.success);
        assert_eq!(deserialized.data["data"], "test");
    }
}
