# Tool Extensibility Design - MCP Server Integration

**Version**: 1.0
**Created**: 2025-11-16
**Status**: Design Proposal
**Relates To**: Tool Integration Plan v1.1

---

## Overview

This document describes how to extend Bodhya's tool system via CLI to support:
1. **MCP Server Management**: Add/remove/list MCP servers
2. **Custom Tool Registration**: Register custom tools via configuration
3. **Tool Discovery**: List available tools from all sources
4. **Runtime Tool Loading**: Load tools dynamically

---

## Current State

### Existing Infrastructure ✅
- `McpClient` trait defined
- `McpServerConfig` struct for server configuration
- `ToolRegistry` with dynamic registration
- Stub `BasicMcpClient` implementation

### Gaps ❌
- No CLI commands for MCP management
- No config file support for MCP servers
- MCP client doesn't actually connect
- No tool discovery mechanism

---

## Architecture Design

### 1. Configuration Schema

**Add to `~/.bodhya/config/default.yaml`:**

```yaml
# Bodhya Configuration
profile: full
engagement_mode: minimum

# Tool Configuration (NEW)
tools:
  # Built-in tools (always available)
  builtin:
    - filesystem
    - shell
    - edit      # v1.1+
    - search    # v1.1+
    - git       # v1.1+

  # MCP Servers (external tool providers)
  mcp_servers:
    # Example: Filesystem MCP server
    - name: filesystem-extended
      type: stdio
      command: ["npx", "-y", "@modelcontextprotocol/server-filesystem", "/path/to/allow"]
      enabled: true
      env:
        NODE_ENV: production

    # Example: GitHub MCP server
    - name: github
      type: stdio
      command: ["npx", "-y", "@modelcontextprotocol/server-github"]
      enabled: true
      env:
        GITHUB_TOKEN: "${GITHUB_TOKEN}"  # Read from environment

    # Example: Brave Search MCP server
    - name: brave-search
      type: stdio
      command: ["npx", "-y", "@modelcontextprotocol/server-brave-search"]
      enabled: false  # Disabled by default
      env:
        BRAVE_API_KEY: "${BRAVE_API_KEY}"

    # Example: Custom HTTP-based MCP server
    - name: custom-api
      type: http
      url: "http://localhost:8080/mcp"
      enabled: true
      headers:
        Authorization: "Bearer ${API_TOKEN}"

# Existing config continues...
agents:
  code:
    enabled: true
    # ...
```

### 2. Code Structure Changes

**Update `crates/core/src/config.rs`:**

```rust
/// Tool configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Built-in tools to enable
    #[serde(default = "default_builtin_tools")]
    pub builtin: Vec<String>,

    /// MCP server configurations
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
}

fn default_builtin_tools() -> Vec<String> {
    vec![
        "filesystem".to_string(),
        "shell".to_string(),
    ]
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            builtin: default_builtin_tools(),
            mcp_servers: Vec::new(),
        }
    }
}

// Add to AppConfig
pub struct AppConfig {
    pub profile: String,
    pub engagement_mode: EngagementMode,
    pub agents: HashMap<String, AgentConfig>,
    pub models: ModelConfigs,
    pub paths: PathsConfig,
    pub logging: LoggingConfig,
    pub tools: ToolsConfig,  // NEW
}
```

**Enhance `McpServerConfig` in `crates/core/src/tool.rs`:**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub server_type: String,  // "stdio" or "http"

    // For stdio servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    // For HTTP servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    // Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    // Enable/disable
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }
```

### 3. Full MCP Client Implementation

**Replace stub in `crates/tools-mcp/src/mcp_client.rs`:**

```rust
use async_trait::async_trait;
use bodhya_core::{McpClient, McpServerConfig, Result, ToolRequest, ToolResponse};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct StdioMcpClient {
    config: Option<McpServerConfig>,
    process: Option<Arc<Mutex<Child>>>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    stdout: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
    available_tools: Vec<McpToolInfo>,
}

#[derive(Clone, Debug)]
struct McpToolInfo {
    name: String,
    description: String,
    input_schema: Value,
}

impl StdioMcpClient {
    pub fn new() -> Self {
        Self {
            config: None,
            process: None,
            stdin: None,
            stdout: None,
            available_tools: Vec::new(),
        }
    }

    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        // Implement JSON-RPC 2.0 protocol
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        // Send via stdin
        let mut stdin = self.stdin.as_ref()
            .ok_or_else(|| bodhya_core::Error::Tool("Not connected".into()))?
            .lock().await;

        let request_str = serde_json::to_string(&request)?;
        stdin.write_all(request_str.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        // Read response from stdout
        let mut stdout = self.stdout.as_ref()
            .ok_or_else(|| bodhya_core::Error::Tool("Not connected".into()))?
            .lock().await;

        let mut response_line = String::new();
        stdout.read_line(&mut response_line).await?;

        let response: Value = serde_json::from_str(&response_line)?;

        // Check for error
        if let Some(error) = response.get("error") {
            return Err(bodhya_core::Error::Tool(
                format!("MCP error: {}", error)
            ));
        }

        Ok(response["result"].clone())
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn connect(&mut self, config: &McpServerConfig) -> Result<()> {
        if config.server_type != "stdio" {
            return Err(bodhya_core::Error::Tool(
                "StdioMcpClient only supports stdio servers".into()
            ));
        }

        let command = config.command.as_ref()
            .ok_or_else(|| bodhya_core::Error::Tool("No command specified".into()))?;

        // Spawn the MCP server process
        let mut cmd = Command::new(&command[0]);
        if command.len() > 1 {
            cmd.args(&command[1..]);
        }

        // Set environment variables
        for (key, value) in &config.env {
            // Expand environment variables like ${VAR}
            let expanded = expand_env_var(value);
            cmd.env(key, expanded);
        }

        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;

        // Extract stdin/stdout
        let stdin = child.stdin.take()
            .ok_or_else(|| bodhya_core::Error::Tool("Failed to get stdin".into()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| bodhya_core::Error::Tool("Failed to get stdout".into()))?;

        self.stdin = Some(Arc::new(Mutex::new(stdin)));
        self.stdout = Some(Arc::new(Mutex::new(BufReader::new(stdout))));
        self.process = Some(Arc::new(Mutex::new(child)));
        self.config = Some(config.clone());

        // Initialize connection
        let result = self.send_request("initialize", json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "bodhya",
                "version": "1.1.0"
            }
        })).await?;

        // Discover available tools
        self.discover_tools().await?;

        tracing::info!("Connected to MCP server '{}' ({} tools available)",
            config.name, self.available_tools.len());

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(process) = self.process.take() {
            let mut child = process.lock().await;
            child.kill().await?;
        }

        self.stdin = None;
        self.stdout = None;
        self.available_tools.clear();
        self.config = None;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.process.is_some()
    }

    async fn list_tools(&self) -> Result<Vec<String>> {
        Ok(self.available_tools.iter()
            .map(|t| t.name.clone())
            .collect())
    }

    async fn call_tool(&self, request: ToolRequest) -> Result<ToolResponse> {
        let result = self.send_request("tools/call", json!({
            "name": request.operation,
            "arguments": request.params
        })).await?;

        // Parse MCP response into ToolResponse
        if let Some(content) = result.get("content") {
            Ok(ToolResponse::success(content.clone()))
        } else {
            Ok(ToolResponse::failure("No content in MCP response"))
        }
    }
}

impl StdioMcpClient {
    async fn discover_tools(&mut self) -> Result<()> {
        let result = self.send_request("tools/list", json!({})).await?;

        if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
            self.available_tools = tools.iter()
                .filter_map(|tool| {
                    Some(McpToolInfo {
                        name: tool.get("name")?.as_str()?.to_string(),
                        description: tool.get("description")?.as_str()?.to_string(),
                        input_schema: tool.get("inputSchema")?.clone(),
                    })
                })
                .collect();
        }

        Ok(())
    }
}

fn expand_env_var(value: &str) -> String {
    // Simple ${VAR} expansion
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
    re.replace_all(value, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_default()
    }).to_string()
}
```

### 4. CLI Commands

**Add to `crates/cli/src/tools_cmd.rs` (NEW FILE):**

```rust
/// CLI commands for tool management
use anyhow::Result;
use bodhya_core::{AppConfig, McpServerConfig};
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ToolsCommand {
    /// List all available tools
    List {
        /// Show tools from MCP servers
        #[arg(long)]
        mcp: bool,
    },

    /// Add a new MCP server
    AddMcp {
        /// Server name
        name: String,

        /// Server type (stdio or http)
        #[arg(long, default_value = "stdio")]
        server_type: String,

        /// Command to run (for stdio servers)
        #[arg(long)]
        command: Option<String>,

        /// URL (for http servers)
        #[arg(long)]
        url: Option<String>,

        /// Enable immediately
        #[arg(long)]
        enable: bool,
    },

    /// Remove an MCP server
    RemoveMcp {
        /// Server name
        name: String,
    },

    /// Enable/disable an MCP server
    ToggleMcp {
        /// Server name
        name: String,

        /// Enable or disable
        #[arg(long)]
        enable: bool,
    },

    /// List configured MCP servers
    ListMcp,

    /// Test connection to an MCP server
    TestMcp {
        /// Server name
        name: String,
    },
}

pub async fn execute(cmd: ToolsCommand, config_path: &str) -> Result<()> {
    match cmd {
        ToolsCommand::List { mcp } => list_tools(mcp).await,
        ToolsCommand::AddMcp { name, server_type, command, url, enable } => {
            add_mcp_server(config_path, name, server_type, command, url, enable).await
        }
        ToolsCommand::RemoveMcp { name } => remove_mcp_server(config_path, name).await,
        ToolsCommand::ToggleMcp { name, enable } => toggle_mcp_server(config_path, name, enable).await,
        ToolsCommand::ListMcp => list_mcp_servers(config_path).await,
        ToolsCommand::TestMcp { name } => test_mcp_server(config_path, name).await,
    }
}

async fn list_tools(include_mcp: bool) -> Result<()> {
    println!("Built-in Tools:");
    println!("  • filesystem - File operations (read, write, list)");
    println!("  • shell - Command execution");
    println!("  • edit - Smart file editing (v1.1+)");
    println!("  • search - Code search and grep (v1.1+)");
    println!("  • git - Git operations (v1.1+)");

    if include_mcp {
        println!("\nMCP Server Tools:");
        // TODO: Connect to each enabled MCP server and list their tools
        println!("  (Use 'bodhya tools list-mcp' to see configured servers)");
    }

    Ok(())
}

async fn add_mcp_server(
    config_path: &str,
    name: String,
    server_type: String,
    command: Option<String>,
    url: Option<String>,
    enable: bool,
) -> Result<()> {
    let mut config = AppConfig::from_file(config_path)?;

    // Parse command string into Vec<String>
    let command_vec = command.map(|cmd| {
        shell_words::split(&cmd).expect("Failed to parse command")
    });

    let mcp_config = McpServerConfig {
        name: name.clone(),
        server_type,
        command: command_vec,
        url,
        headers: None,
        env: std::collections::HashMap::new(),
        enabled: enable,
    };

    config.tools.mcp_servers.push(mcp_config);
    config.to_file(config_path)?;

    println!("✓ Added MCP server '{}'", name);
    if enable {
        println!("  Server is enabled and will be loaded on next run");
    } else {
        println!("  Server is disabled. Use 'bodhya tools toggle-mcp {}' to enable", name);
    }

    Ok(())
}

async fn list_mcp_servers(config_path: &str) -> Result<()> {
    let config = AppConfig::from_file(config_path)?;

    if config.tools.mcp_servers.is_empty() {
        println!("No MCP servers configured.");
        println!("\nTo add one, use:");
        println!("  bodhya tools add-mcp <name> --command 'npx @modelcontextprotocol/server-filesystem'");
        return Ok(());
    }

    println!("Configured MCP Servers:\n");
    for server in &config.tools.mcp_servers {
        let status = if server.enabled { "✓" } else { "✗" };
        println!("{} {} ({})", status, server.name, server.server_type);

        if let Some(cmd) = &server.command {
            println!("    Command: {}", cmd.join(" "));
        }
        if let Some(url) = &server.url {
            println!("    URL: {}", url);
        }
        if !server.env.is_empty() {
            println!("    Environment: {} variables", server.env.len());
        }
        println!();
    }

    Ok(())
}

async fn test_mcp_server(config_path: &str, name: String) -> Result<()> {
    use bodhya_tools_mcp::StdioMcpClient;
    use bodhya_core::McpClient;

    let config = AppConfig::from_file(config_path)?;

    let server_config = config.tools.mcp_servers.iter()
        .find(|s| s.name == name)
        .ok_or_else(|| anyhow::anyhow!("MCP server '{}' not found", name))?;

    println!("Testing connection to '{}'...", name);

    let mut client = StdioMcpClient::new();

    match client.connect(server_config).await {
        Ok(_) => {
            println!("✓ Successfully connected!");

            let tools = client.list_tools().await?;
            println!("\nAvailable tools ({}):", tools.len());
            for tool in tools {
                println!("  • {}", tool);
            }

            client.disconnect().await?;
            Ok(())
        }
        Err(e) => {
            println!("✗ Connection failed: {}", e);
            Err(e.into())
        }
    }
}
```

**Update `crates/cli/src/main.rs`:**

```rust
mod tools_cmd;  // Add this

#[derive(Debug, Subcommand)]
enum Commands {
    Init(init_cmd::InitArgs),
    Run(run_cmd::RunArgs),
    Models(models_cmd::ModelsCommand),
    History(history_cmd::HistoryCommand),
    Serve(serve_cmd::ServeCommand),
    Tools(tools_cmd::ToolsCommand),  // NEW
}

// In execute_command():
Commands::Tools(cmd) => {
    tools_cmd::execute(cmd, &config_path).await
}
```

---

## Usage Examples

### 1. List Available Tools

```bash
# List built-in tools
bodhya tools list

# List all tools including MCP servers
bodhya tools list --mcp
```

### 2. Add MCP Server

```bash
# Add filesystem MCP server
bodhya tools add-mcp filesystem-extended \
  --command "npx -y @modelcontextprotocol/server-filesystem /path/to/allow" \
  --enable

# Add GitHub MCP server
bodhya tools add-mcp github \
  --command "npx -y @modelcontextprotocol/server-github" \
  --enable

# Add custom HTTP MCP server
bodhya tools add-mcp my-api \
  --server-type http \
  --url "http://localhost:8080/mcp" \
  --enable
```

### 3. Manage MCP Servers

```bash
# List configured servers
bodhya tools list-mcp

# Test connection
bodhya tools test-mcp github

# Enable/disable
bodhya tools toggle-mcp github --enable
bodhya tools toggle-mcp brave-search --enable=false

# Remove server
bodhya tools remove-mcp old-server
```

### 4. Use MCP Tools in Agents

Once MCP servers are configured, agents automatically have access:

```bash
# CodeAgent can now use GitHub tools
bodhya run --domain code --task "Create a GitHub issue for bug #123"

# With working directory
bodhya run --domain code --working-dir /path/to/repo \
  --task "Search for TODO comments and create GitHub issues"
```

---

## Implementation Checklist

### Phase 1: Configuration (1 day)
- [ ] Add `ToolsConfig` to `core/src/config.rs`
- [ ] Enhance `McpServerConfig` with headers, enabled flag
- [ ] Add `tools` field to `AppConfig`
- [ ] Write tests for config serialization
- [ ] Update config templates

### Phase 2: Full MCP Client (2-3 days)
- [ ] Implement `StdioMcpClient` with JSON-RPC protocol
- [ ] Add environment variable expansion
- [ ] Implement tool discovery
- [ ] Add error handling and logging
- [ ] Write comprehensive tests
- [ ] Add `HttpMcpClient` for HTTP servers (optional)

### Phase 3: CLI Commands (1-2 days)
- [ ] Create `tools_cmd.rs` with all subcommands
- [ ] Implement `list`, `add-mcp`, `remove-mcp`
- [ ] Implement `toggle-mcp`, `list-mcp`, `test-mcp`
- [ ] Add to main CLI router
- [ ] Write CLI integration tests

### Phase 4: Integration (1 day)
- [ ] Update `ToolRegistry` to load MCP servers from config
- [ ] Update `Controller` to initialize MCP clients
- [ ] Add MCP tools to `AgentContext`
- [ ] Test end-to-end with real MCP server
- [ ] Write integration tests

### Phase 5: Documentation (1 day)
- [ ] Update user guide with MCP examples
- [ ] Create MCP server configuration guide
- [ ] Add troubleshooting section
- [ ] Document available MCP servers
- [ ] Add examples to README

---

## Available MCP Servers

Users can integrate these official MCP servers:

1. **Filesystem** - File operations
   ```bash
   bodhya tools add-mcp fs --command "npx @modelcontextprotocol/server-filesystem /path" --enable
   ```

2. **GitHub** - GitHub API access
   ```bash
   bodhya tools add-mcp github --command "npx @modelcontextprotocol/server-github" --enable
   ```

3. **GitLab** - GitLab API access
   ```bash
   bodhya tools add-mcp gitlab --command "npx @modelcontextprotocol/server-gitlab" --enable
   ```

4. **Brave Search** - Web search
   ```bash
   bodhya tools add-mcp search --command "npx @modelcontextprotocol/server-brave-search" --enable
   ```

5. **Postgres** - Database access
   ```bash
   bodhya tools add-mcp db --command "npx @modelcontextprotocol/server-postgres" --enable
   ```

Plus any custom MCP servers users create!

---

## Security Considerations

1. **Environment Variable Expansion**: Only expand `${VAR}` syntax, validate values
2. **Command Execution**: Validate command paths, no arbitrary code execution
3. **Sandboxing**: MCP servers run in separate processes
4. **Permissions**: Explicitly list allowed paths for filesystem servers
5. **Secrets**: Store API keys in environment, not config files

---

## Testing Strategy

1. **Unit Tests**: Config serialization, command parsing
2. **Integration Tests**: MCP client protocol, tool discovery
3. **End-to-End Tests**: Full workflow with mock MCP server
4. **Manual Tests**: Test with real MCP servers (GitHub, filesystem)

---

## Timeline

- **Week 1**: Configuration + MCP client implementation
- **Week 2**: CLI commands + integration
- **Week 3**: Testing + documentation
- **Total**: 3 weeks (can run parallel with Tool Integration Plan)

---

## Benefits

1. **Extensibility**: Users can add any MCP-compatible tool
2. **No Code Changes**: Add tools via CLI/config only
3. **Ecosystem**: Leverage existing MCP server ecosystem
4. **Flexibility**: Support both stdio and HTTP servers
5. **Security**: Sandboxed execution, explicit permissions

---

## Future Enhancements

1. **Tool Marketplace**: Browse/install MCP servers from catalog
2. **Auto-Discovery**: Scan for locally installed MCP servers
3. **Tool Versioning**: Manage MCP server versions
4. **Custom Tools**: Create MCP servers from Rust traits
5. **Tool Composition**: Combine multiple tools into workflows

---

**End of Tool Extensibility Design**
