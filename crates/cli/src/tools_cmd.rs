/// Tool management commands
///
/// This module implements the `bodhya tools` command for managing tools and MCP servers.
use bodhya_core::{AppConfig, McpServerConfig, Result};
use bodhya_tools_mcp::{McpClient, StdioMcpClient, ToolRegistry};
use std::path::PathBuf;

use crate::utils;

/// List all available tools (builtin + MCP)
pub async fn list_tools() -> Result<()> {
    // Load config to get MCP servers
    let config = load_config()?;

    // Create registry with defaults
    let registry = ToolRegistry::with_defaults();
    let builtin_tools = registry.list_tools();

    println!("Built-in Tools:");
    for tool in &builtin_tools {
        println!("  • {}", tool);
    }

    // List MCP servers and their tools
    let mcp_servers = config.tools.enabled_mcp_servers();
    if !mcp_servers.is_empty() {
        println!("\nMCP Server Tools:");
        for server in mcp_servers {
            println!("  {} ({})", server.name, server.server_type);

            // Try to connect and list tools
            let mut client = StdioMcpClient::new();
            match client.connect(server).await {
                Ok(_) => {
                    match client.list_tools().await {
                        Ok(tools) => {
                            for tool in tools {
                                println!("    • {}", tool);
                            }
                        }
                        Err(e) => println!("    Error listing tools: {}", e),
                    }
                    let _ = client.disconnect().await;
                }
                Err(e) => println!("    Error connecting: {}", e),
            }
        }
    }

    println!("\nTotal built-in tools: {}", builtin_tools.len());
    Ok(())
}

/// List configured MCP servers
pub fn list_mcp_servers() -> Result<()> {
    let config = load_config()?;

    if config.tools.mcp_servers.is_empty() {
        println!("No MCP servers configured.");
        return Ok(());
    }

    println!("Configured MCP Servers:\n");
    for server in &config.tools.mcp_servers {
        let status = if server.enabled {
            "enabled"
        } else {
            "disabled"
        };
        println!("  {} ({}) - {}", server.name, server.server_type, status);

        if let Some(cmd) = &server.command {
            println!("    Command: {}", cmd.join(" "));
        }

        if let Some(url) = &server.url {
            println!("    URL: {}", url);
        }

        if !server.env.is_empty() {
            println!("    Environment variables: {}", server.env.len());
        }

        println!();
    }

    Ok(())
}

/// Add a new MCP server to configuration
pub fn add_mcp_server(
    name: String,
    server_type: String,
    command: Option<Vec<String>>,
    url: Option<String>,
    enabled: bool,
) -> Result<()> {
    let config_path = utils::default_config_path()?;
    let mut config = load_config()?;

    // Check if server already exists
    if config.tools.find_mcp_server(&name).is_some() {
        return Err(bodhya_core::Error::Config(format!(
            "MCP server '{}' already exists",
            name
        )));
    }

    // Create new server config
    let server = if server_type == "stdio" {
        if let Some(cmd) = command {
            McpServerConfig::new_stdio(name, cmd).with_enabled(enabled)
        } else {
            return Err(bodhya_core::Error::Config(
                "Command required for stdio MCP server".to_string(),
            ));
        }
    } else if server_type == "http" {
        if let Some(u) = url {
            McpServerConfig::new_http(name, u).with_enabled(enabled)
        } else {
            return Err(bodhya_core::Error::Config(
                "URL required for http MCP server".to_string(),
            ));
        }
    } else {
        return Err(bodhya_core::Error::Config(format!(
            "Unsupported server type: {}",
            server_type
        )));
    };

    // Add to config
    config.tools.mcp_servers.push(server.clone());

    // Save config
    save_config(&config_path, &config)?;

    println!("✓ Added MCP server '{}'", server.name);
    Ok(())
}

/// Remove an MCP server from configuration
pub fn remove_mcp_server(name: String) -> Result<()> {
    let config_path = utils::default_config_path()?;
    let mut config = load_config()?;

    // Find and remove server
    let initial_len = config.tools.mcp_servers.len();
    config.tools.mcp_servers.retain(|s| s.name != name);

    if config.tools.mcp_servers.len() == initial_len {
        return Err(bodhya_core::Error::Config(format!(
            "MCP server '{}' not found",
            name
        )));
    }

    // Save config
    save_config(&config_path, &config)?;

    println!("✓ Removed MCP server '{}'", name);
    Ok(())
}

/// Toggle MCP server enabled/disabled
pub fn toggle_mcp_server(name: String, enable: bool) -> Result<()> {
    let config_path = utils::default_config_path()?;
    let mut config = load_config()?;

    // Find and toggle server
    if let Some(server) = config.tools.find_mcp_server_mut(&name) {
        server.enabled = enable;

        // Save config
        save_config(&config_path, &config)?;

        let action = if enable { "enabled" } else { "disabled" };
        println!("✓ {} MCP server '{}'", action, name);
        Ok(())
    } else {
        Err(bodhya_core::Error::Config(format!(
            "MCP server '{}' not found",
            name
        )))
    }
}

/// Test connection to an MCP server
pub async fn test_mcp_server(name: String) -> Result<()> {
    let config = load_config()?;

    // Find server
    let server = config
        .tools
        .find_mcp_server(&name)
        .ok_or_else(|| bodhya_core::Error::Config(format!("MCP server '{}' not found", name)))?;

    println!("Testing connection to MCP server '{}'...", name);
    println!("  Type: {}", server.server_type);

    if let Some(cmd) = &server.command {
        println!("  Command: {}", cmd.join(" "));
    }

    // Try to connect
    let mut client = StdioMcpClient::new();
    match client.connect(server).await {
        Ok(_) => {
            println!("✓ Connection successful");

            // Try to list tools
            match client.list_tools().await {
                Ok(tools) => {
                    println!("✓ Found {} tools:", tools.len());
                    for tool in tools {
                        println!("    • {}", tool);
                    }
                }
                Err(e) => {
                    println!("✗ Error listing tools: {}", e);
                }
            }

            // Disconnect
            client.disconnect().await?;
            Ok(())
        }
        Err(e) => {
            println!("✗ Connection failed: {}", e);
            Err(e)
        }
    }
}

/// Load configuration from default path
fn load_config() -> Result<AppConfig> {
    let config_path = utils::default_config_path()?;

    if !config_path.exists() {
        return Err(bodhya_core::Error::Config(
            "Configuration file not found. Run 'bodhya init' first.".to_string(),
        ));
    }

    AppConfig::from_file(config_path)
}

/// Save configuration to file
fn save_config(path: &PathBuf, config: &AppConfig) -> Result<()> {
    config.to_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn with_temp_home<F>(f: F)
    where
        F: FnOnce(&TempDir),
    {
        let temp_home = TempDir::new().unwrap();
        let old_home = env::var("HOME").ok();

        env::set_var("HOME", temp_home.path());

        f(&temp_home);

        if let Some(home) = old_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    fn setup_test_config(temp_home: &TempDir) {
        let bodhya_home = temp_home.path().join(".bodhya");
        let config_dir = bodhya_home.join("config");
        std::fs::create_dir_all(&config_dir).unwrap();

        let mut config = AppConfig::default();
        config.tools.mcp_servers.push(McpServerConfig::new_stdio(
            "test-server",
            vec!["echo".to_string()],
        ));

        let config_yaml = serde_yaml::to_string(&config).unwrap();
        let config_path = config_dir.join("default.yaml");
        std::fs::write(config_path, config_yaml).unwrap();
    }

    #[test]
    #[ignore] // TODO: Fix HOME env var mocking for concurrent tests
    fn test_list_mcp_servers_empty() {
        with_temp_home(|temp_home| {
            let bodhya_home = temp_home.path().join(".bodhya");
            let config_dir = bodhya_home.join("config");
            std::fs::create_dir_all(&config_dir).unwrap();

            let config = AppConfig::default();
            let config_yaml = serde_yaml::to_string(&config).unwrap();
            let config_path = config_dir.join("default.yaml");
            std::fs::write(config_path, config_yaml).unwrap();

            let result = list_mcp_servers();
            assert!(result.is_ok());
        });
    }

    #[test]
    #[ignore] // TODO: Fix HOME env var mocking for concurrent tests
    fn test_add_mcp_server_stdio() {
        with_temp_home(|temp_home| {
            let bodhya_home = temp_home.path().join(".bodhya");
            let config_dir = bodhya_home.join("config");
            std::fs::create_dir_all(&config_dir).unwrap();

            let config = AppConfig::default();
            let config_yaml = serde_yaml::to_string(&config).unwrap();
            let config_path = config_dir.join("default.yaml");
            std::fs::write(config_path, config_yaml).unwrap();

            let result = add_mcp_server(
                "new-server".to_string(),
                "stdio".to_string(),
                Some(vec!["test-cmd".to_string()]),
                None,
                true,
            );
            assert!(result.is_ok());

            // Verify server was added
            let config = load_config().unwrap();
            assert!(config.tools.find_mcp_server("new-server").is_some());
        });
    }

    #[test]
    #[ignore] // TODO: Fix HOME env var mocking for concurrent tests
    fn test_remove_mcp_server() {
        with_temp_home(|temp_home| {
            setup_test_config(temp_home);

            let result = remove_mcp_server("test-server".to_string());
            assert!(result.is_ok());

            // Verify server was removed
            let config = load_config().unwrap();
            assert!(config.tools.find_mcp_server("test-server").is_none());
        });
    }

    #[test]
    #[ignore] // TODO: Fix HOME env var mocking for concurrent tests
    fn test_toggle_mcp_server() {
        with_temp_home(|temp_home| {
            setup_test_config(temp_home);

            let result = toggle_mcp_server("test-server".to_string(), false);
            assert!(result.is_ok());

            // Verify server was disabled
            let config = load_config().unwrap();
            let server = config.tools.find_mcp_server("test-server").unwrap();
            assert!(!server.enabled);
        });
    }
}
