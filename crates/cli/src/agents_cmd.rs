/// Agent information and listing command
use bodhya_agent_code::CodeAgent;
use bodhya_agent_mail::MailAgent;
use bodhya_core::{Agent, Result};

/// List all available agents and their capabilities
pub fn list_agents() -> Result<()> {
    println!("\nAvailable Agents:");
    println!("{}", "=".repeat(80));

    // Create agent instances
    let code_agent = CodeAgent::new();
    let mail_agent = MailAgent::new();

    let agents: Vec<Box<dyn Agent>> = vec![Box::new(code_agent), Box::new(mail_agent)];

    for agent in agents {
        let cap = agent.capability();
        let status = if agent.is_enabled() {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        };

        println!("\n{} ({})", cap.domain.to_uppercase(), agent.id());
        println!("  Status: {}", status);
        println!("  Description: {}", cap.description);
        println!("  Capabilities:");
        for intent in &cap.intents {
            println!("    - {}", intent);
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("\nUse 'bodhya run --domain <domain> --task <description>' to execute tasks");

    Ok(())
}

/// Show detailed information about a specific agent
pub fn show_agent(agent_id: &str) -> Result<()> {
    let agent: Box<dyn Agent> = match agent_id.to_lowercase().as_str() {
        "code" | "codeagent" => Box::new(CodeAgent::new()),
        "mail" | "mailagent" => Box::new(MailAgent::new()),
        _ => {
            return Err(bodhya_core::Error::InvalidInput(format!(
                "Unknown agent: {}. Available agents: code, mail",
                agent_id
            )))
        }
    };

    let cap = agent.capability();
    let status = if agent.is_enabled() {
        "✓ Enabled"
    } else {
        "✗ Disabled"
    };

    println!("\nAgent Details: {}", cap.domain.to_uppercase());
    println!("{}", "=".repeat(80));
    println!("ID:          {}", agent.id());
    println!("Status:      {}", status);
    println!("Domain:      {}", cap.domain);
    println!("Description: {}", cap.description);
    println!("\nCapabilities:");
    for intent in &cap.intents {
        println!("  - {}", intent);
    }

    println!("\n{}", "=".repeat(80));
    println!("\nExample Usage:");
    println!(
        "  bodhya run --domain {} --task \"<your task description>\"",
        cap.domain
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_agents() {
        let result = list_agents();
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_agent_code() {
        let result = show_agent("code");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_agent_mail() {
        let result = show_agent("mail");
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_agent_invalid() {
        let result = show_agent("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_show_agent_case_insensitive() {
        assert!(show_agent("CODE").is_ok());
        assert!(show_agent("Mail").is_ok());
        assert!(show_agent("CodeAgent").is_ok());
    }
}
