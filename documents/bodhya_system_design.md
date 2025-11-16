# BODHYA – System Design

## 1. High-Level Architecture

Bodhya is a local-first, modular multi-agent platform built in Rust. It consists of:

- **Central Controller Agent (Bodhya.Controller)**
  - Receives tasks (via CLI or API).
  - Classifies the domain (code, mail, future domains).
  - Selects the appropriate domain agent based on configuration, routing strategy, and agent capability metadata.
  - Manages engagement mode for local vs remote models.
  - Provides tools and working directory to agents via AgentContext.
  - Aggregates logs and metrics.

- **Domain Agents (Pluggable Modules)**
  - **Bodhya.CodeAgent**
    - Planner sub-agent (task decomposition, BDD).
    - BDD engine (Gherkin generation).
    - TDD engine (test-first code generation).
    - Implementation generator.
    - Reviewer/refiner.
    - **Agentic Executor** (v1.1+):
      - Executes generated code in working directory.
      - Runs tests and observes results.
      - Iterates on failures with error analysis.
      - Performs file operations, command execution, and git operations.
  - **Bodhya.MailAgent**
    - Draft generator.
    - Tone & clarity refiner.
    - (Future) classifier/policy checker.
  - Future domain agents (Summarization, Document Q&A, Planning, etc.) can be added without modifying core logic.

- **Model Registry & Inference Layer**
  - Local inference backends via **mistral.rs** (Rust-native).
  - Model roles: `planner`, `coder`, `reviewer`, `writer`, `summarizer`, etc.
  - Engagement modes:
    - `Minimum` – local only (v1).
    - `Medium` – local primary, remote fallback (future).
    - `Maximum` – remote heavily used (future).
  - Model manifest file (`models.yaml`) describing available models by role:
    - Logical role → model config (name, size, quantization, source URL, checksum).
  - Model manager responsible for:
    - Checking which models are installed.
    - Downloading missing models on demand.
    - Storing them under `~/.bodhya/models/`.
    - Verifying checksums and health.

- **Tool / MCP Layer** (v1.1 Enhanced)
  - A uniform interface for tools (e.g., filesystem, git, shell, HTTP).
  - **ToolRegistry**: Central registry for all tools, providing discovery and routing.
  - **Core Tools**:
    - **FilesystemTool**: Read, write, list files and directories.
    - **ShellTool**: Execute shell commands with timeout and working directory support.
    - **EditTool** (v1.1): Smart file editing with replace, patch, insert operations.
    - **SearchTool** (v1.1): Code search, grep, find definitions and references.
    - **GitTool** (v1.1): Git operations (status, diff, add, commit, push with safety checks).
  - **CodeAgentTools**: High-level wrapper providing agent-friendly tool interface.
  - Support for MCP servers as first-class integration points.
  - **Sandboxing**: All file operations restricted to working directory for security.
  - Domain agents access tools via AgentContext, not directly.

- **Validation & Metrics**
  - CodeAgent:
    - Automatic execution: `cargo check`, `cargo test`, `cargo clippy`.
    - Coverage tools, `cargo-audit`, `cargo-deny`.
    - Iterative refinement based on test/build failures.
  - MailAgent:
    - Style and length heuristics (initially).
    - Future: sentiment or tone classifiers, still local-first.
  - Metrics persisted via a storage layer (e.g., SQLite) for performance and quality evaluation.
  - **Execution tracking**: Files modified, commands run, iterations performed.

---

## 2. Agent Capability Contract

Each agent exposes both behavior and metadata so the controller can route tasks intelligently.

### 2.1 Capability Struct (Conceptual)

- **Domain:** high-level area, e.g., `"code"`, `"mail"`, `"summarization"`.  
- **Intents:** actions supported, e.g., `"generate"`, `"refine"`, `"summarize"`, `"classify"`.  
- **Description:** human-readable explanation, used for routing and UI.  

The controller uses capability metadata to:

- Match task descriptions to agents.  
- Avoid hardcoding specific agent IDs in routing logic.  
- Allow new agents to register themselves via configuration.

---

## 3. Installation & Model Management Design

### 3.1 Installer

- Distributes a single Bodhya binary (plus optional helper scripts) for each OS.  
- When run, installer:  
  - Places `bodhya` on PATH.  
  - Creates `~/.bodhya/` with:  
    - `config/` (default config templates).  
    - `models/` (empty or minimal starter).  
    - `logs/` and `cache/` (optional).  
  - Copies `scripts/check_all.sh` into the repo or an appropriate tools folder.

### 3.2 Initialization (`bodhya init`)

- Prompts the user for:  
  - Profile (code / mail / full).  
  - Optional eager model downloads.  
- Generates a config file (e.g., `~/.bodhya/config/default.yaml`) with:  
  - Active agents per profile.  
  - Model role → model ID mappings.  
  - Engagement policy (default: minimum).  

### 3.3 Model Manifest and Manager

- `models.yaml` example:

```yaml
models:
  code_planner:
    role: "Planner"
    domain: "code"
    display_name: "Code Planner (Qwen)"
    source_url: "https://example.com/models/code-planner.bin"
    size_gb: 4
    checksum: "sha256:..."
  code_coder:
    role: "Coder"
    domain: "code"
    display_name: "Code Generator (DeepSeek)"
    source_url: "https://example.com/models/code-coder.bin"
    size_gb: 8
    checksum: "sha256:..."
  mail_writer:
    role: "Writer"
    domain: "mail"
    display_name: "Mail Writer (Mistral)"
    source_url: "https://example.com/models/mail-writer.bin"
    size_gb: 3
    checksum: "sha256:..."
```

- Model manager responsibilities:  
  - `list` – report installed models and their roles.  
  - `install <id>` – download and validate model, record metadata.  
  - `remove <id>` – safely delete a model.  
  - Provide a Rust API used by domain agents to obtain an appropriate `ModelBackend` instance.

CLI integration: `bodhya models list/install/remove` uses this manager.

---

## 4. Data Flows

### 4.1 Task Handling Flow (v1.0)

1. User submits a task (via CLI or API).
2. Controller:
   - Normalizes input → `Task` struct.
   - Uses capability-aware routing to select an agent.
   - Creates `AgentContext` with tools, working directory, and model registry.
3. Domain Agent:
   - Orchestrates sub-agents and model calls.
   - Uses tools/MCP where needed (e.g., file IO, git).
4. Results:
   - Returned to user.
   - Logged to metrics/storage.

### 4.2 Agentic Execution Flow (v1.1+)

**CodeAgent with Execution Mode:**

```
1. User submits task
   ↓
2. Controller creates AgentContext
   - tools: ToolRegistry
   - working_dir: PathBuf
   - model_registry: Arc<ModelRegistry>
   - execution_limits: ExecutionLimits
   ↓
3. CodeAgent receives Task + AgentContext
   ↓
4. Planning Phase
   - Planner generates CodePlan
   ↓
5. Specification Phase
   - BDD Generator creates GherkinFeature
   - TDD Generator creates test code
   ↓
6. Implementation Phase
   - ImplGenerator creates implementation code
   ↓
7. Execution Phase (NEW in v1.1)
   - CodeAgentTools writes test file to working_dir
   - CodeAgentTools writes impl file to working_dir
   - Execute: cargo test
   - Observe: Parse test output
   ↓
8. Decision Point: Tests Pass?
   ├─ YES → Continue to Review
   └─ NO → Iteration Loop (up to max_iterations)
       ↓
       Error Analysis
       - Parse compiler/test errors
       - Extract actionable insights
       ↓
       Refinement
       - Generate fixes based on errors
       - Update implementation code
       ↓
       Re-execute (go back to step 7)
   ↓
9. Review Phase
   - CodeReviewer analyzes final code
   ↓
10. Optional: Git Operations
    - git add generated files
    - git commit with auto-generated message
    ↓
11. Return AgentResult
    - success: bool
    - content: generated code + execution log
    - metadata: {iterations, files_modified, tests_passed}
```

### 4.3 Tool Invocation Flow

```
Agent → CodeAgentTools → ToolRegistry → Specific Tool → Result
  ↓                          ↓                ↓
  AgentContext          ToolRequest      FilesystemTool
                                         ShellTool
                                         EditTool
                                         SearchTool
                                         GitTool
```

**Example: Write File Operation**

```rust
// Agent code
let tools = CodeAgentTools::new(ctx.tools, ctx.working_dir);
tools.write_file("src/lib.rs", &impl_code).await?;

// CodeAgentTools implementation
async fn write_file(&self, path: &str, content: &str) -> Result<()> {
    let request = ToolRequest::new(
        "filesystem",
        "write",
        json!({ "path": path, "content": content })
    );
    self.registry.execute(request).await?;
    self.stats.lock().await.files_written += 1;
    Ok(())
}
```

### 4.4 Model Selection Flow

1. Domain agent requests a model for a role & domain & engagement mode.
2. Model registry:
   - Reads config + `models.yaml` mapping to a `ModelBackend` instance.
   - In v1, resolves to local mistral.rs engines only.
3. Domain agent invokes the returned backend to generate/critique content.

### 4.5 Vertical Slice Flow

For the first vertical slice:

1. CLI creates a `Task` with a simple description.
2. Controller routes to CodeAgent based on domain hints/intents.
3. CodeAgent returns a static placeholder result (no real model calls yet).
4. End-to-end integration is validated early.
5. Next slices progressively:
   - Model registry stub → real local backend.
   - Minimal planner → BDD → TDD → full codegen pipeline.
   - Tool integration → execution mode → agentic iteration.

---

## 5. Evaluation & Quality Harness

- Standard script `scripts/check_all.sh` runs:  
  - `cargo fmt --check`  
  - `cargo clippy --all-targets -- -D warnings`  
  - `cargo test --all`  
  - Optional: `cargo audit`, `cargo deny`  

- Evaluation harness (`eval/`) for:  
  - CodeAgent: standard code-generation tasks, correctness + coverage.  
  - MailAgent: standard drafting tasks, length and politeness/clarity heuristics.

---

## 6. Non-Functional Requirements

- **Performance:**  
  - Minimize latency for typical tasks.  
  - Utilize streaming where appropriate.  

- **Security & Privacy:**
  - Default to local-only inference.
  - Remote connectors are explicitly configured; no accidental remote calls.
  - **Tool Sandboxing** (v1.1+):
    - All file operations restricted to `working_dir`.
    - Path validation prevents directory traversal (`../` attacks).
    - Canonical path checking before operations.
    - No absolute paths allowed outside working directory.
  - **Command Execution Safety**:
    - Command injection prevention via argument arrays.
    - Timeout enforcement for all commands.
    - Working directory isolation.
  - **Git Operation Safety**:
    - Confirmations required for destructive operations.
    - No force push without explicit user flag.
    - Repository validation before operations.

- **Reliability:**
  - Clear error boundaries between agents, controller, tools, and model manager.
  - Fail fast with descriptive diagnostics.
  - `check_all.sh` must pass before merges to main or tagged releases.
  - **Execution Limits** (v1.1+):
    - Maximum iterations (default: 3) to prevent infinite loops.
    - Maximum file writes (default: 20) to prevent resource exhaustion.
    - Maximum command executions (default: 10) to limit impact.
    - Global timeout (default: 300s) for task completion.
  - **Graceful Degradation**:
    - If tools unavailable, fall back to text-only generation.
    - If execution fails, return partial results with error details.

- **Maintainability:**
  - Agents and models defined by contracts and configuration.
  - New models/agents added without core rewrites.
  - Prompts stored and versioned as files, treated as part of the codebase.
  - **Tool Modularity** (v1.1+):
    - Each tool is independent, can be added/removed via registry.
    - Tool interface uniform across all implementations.
    - Tools testable in isolation.
