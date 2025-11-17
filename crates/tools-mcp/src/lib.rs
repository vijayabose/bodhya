/// Tool and MCP integrations for Bodhya
///
/// This crate provides concrete implementations of the Tool and McpClient traits
/// defined in bodhya-core, including filesystem operations, shell command execution,
/// and MCP server integration.
// Re-export core tool types for convenience
pub use bodhya_core::{McpClient, McpServerConfig, Tool, ToolRequest, ToolResponse};

mod edit_tool;
mod fs_tool;
mod json_rpc;
mod mcp_client;
mod mcp_tool_wrapper;
mod search_tool;
mod shell_tool;
mod stdio_mcp_client;

// Re-export tool implementations
pub use edit_tool::{EditOperation, EditResult, EditTool};
pub use fs_tool::FilesystemTool;
pub use json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RequestId};
pub use mcp_client::BasicMcpClient;
pub use mcp_tool_wrapper::McpToolWrapper;
pub use search_tool::{SearchMatch, SearchResult, SearchTool};
pub use shell_tool::ShellTool;
pub use stdio_mcp_client::StdioMcpClient;

use std::sync::Arc;
use tokio::sync::Mutex;

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
    /// MCP clients (shared across tools from same server)
    mcp_clients: Vec<Arc<Mutex<Box<dyn McpClient>>>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            mcp_clients: Vec::new(),
        }
    }

    /// Create a tool registry with default tools (filesystem, shell, edit, search)
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(FilesystemTool::new()));
        registry.register(Box::new(ShellTool::new()));
        registry.register(Box::new(EditTool::new()));
        registry.register(Box::new(SearchTool::new()));
        registry
    }

    /// Load MCP servers from configuration and register their tools
    pub async fn load_mcp_servers(
        &mut self,
        servers: &[McpServerConfig],
    ) -> bodhya_core::Result<()> {
        for server in servers {
            if !server.enabled {
                continue;
            }

            // Create and connect client
            let mut client: Box<dyn McpClient> = Box::new(StdioMcpClient::new());

            match client.connect(server).await {
                Ok(_) => {
                    // List tools from server
                    match client.list_tools().await {
                        Ok(tools) => {
                            // Wrap client in Arc<Mutex> for sharing
                            let client_arc = Arc::new(Mutex::new(client));

                            // Register each tool
                            for tool_name in tools {
                                let wrapper = McpToolWrapper::new(
                                    tool_name,
                                    client_arc.clone(),
                                    server.name.clone(),
                                );
                                self.register(Box::new(wrapper));
                            }

                            // Store client reference
                            self.mcp_clients.push(client_arc);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to list tools from MCP server '{}': {}",
                                server.name, e
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to connect to MCP server '{}': {}",
                        server.name, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Get a tool by ID
    pub fn get_tool(&self, tool_id: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.id() == tool_id)
            .map(|t| t.as_ref())
    }

    /// List all registered tool IDs
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.iter().map(|t| t.id().to_string()).collect()
    }

    /// Execute a tool request
    pub async fn execute(&self, request: ToolRequest) -> bodhya_core::Result<ToolResponse> {
        let tool = self.get_tool(&request.tool).ok_or_else(|| {
            bodhya_core::Error::Tool(format!("Tool '{}' not found", request.tool))
        })?;

        tool.execute(request).await
    }

    /// Disconnect all MCP clients
    pub async fn disconnect_all(&mut self) -> bodhya_core::Result<()> {
        for client in &self.mcp_clients {
            let mut client_guard = client.lock().await;
            let _ = client_guard.disconnect().await; // Ignore errors
        }
        self.mcp_clients.clear();
        Ok(())
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.list_tools().len(), 0);
    }

    #[test]
    fn test_tool_registry_with_defaults() {
        let registry = ToolRegistry::with_defaults();
        let tools = registry.list_tools();

        assert!(tools.contains(&"filesystem".to_string()));
        assert!(tools.contains(&"shell".to_string()));
        assert!(tools.contains(&"edit".to_string()));
        assert!(tools.contains(&"search".to_string()));
    }

    #[test]
    fn test_tool_registry_get_tool() {
        let registry = ToolRegistry::with_defaults();

        let fs_tool = registry.get_tool("filesystem");
        assert!(fs_tool.is_some());
        assert_eq!(fs_tool.unwrap().id(), "filesystem");

        let nonexistent = registry.get_tool("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[tokio::test]
    async fn test_tool_registry_execute() {
        let registry = ToolRegistry::with_defaults();

        let request = ToolRequest::new(
            "shell",
            "exec",
            serde_json::json!({
                "command": "echo",
                "args": ["test"]
            }),
        );

        let result = registry.execute(request).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
    }

    #[tokio::test]
    async fn test_tool_registry_execute_nonexistent_tool() {
        let registry = ToolRegistry::with_defaults();

        let request = ToolRequest::new("nonexistent", "test", serde_json::json!({}));

        let result = registry.execute(request).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_registry_register() {
        let mut registry = ToolRegistry::new();
        assert_eq!(registry.list_tools().len(), 0);

        registry.register(Box::new(FilesystemTool::new()));
        assert_eq!(registry.list_tools().len(), 1);
        assert!(registry.get_tool("filesystem").is_some());
    }
}
