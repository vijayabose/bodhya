# Bodhya Developer Guide

**Version**: 1.0
**Last Updated**: 2025-11-16

This guide is for developers who want to extend Bodhya by creating custom agents, integrating new models, or contributing to the core platform.

## Table of Contents

1. [Development Setup](#development-setup)
2. [Architecture Overview](#architecture-overview)
3. [Creating a Custom Agent](#creating-a-custom-agent)
4. [Working with Models](#working-with-models)
5. [Tool Integration](#tool-integration)
6. [Testing Strategy](#testing-strategy)
7. [Quality Gates](#quality-gates)
8. [Contributing](#contributing)
9. [API Development](#api-development)
10. [Best Practices](#best-practices)

---

## Development Setup

### Prerequisites

```bash
# Install Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
rustup component add rustfmt clippy

# Install SQLite (for storage crate)
# Ubuntu/Debian:
sudo apt-get install libsqlite3-dev

# macOS:
brew install sqlite

# Install Git
sudo apt-get install git  # Ubuntu/Debian
brew install git           # macOS
```

### Clone and Build

```bash
# Clone repository
git clone https://github.com/vijayabose/bodhya.git
cd bodhya

# Build all crates
cargo build

# Run tests
cargo test --all

# Run quality gates
./scripts/check_all.sh
```

### Project Structure

```
bodhya/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── core/                  # Shared traits & types
│   ├── controller/            # Task routing & orchestration
│   ├── model-registry/        # Model manifest & backends
│   ├── tools-mcp/             # Tool integrations
│   ├── agent-code/            # Code generation agent
│   ├── agent-mail/            # Email writing agent
│   ├── storage/               # SQLite persistence
│   ├── cli/                   # CLI application
│   └── api-server/            # REST/WebSocket API
├── eval/                      # Evaluation harnesses
│   ├── code_agent/
│   └── mail_agent/
├── documents/                 # Design documentation
│   ├── bodhya_brd.md
│   ├── bodhya_system_design.md
│   ├── bodhya_code_design.md
│   └── ...
└── scripts/                   # Build & utility scripts
    └── check_all.sh
```

---

## Architecture Overview

### Core Concepts

**1. Agent Trait**

All agents implement the `Agent` trait from `bodhya_core`:

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    /// Unique agent identifier
    fn id(&self) -> &'static str;

    /// Agent capabilities (domain, intents, description)
    fn capability(&self) -> AgentCapability;

    /// Handle a task
    async fn handle(&self, task: Task, ctx: AgentContext)
        -> Result<AgentResult>;

    /// Whether agent is currently enabled
    fn is_enabled(&self) -> bool {
        true
    }
}
```

**2. Task Flow**

```
User Input
    ↓
CLI/API
    ↓
Controller (routes based on capabilities)
    ↓
Selected Agent (handles task)
    ↓
Model Registry (provides models)
    ↓
Tool Layer (executes operations)
    ↓
Result
```

**3. Key Types**

```rust
// Task representation
pub struct Task {
    pub id: String,
    pub description: String,
    pub domain: Option<String>,
    pub payload: serde_json::Value,
}

// Agent result
pub struct AgentResult {
    pub task_id: String,
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

// Agent capabilities
pub struct AgentCapability {
    pub domain: String,
    pub intents: Vec<String>,
    pub description: String,
}
```

---

## Creating a Custom Agent

### Step 1: Create New Crate

```bash
# Create agent crate
cargo new --lib crates/agent-myagent

# Add to workspace Cargo.toml
[workspace]
members = [
    # ... existing members ...
    "crates/agent-myagent",
]
```

### Step 2: Add Dependencies

Edit `crates/agent-myagent/Cargo.toml`:

```toml
[package]
name = "bodhya-agent-myagent"
version = "0.1.0"
edition = "2021"

[dependencies]
bodhya-core = { path = "../core" }
async-trait = "0.1"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tokio-test = "0.4"
```

### Step 3: Implement the Agent

Create `crates/agent-myagent/src/lib.rs`:

```rust
use async_trait::async_trait;
use bodhya_core::{
    Agent, AgentCapability, AgentContext, AgentResult, Task, Result,
};

/// MyAgent - Description of what this agent does
pub struct MyAgent {
    enabled: bool,
}

impl MyAgent {
    /// Create a new MyAgent
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Create with specific enabled state
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Default for MyAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for MyAgent {
    fn id(&self) -> &'static str {
        "myagent"
    }

    fn capability(&self) -> AgentCapability {
        AgentCapability {
            domain: "mydomain".to_string(),
            intents: vec![
                "generate".to_string(),
                "analyze".to_string(),
            ],
            description: "Agent that handles mydomain tasks".to_string(),
        }
    }

    async fn handle(&self, task: Task, _ctx: AgentContext)
        -> Result<AgentResult> {
        // 1. Process the task
        let result_content = self.process_task(&task.description).await?;

        // 2. Return result
        Ok(AgentResult::success(&task.id, result_content))
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl MyAgent {
    async fn process_task(&self, description: &str) -> Result<String> {
        // Your agent logic here
        Ok(format!("Processed: {}", description))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = MyAgent::new();
        assert_eq!(agent.id(), "myagent");
        assert!(agent.is_enabled());
    }

    #[test]
    fn test_capability() {
        let agent = MyAgent::new();
        let cap = agent.capability();
        assert_eq!(cap.domain, "mydomain");
        assert_eq!(cap.intents.len(), 2);
    }

    #[tokio::test]
    async fn test_handle_task() {
        let agent = MyAgent::new();
        let task = Task::new("test task");
        let ctx = AgentContext::default();

        let result = agent.handle(task, ctx).await.unwrap();
        assert!(result.success);
        assert!(result.content.is_some());
    }
}
```

### Step 4: Add to Configuration

Create `~/.bodhya/config/default.yaml`:

```yaml
agents:
  myagent:
    enabled: true
    models:
      processor: qwen2.5-7b-instruct  # Your model choice
```

### Step 5: Register in CLI

Edit `crates/cli/src/run_cmd.rs`:

```rust
use bodhya_agent_myagent::MyAgent;

pub async fn run_task(domain: Option<String>, task: String) -> Result<()> {
    // ... existing code ...

    // Add your agent
    let myagent = Arc::new(MyAgent::new()) as Arc<dyn Agent>;
    agents.push(myagent);

    // ... rest of function ...
}
```

---

## Working with Models

### Model Registry Integration

Agents request models from the ModelRegistry:

```rust
use bodhya_model_registry::ModelRegistry;
use bodhya_core::{ModelRole, ModelRequest};

// Get a model for a specific role
let registry = ModelRegistry::from_manifest("~/.bodhya/models.yaml")?;

let model_request = ModelRequest::builder()
    .role(ModelRole::Planner)
    .domain("mydomain")
    .prompt("Your prompt here")
    .build();

let response = registry
    .get_backend_for_role(ModelRole::Planner, "mydomain")?
    .generate(model_request)
    .await?;

println!("Model output: {}", response.content);
```

### Adding Custom Models

Edit `~/.bodhya/models.yaml`:

```yaml
models:
  - id: my-custom-model
    display_name: "My Custom Model"
    description: "Custom model for my domain"
    role: planner
    domain: mydomain
    backend:
      type: local
      config:
        model_path: /path/to/model.gguf
        context_size: 4096
        temperature: 0.7
    source:
      type: huggingface
      repo: myorg/mymodel
      filename: model.gguf
    checksum:
      sha256: "abc123..."
    size_bytes: 4500000000
```

### Custom Model Backend

Implement `ModelBackend` trait:

```rust
use async_trait::async_trait;
use bodhya_core::{ModelBackend, ModelRequest, ModelResponse, Result};

pub struct MyModelBackend {
    model_path: String,
}

#[async_trait]
impl ModelBackend for MyModelBackend {
    fn id(&self) -> &'static str {
        "my-backend"
    }

    async fn generate(&self, request: ModelRequest)
        -> Result<ModelResponse> {
        // Your inference logic here
        Ok(ModelResponse {
            content: "Generated content".to_string(),
            tokens: Some(100),
            finish_reason: Some("stop".to_string()),
        })
    }

    fn is_available(&self) -> bool {
        std::path::Path::new(&self.model_path).exists()
    }
}
```

---

## Tool Integration

### Using Built-in Tools

```rust
use bodhya_tools_mcp::ToolRegistry;

// Create tool registry
let mut registry = ToolRegistry::new();

// Register tools
registry.register_defaults()?;

// Execute filesystem operation
let result = registry.execute(
    "filesystem",
    "read",
    &serde_json::json!({"path": "/path/to/file.txt"})
).await?;

// Execute shell command
let result = registry.execute(
    "shell",
    "execute",
    &serde_json::json!({"command": "ls -la"})
).await?;
```

### Creating Custom Tools

```rust
use async_trait::async_trait;
use bodhya_core::{Tool, ToolRequest, ToolResponse, Result};

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn id(&self) -> &'static str {
        "mytool"
    }

    fn description(&self) -> &str {
        "Description of what my tool does"
    }

    fn supported_operations(&self) -> Vec<&'static str> {
        vec!["action1", "action2"]
    }

    async fn execute(&self, request: ToolRequest)
        -> Result<ToolResponse> {
        match request.operation.as_str() {
            "action1" => {
                // Handle action1
                Ok(ToolResponse::success("Result of action1"))
            }
            "action2" => {
                // Handle action2
                Ok(ToolResponse::success("Result of action2"))
            }
            _ => Err(anyhow::anyhow!("Unsupported operation")),
        }
    }
}
```

---

## Testing Strategy

### Unit Tests

Test individual components:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let component = MyComponent::new();
        assert_eq!(component.value(), expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = my_async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Create `tests/integration_test.rs`:

```rust
use bodhya_agent_myagent::MyAgent;
use bodhya_core::{Agent, Task, AgentContext};

#[tokio::test]
async fn test_full_workflow() {
    // Setup
    let agent = MyAgent::new();
    let task = Task::new("integration test task");
    let ctx = AgentContext::default();

    // Execute
    let result = agent.handle(task, ctx).await.unwrap();

    // Verify
    assert!(result.success);
    assert!(result.content.is_some());
}
```

### BDD Tests (using Gherkin scenarios)

See `documents/bodhya_gherkin_test_cases.md` for test scenario templates.

### Evaluation Harnesses

Create quality scoring for your agent:

```rust
// eval/my_agent/src/scorer.rs
pub struct MyAgentScore {
    pub metric1: f64,  // 0-40
    pub metric2: f64,  // 0-30
    pub metric3: f64,  // 0-30
    pub total: f64,    // 0-100
}

impl MyAgentScore {
    pub fn calculate(output: &str) -> Self {
        let metric1 = score_metric1(output);
        let metric2 = score_metric2(output);
        let metric3 = score_metric3(output);
        let total = metric1 + metric2 + metric3;

        Self { metric1, metric2, metric3, total }
    }

    pub fn passes(&self) -> bool {
        self.total >= 85.0  // Your threshold
    }
}
```

---

## Quality Gates

### Running Quality Checks

```bash
# Run all quality gates
./scripts/check_all.sh
```

This runs:
1. `cargo fmt --check` - Code formatting
2. `cargo clippy --all-targets -- -D warnings` - Lints
3. `cargo test --all` - All tests
4. `cargo audit` (optional) - Security vulnerabilities

### Pre-Commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "Running quality gates..."
./scripts/check_all.sh

echo "All checks passed! ✅"
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

### Continuous Integration

Example GitHub Actions workflow:

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Run quality gates
        run: ./scripts/check_all.sh

      - name: Upload coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
```

---

## Contributing

### Contribution Workflow

1. **Fork the repository**
   ```bash
   # Fork on GitHub, then:
   git clone https://github.com/YOUR_USERNAME/bodhya.git
   cd bodhya
   git remote add upstream https://github.com/vijayabose/bodhya.git
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/my-amazing-feature
   ```

3. **Make your changes**
   - Follow existing code style
   - Add tests for new functionality
   - Update documentation

4. **Run quality gates**
   ```bash
   ./scripts/check_all.sh
   ```

5. **Commit your changes**
   ```bash
   git commit -m "Add amazing feature

   - Implement X
   - Add tests for Y
   - Update documentation
   "
   ```

6. **Push and create PR**
   ```bash
   git push origin feature/my-amazing-feature
   # Then create PR on GitHub
   ```

### Code Style Guidelines

**Rust Style:**
- Use `rustfmt` for formatting
- Follow Rust API guidelines
- Prefer explicit error handling
- Use meaningful variable names
- Add documentation comments

**Example:**
```rust
/// Calculates the fibonacci number at position n.
///
/// # Arguments
///
/// * `n` - The position in the fibonacci sequence
///
/// # Returns
///
/// The fibonacci number at position n
///
/// # Examples
///
/// ```
/// let result = fibonacci(10);
/// assert_eq!(result, 55);
/// ```
pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

### Documentation Requirements

- Public APIs must have doc comments
- Complex logic should have inline comments
- Update relevant .md files
- Add examples where helpful

---

## API Development

### Adding New Endpoints

Edit `crates/api-server/src/routes.rs`:

```rust
/// GET /my-endpoint
pub async fn my_endpoint(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MyResponse>, ApiError> {
    // Your logic here
    Ok(Json(MyResponse { data: "..." }))
}

// Register in main.rs
let app = Router::new()
    .route("/my-endpoint", get(my_endpoint))
    // ... other routes ...
    .with_state(state);
```

### WebSocket Handlers

```rust
pub async fn ws_my_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_my_socket(socket, state))
}

async fn handle_my_socket(
    socket: WebSocket,
    state: Arc<AppState>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Handle messages
    while let Some(Ok(msg)) = receiver.next().await {
        // Process message
        let response = process_ws_message(msg).await;

        // Send response
        sender.send(Message::Text(response)).await.ok();
    }
}
```

---

## Best Practices

### 1. Error Handling

```rust
// Use Result types consistently
pub fn my_function() -> Result<String> {
    let value = risky_operation()?;
    Ok(value.to_string())
}

// Provide context for errors
pub fn my_function() -> Result<String> {
    let value = risky_operation()
        .context("Failed to perform risky operation")?;
    Ok(value.to_string())
}
```

### 2. Async Best Practices

```rust
// Use async/await appropriately
pub async fn fetch_data() -> Result<Data> {
    let response = client.get("url").await?;
    let data = response.json().await?;
    Ok(data)
}

// Don't block async runtime
pub async fn process() -> Result<()> {
    // Good: Use tokio::task::spawn_blocking for CPU-intensive work
    let result = tokio::task::spawn_blocking(|| {
        expensive_computation()
    }).await?;

    Ok(())
}
```

### 3. Testing Patterns

```rust
// Use test helpers
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent() -> MyAgent {
        MyAgent::new()
    }

    #[tokio::test]
    async fn test_with_helper() {
        let agent = create_test_agent();
        // Test using helper
    }
}
```

### 4. Dependency Injection

```rust
// Prefer dependency injection over global state
pub struct MyService {
    registry: Arc<ModelRegistry>,
    tools: Arc<ToolRegistry>,
}

impl MyService {
    pub fn new(
        registry: Arc<ModelRegistry>,
        tools: Arc<ToolRegistry>,
    ) -> Self {
        Self { registry, tools }
    }
}
```

### 5. Configuration

```rust
// Use serde for configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyConfig {
    pub enabled: bool,
    pub timeout: u64,
    pub max_retries: usize,
}

impl Default for MyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout: 30,
            max_retries: 3,
        }
    }
}
```

---

## Resources

- **Design Documents**: `documents/` directory
- **API Docs**: Run `cargo doc --open`
- **Examples**: See `crates/agent-code/` and `crates/agent-mail/`
- **Community**: https://github.com/vijayabose/bodhya/discussions
- **Issues**: https://github.com/vijayabose/bodhya/issues

---

## Getting Help

- **Questions**: Open a discussion on GitHub
- **Bugs**: Create an issue with reproduction steps
- **Features**: Propose in discussions before implementing
- **Security**: Email security concerns (don't open public issues)

---

**Happy developing! We're excited to see what you build with Bodhya.** 🚀
