# BODHYA – Code Design (Rust Workspace)

## 1. Workspace Layout

```text
bodhya/
├─ Cargo.toml
├─ scripts/
│   └─ check_all.sh          # quality gate script
├─ eval/
│   ├─ code_agent/           # evaluation tasks for CodeAgent
│   └─ mail_agent/           # evaluation tasks for MailAgent
└─ crates/
   ├─ core/                 # shared traits & types
   │   ├─ src/
   │   │   ├─ agent.rs      # Agent trait, Task, Result, AgentCapability
   │   │   ├─ tool.rs       # Tool/MCP interfaces
   │   │   ├─ model.rs      # ModelBackend, ModelRole, EngagementMode
   │   │   ├─ config.rs     # global config structs
   │   │   └─ errors.rs
   │   └─ Cargo.toml
   │
   ├─ controller/           # central controller agent
   │   ├─ src/
   │   │   ├─ lib.rs
   │   │   ├─ routing.rs    # choose domain agent using capabilities
   │   │   ├─ engagement.rs # local vs remote decision (v1: local-only)
   │   │   └─ orchestrator.rs   # run pipeline, logging
   │   └─ Cargo.toml
   │
   ├─ model-registry/
   │   ├─ src/
   │   │   ├─ lib.rs
   │   │   ├─ manifest.rs       # parse models.yaml
   │   │   ├─ local_mistral.rs  # mistral.rs integration
   │   │   ├─ remote_stub.rs    # placeholders for remote APIs
   │   │   └─ registry.rs       # mapping (domain, role) → backend
   │   └─ Cargo.toml
   │
   ├─ tools-mcp/
   │   ├─ src/
   │   │   ├─ lib.rs
   │   │   ├─ mcp_client.rs     # generic MCP client
   │   │   ├─ fs_tool.rs        # filesystem MCP
   │   │   └─ shell_tool.rs     # command exec (e.g., cargo)
   │   └─ Cargo.toml
   │
   ├─ agent-code/           # Bodhya.CodeAgent
   │   ├─ src/
   │   │   ├─ lib.rs
   │   │   ├─ planner.rs        # call planner model
   │   │   ├─ bdd.rs            # BDD/Gherkin
   │   │   ├─ tdd.rs            # TDD loop
   │   │   ├─ impl_gen.rs       # code generator
   │   │   ├─ review.rs         # review/refine
   │   │   └─ validate.rs       # clippy, tests, coverage
   │   └─ Cargo.toml
   │
   ├─ agent-mail/           # Bodhya.MailAgent
   │   ├─ src/
   │   │   ├─ lib.rs
   │   │   ├─ draft.rs          # initial draft
   │   │   ├─ refine.rs         # tone/clarity
   │   │   └─ classify.rs       # (future) classification/policy
   │   └─ Cargo.toml
   │
   ├─ storage/              # optional: metrics, sessions
   │   ├─ src/
   │   │   ├─ lib.rs
   │   │   ├─ sqlite.rs
   │   │   └─ models.rs
   │   └─ Cargo.toml
   │
   ├─ cli/                  # user-facing CLI
   │   ├─ src/
   │   │   ├─ main.rs
   │   │   ├─ commands.rs       # run-task, list-agents, show-config
   │   │   ├─ models_cmd.rs     # models list/install/remove
   │   │   └─ init_cmd.rs       # bodhya init
   │   └─ Cargo.toml
   │
   └─ api-server/           # optional REST/WebSocket
       ├─ src/
       │   ├─ main.rs
       │   └─ routes.rs
       └─ Cargo.toml
```

## 2. Core Traits (Sketch)

```rust
// core::agent

use async_trait::async_trait;
use serde_json::Value;

pub struct Task {
    pub id: String,
    pub domain_hint: Option<String>,
    pub description: String,
    pub payload: Value,
}

pub struct AgentResult {
    pub task_id: String,
    pub content: String,
    pub metadata: Value,
}

pub struct AgentCapability {
    pub domain: String,        // e.g. "code", "mail", "summarization"
    pub intents: Vec<String>,  // e.g. ["generate", "refine"]
    pub description: String,   // human-readable
}

pub struct AgentContext {
    pub config: crate::config::AppConfig,
    // hooks to model registry, tools, storage, etc.
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> &'static str;
    fn capability(&self) -> AgentCapability;
    async fn handle(&self, task: Task, ctx: AgentContext) -> anyhow::Result<AgentResult>;
}
```

```rust
// core::model

#[derive(Clone, Debug)]
pub enum EngagementMode {
    Minimum,
    Medium,
    Maximum,
}

#[derive(Clone, Debug)]
pub enum ModelRole {
    Planner,
    Coder,
    Reviewer,
    Writer,
    Summarizer,
}

#[derive(Clone, Debug)]
pub struct ModelRequest {
    pub role: ModelRole,
    pub domain: String,
    pub prompt: String,
}

#[derive(Clone, Debug)]
pub struct ModelResponse {
    pub text: String,
}

#[async_trait::async_trait]
pub trait ModelBackend: Send + Sync {
    fn id(&self) -> &'static str;
    async fn generate(&self, req: ModelRequest) -> anyhow::Result<ModelResponse>;
}
```

## 3. CLI Design (Relevant Bits)

- `bodhya run --domain code --task "..."`  
- `bodhya init`  
  - Create config file, ask for profile, optionally pre-install models.  
- `bodhya models list`  
- `bodhya models install <id>`  
- `bodhya models remove <id>`  

These commands internally call into `controller`, `model-registry`, and `core` crates.

## 4. Implementation Strategy (Inside-out)

1. Implement `core::errors`, `core::config`, `core::model`, `core::agent`.  
2. Implement `model-registry::manifest` (parse `models.yaml`) and a stub `ModelBackend`.  
3. Implement `controller::routing` and unit tests.  
4. Implement `cli::init_cmd` and `cli::models_cmd` to exercise registry and config.  
5. Implement minimal CodeAgent stub (`agent-code`) returning static responses.  
6. Progressively add planner/BDD/TDD logic and local mistral.rs integration.
