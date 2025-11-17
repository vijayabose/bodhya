/// Bodhya CLI - Main entry point
///
/// This is the command-line interface for Bodhya, providing commands for:
/// - Initialization: `bodhya init`
/// - Model management: `bodhya models list/install/remove`
/// - Task execution: `bodhya run`
/// - History: `bodhya history`
use clap::{Parser, Subcommand};
use std::process;

use bodhya_cli::config_templates::{ConfigTemplate, Profile};
use bodhya_cli::{agents_cmd, history_cmd, init_cmd, models_cmd, run_cmd, serve_cmd, tools_cmd};

#[derive(Parser)]
#[command(name = "bodhya")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Bodhya - Local-first Multi-Agent AI Platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Bodhya with a specific profile
    Init {
        /// Profile to use: code, mail, or full
        #[arg(short, long, default_value = "code")]
        profile: String,

        /// Force re-initialization even if already initialized
        #[arg(short, long)]
        force: bool,
    },

    /// Agent information and management
    #[command(subcommand)]
    Agents(AgentsCommands),

    /// Model management commands
    #[command(subcommand)]
    Models(ModelsCommands),

    /// Tool management commands
    #[command(subcommand)]
    Tools(ToolsCommands),

    /// Run a task
    Run {
        /// Domain hint for routing (code, mail, etc.)
        #[arg(short, long)]
        domain: Option<String>,

        /// Working directory for file operations
        #[arg(short, long)]
        working_dir: Option<String>,

        /// Execution mode: generate-only, execute, execute-with-retry
        #[arg(long, default_value = "execute")]
        execution_mode: String,

        /// Task description
        #[arg(required = true)]
        task: String,
    },

    /// View execution history and metrics
    #[command(subcommand)]
    History(HistoryCommands),

    /// Start API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[derive(Subcommand)]
enum AgentsCommands {
    /// List all available agents
    List,

    /// Show detailed information about a specific agent
    Show {
        /// Agent ID to show (e.g., "code", "mail")
        agent_id: String,
    },
}

#[derive(Subcommand)]
enum ModelsCommands {
    /// List all available models
    List,

    /// Install a model by ID
    Install {
        /// Model ID to install
        model_id: String,
    },

    /// Remove an installed model
    Remove {
        /// Model ID to remove
        model_id: String,
    },
}

#[derive(Subcommand)]
enum ToolsCommands {
    /// List all available tools (builtin + MCP)
    List,

    /// List configured MCP servers
    ListMcp,

    /// Add a new MCP server
    AddMcp {
        /// Server name
        name: String,

        /// Server type (stdio or http)
        #[arg(long, default_value = "stdio")]
        server_type: String,

        /// Command for stdio servers (e.g., "npx @modelcontextprotocol/server-git")
        #[arg(long)]
        command: Option<Vec<String>>,

        /// URL for HTTP servers
        #[arg(long)]
        url: Option<String>,

        /// Disable the server after adding
        #[arg(long)]
        disabled: bool,
    },

    /// Remove an MCP server
    RemoveMcp {
        /// Server name to remove
        name: String,
    },

    /// Enable or disable an MCP server
    ToggleMcp {
        /// Server name
        name: String,

        /// Enable the server
        #[arg(long)]
        enable: bool,
    },

    /// Test connection to an MCP server
    TestMcp {
        /// Server name to test
        name: String,
    },
}

#[derive(Subcommand)]
enum HistoryCommands {
    /// Show recent task execution history
    Show {
        /// Number of sessions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show statistics for a domain
    Stats {
        /// Domain to show stats for (e.g., "code", "mail")
        domain: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    // Execute command
    let result = match cli.command {
        Commands::Init { profile, force } => {
            let profile = match Profile::parse(&profile) {
                Some(p) => p,
                None => {
                    eprintln!(
                        "Error: Invalid profile '{}'. Valid profiles: code, mail, full",
                        profile
                    );
                    eprintln!("\nAvailable profiles:");
                    for p in ConfigTemplate::all_profiles() {
                        eprintln!("  {} - {}", p.as_str(), p.description());
                    }
                    process::exit(1);
                }
            };

            init_cmd::init(profile, force)
        }
        Commands::Agents(agents_cmd) => match agents_cmd {
            AgentsCommands::List => agents_cmd::list_agents(),
            AgentsCommands::Show { agent_id } => agents_cmd::show_agent(&agent_id),
        },
        Commands::Models(models_cmd) => match models_cmd {
            ModelsCommands::List => models_cmd::list_models(),
            ModelsCommands::Install { model_id } => models_cmd::install_model(&model_id),
            ModelsCommands::Remove { model_id } => models_cmd::remove_model(&model_id),
        },
        Commands::Tools(tools_cmd) => match tools_cmd {
            ToolsCommands::List => tools_cmd::list_tools().await,
            ToolsCommands::ListMcp => tools_cmd::list_mcp_servers(),
            ToolsCommands::AddMcp {
                name,
                server_type,
                command,
                url,
                disabled,
            } => tools_cmd::add_mcp_server(name, server_type, command, url, !disabled),
            ToolsCommands::RemoveMcp { name } => tools_cmd::remove_mcp_server(name),
            ToolsCommands::ToggleMcp { name, enable } => tools_cmd::toggle_mcp_server(name, enable),
            ToolsCommands::TestMcp { name } => tools_cmd::test_mcp_server(name).await,
        },
        Commands::Run {
            domain,
            working_dir,
            execution_mode,
            task,
        } => run_cmd::run_task(domain, working_dir, execution_mode, task).await,
        Commands::History(history_cmd) => match history_cmd {
            HistoryCommands::Show { limit } => history_cmd::show_history(limit),
            HistoryCommands::Stats { domain } => history_cmd::show_stats(&domain),
        },
        Commands::Serve { port, host } => serve_cmd::start_server(&host, port).await,
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: bool) {
    use tracing_subscriber::fmt::format::FmtSpan;

    let builder = tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    if verbose {
        builder
            .with_span_events(FmtSpan::CLOSE)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        builder.with_max_level(tracing::Level::INFO).init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_verify() {
        // Verify that the CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_cli_help() {
        let mut cli = Cli::command();
        let help = cli.render_help().to_string();
        assert!(help.contains("Bodhya"));
        assert!(help.contains("init"));
        assert!(help.contains("agents"));
        assert!(help.contains("models"));
        assert!(help.contains("run"));
    }

    #[test]
    fn test_agents_list_command() {
        let cli = Cli::parse_from(["bodhya", "agents", "list"]);
        match cli.command {
            Commands::Agents(AgentsCommands::List) => {}
            _ => panic!("Expected Agents List command"),
        }
    }

    #[test]
    fn test_agents_show_command() {
        let cli = Cli::parse_from(["bodhya", "agents", "show", "code"]);
        match cli.command {
            Commands::Agents(AgentsCommands::Show { agent_id }) => {
                assert_eq!(agent_id, "code");
            }
            _ => panic!("Expected Agents Show command"),
        }
    }

    #[test]
    fn test_init_command_defaults() {
        let cli = Cli::parse_from(["bodhya", "init"]);
        match cli.command {
            Commands::Init { profile, force } => {
                assert_eq!(profile, "code");
                assert!(!force);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_init_command_with_profile() {
        let cli = Cli::parse_from(["bodhya", "init", "--profile", "mail"]);
        match cli.command {
            Commands::Init { profile, force } => {
                assert_eq!(profile, "mail");
                assert!(!force);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_init_command_with_force() {
        let cli = Cli::parse_from(["bodhya", "init", "--force"]);
        match cli.command {
            Commands::Init { profile: _, force } => {
                assert!(force);
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_models_list_command() {
        let cli = Cli::parse_from(["bodhya", "models", "list"]);
        match cli.command {
            Commands::Models(ModelsCommands::List) => {}
            _ => panic!("Expected Models List command"),
        }
    }

    #[test]
    fn test_models_install_command() {
        let cli = Cli::parse_from(["bodhya", "models", "install", "test_model"]);
        match cli.command {
            Commands::Models(ModelsCommands::Install { model_id }) => {
                assert_eq!(model_id, "test_model");
            }
            _ => panic!("Expected Models Install command"),
        }
    }

    #[test]
    fn test_models_remove_command() {
        let cli = Cli::parse_from(["bodhya", "models", "remove", "test_model"]);
        match cli.command {
            Commands::Models(ModelsCommands::Remove { model_id }) => {
                assert_eq!(model_id, "test_model");
            }
            _ => panic!("Expected Models Remove command"),
        }
    }

    #[test]
    fn test_run_command() {
        let cli = Cli::parse_from(["bodhya", "run", "Generate hello world"]);
        match cli.command {
            Commands::Run {
                domain,
                working_dir,
                execution_mode,
                task,
            } => {
                assert_eq!(domain, None);
                assert_eq!(working_dir, None);
                assert_eq!(execution_mode, "execute");
                assert_eq!(task, "Generate hello world");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_command_with_domain() {
        let cli = Cli::parse_from(["bodhya", "run", "--domain", "code", "Generate code"]);
        match cli.command {
            Commands::Run {
                domain,
                working_dir,
                execution_mode,
                task,
            } => {
                assert_eq!(domain, Some("code".to_string()));
                assert_eq!(working_dir, None);
                assert_eq!(execution_mode, "execute");
                assert_eq!(task, "Generate code");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_command_with_working_dir() {
        let cli = Cli::parse_from(["bodhya", "run", "--working-dir", "/tmp", "Generate code"]);
        match cli.command {
            Commands::Run {
                domain,
                working_dir,
                execution_mode,
                task,
            } => {
                assert_eq!(domain, None);
                assert_eq!(working_dir, Some("/tmp".to_string()));
                assert_eq!(execution_mode, "execute");
                assert_eq!(task, "Generate code");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_command_with_all_flags() {
        let cli = Cli::parse_from([
            "bodhya",
            "run",
            "--domain",
            "code",
            "--working-dir",
            "/home/user/project",
            "Generate code",
        ]);
        match cli.command {
            Commands::Run {
                domain,
                working_dir,
                execution_mode,
                task,
            } => {
                assert_eq!(domain, Some("code".to_string()));
                assert_eq!(working_dir, Some("/home/user/project".to_string()));
                assert_eq!(execution_mode, "execute");
                assert_eq!(task, "Generate code");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_command_with_execution_mode() {
        let cli = Cli::parse_from([
            "bodhya",
            "run",
            "--execution-mode",
            "generate-only",
            "Generate code",
        ]);
        match cli.command {
            Commands::Run {
                domain,
                working_dir,
                execution_mode,
                task,
            } => {
                assert_eq!(domain, None);
                assert_eq!(working_dir, None);
                assert_eq!(execution_mode, "generate-only");
                assert_eq!(task, "Generate code");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::parse_from(["bodhya", "--verbose", "models", "list"]);
        assert!(cli.verbose);
    }

    #[test]
    fn test_profile_parse() {
        assert_eq!(Profile::parse("code"), Some(Profile::Code));
        assert_eq!(Profile::parse("mail"), Some(Profile::Mail));
        assert_eq!(Profile::parse("full"), Some(Profile::Full));
        assert_eq!(Profile::parse("invalid"), None);
    }
}
