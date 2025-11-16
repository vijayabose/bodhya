# Bodhya Tool Integration & Agentic Execution Plan

**Version**: 1.0
**Created**: 2025-11-16
**Status**: Design & Planning
**Goal**: Enable CodeAgent to perform file operations, command execution, and iterative workflows like Claude Code

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [Architecture Changes](#architecture-changes)
4. [Implementation Phases](#implementation-phases)
5. [Detailed Checklists](#detailed-checklists)
6. [Testing Strategy](#testing-strategy)
7. [Risk Mitigation](#risk-mitigation)

---

## Executive Summary

### Objective
Transform Bodhya's CodeAgent from a "text output generator" to a "full agentic executor" that can:
- Read and write files in a working directory
- Execute shell commands and observe results
- Search and edit code intelligently
- Perform git operations
- Iterate on failures until success (agentic loop)

### Scope
- **Phase 1**: Tool Integration (Foundation) - 2 weeks
- **Phase 2**: Advanced Tools (Edit, Search, Git) - 2 weeks
- **Phase 3**: Agentic Execution Loop - 1 week
- **Phase 4**: Polish & Documentation - 1 week

**Total Estimated Duration**: 6 weeks

### Key Principles
- Inside-out implementation (core types → tools → agents → CLI)
- BDD/TDD approach (tests first)
- Modular design (each tool is independent)
- Backward compatible (existing functionality preserved)
- Safe by default (sandboxing, confirmations)

---

## Current State Analysis

### ✅ What We Have
```
bodhya/
├── crates/
│   ├── tools-mcp/
│   │   ├── FilesystemTool     ✅ Read, write, list files
│   │   ├── ShellTool          ✅ Execute commands
│   │   └── ToolRegistry       ✅ Tool management
│   ├── agent-code/
│   │   ├── Planner            ✅ Task decomposition
│   │   ├── BddGenerator       ✅ Gherkin generation
│   │   ├── TddGenerator       ✅ Test generation
│   │   ├── ImplGenerator      ✅ Code generation
│   │   └── CodeReviewer       ✅ Code review
```

### ❌ Critical Gaps
1. **AgentContext** lacks tools, working directory, model registry
2. **Tools are not connected** to agents (exist but unused)
3. **No file editing** capability (only read/overwrite)
4. **No search/grep** functionality
5. **No git operations** tool
6. **No agentic iteration** (no observe-retry loop)
7. **CodeAgent outputs text** instead of executing operations

---

## Architecture Changes

### 1. Enhanced AgentContext

**Before:**
```rust
pub struct AgentContext {
    pub config: AppConfig,
    pub metadata: serde_json::Value,
}
```

**After:**
```rust
pub struct AgentContext {
    pub config: AppConfig,
    pub metadata: serde_json::Value,
    // NEW: Tool access
    pub tools: Arc<ToolRegistry>,
    // NEW: Working directory for file operations
    pub working_dir: PathBuf,
    // NEW: Model registry for agent use
    pub model_registry: Option<Arc<ModelRegistry>>,
    // NEW: Safety limits
    pub execution_limits: ExecutionLimits,
}

pub struct ExecutionLimits {
    pub max_iterations: usize,
    pub max_file_writes: usize,
    pub max_command_executions: usize,
    pub timeout_secs: u64,
}
```

### 2. CodeAgent Tool Wrapper

**New Module**: `crates/agent-code/src/tools.rs`

```rust
pub struct CodeAgentTools {
    registry: Arc<ToolRegistry>,
    working_dir: PathBuf,
    execution_stats: Arc<Mutex<ExecutionStats>>,
}

impl CodeAgentTools {
    // File operations
    pub async fn read_file(&self, path: &str) -> Result<String>;
    pub async fn write_file(&self, path: &str, content: &str) -> Result<()>;
    pub async fn edit_file(&self, path: &str, old: &str, new: &str) -> Result<()>;
    pub async fn list_files(&self, pattern: &str) -> Result<Vec<PathBuf>>;
    pub async fn file_exists(&self, path: &str) -> Result<bool>;

    // Command execution
    pub async fn run_command(&self, cmd: &str, args: &[&str]) -> Result<CommandOutput>;
    pub async fn run_cargo(&self, subcommand: &str, args: &[&str]) -> Result<CommandOutput>;

    // Search operations
    pub async fn search_code(&self, pattern: &str) -> Result<Vec<SearchMatch>>;
    pub async fn find_definition(&self, symbol: &str) -> Result<Option<Location>>;

    // Git operations
    pub async fn git_status(&self) -> Result<GitStatus>;
    pub async fn git_diff(&self) -> Result<String>;
    pub async fn git_add(&self, files: &[&str]) -> Result<()>;
    pub async fn git_commit(&self, message: &str) -> Result<()>;
}
```

### 3. New Tool Implementations

```
crates/tools-mcp/src/
├── lib.rs              (updated with new tools)
├── fs_tool.rs          (existing - unchanged)
├── shell_tool.rs       (existing - unchanged)
├── mcp_client.rs       (existing - unchanged)
├── edit_tool.rs        (NEW - smart file editing)
├── search_tool.rs      (NEW - code search/grep)
├── git_tool.rs         (NEW - git operations)
└── tool_helpers.rs     (NEW - shared utilities)
```

### 4. Agentic Execution Loop

**New Module**: `crates/agent-code/src/executor.rs`

```rust
pub struct AgenticExecutor {
    tools: Arc<CodeAgentTools>,
    registry: Arc<ModelRegistry>,
    max_iterations: usize,
}

pub struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub expected_files: Vec<PathBuf>,
    pub validation_commands: Vec<String>,
}

pub enum ExecutionStep {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    EditFile { path: String, operations: Vec<Edit> },
    RunCommand { command: String, args: Vec<String> },
    ValidateOutput { expected_pattern: String },
}

impl AgenticExecutor {
    pub async fn execute_plan(&self, plan: ExecutionPlan) -> Result<ExecutionResult>;
    pub async fn execute_with_retry(&self, plan: ExecutionPlan) -> Result<ExecutionResult>;
}
```

### 5. Updated CodeAgent Flow

**Before:**
```
Task → Planner → BDD → TDD → ImplGen → Review → Text Output
```

**After:**
```
Task → Planner → BDD → TDD → ImplGen → Execute Files → Run Tests →
  ↓ (if tests fail)
  └─→ Analyze Errors → Refine → Execute → Run Tests → (repeat)
  ↓ (if tests pass)
  Review → Git Operations (optional) → Result
```

---

## Implementation Phases

### Phase 1: Tool Integration Foundation (2 weeks)

**Goal**: Connect existing tools to agents and enable basic file/command operations

**Deliverables**:
- Enhanced AgentContext with tools
- CodeAgentTools wrapper module
- Controller integration
- CodeAgent can write files and run commands
- CLI supports working directory

**Success Criteria**:
- `bodhya run --domain code --task "generate hello world" --working-dir /tmp/test` creates actual files
- Tests pass for tool operations
- CodeAgent can execute `cargo test` and observe output

### Phase 2: Advanced Tool Capabilities (2 weeks)

**Goal**: Add intelligent file editing, code search, and git operations

**Deliverables**:
- EditTool with replace/patch/insert operations
- SearchTool with grep/find functionality
- GitTool with status/diff/commit/push operations
- Tool registry includes all new tools
- Comprehensive tests for each tool

**Success Criteria**:
- EditTool can modify specific sections of files without rewriting entire file
- SearchTool can find symbol definitions across codebase
- GitTool can safely commit changes (with confirmation)
- All tools have 80%+ test coverage

### Phase 3: Agentic Execution Loop (1 week)

**Goal**: Enable CodeAgent to iterate on failures and self-correct

**Deliverables**:
- AgenticExecutor with retry logic
- Error analysis and refinement prompts
- Execution limits and safety checks
- Metrics and logging for iterations

**Success Criteria**:
- CodeAgent can fix failing tests automatically (at least 2 iterations)
- Execution respects safety limits (max iterations, timeouts)
- Clear logging shows decision-making process
- Integration tests pass for multi-iteration scenarios

### Phase 4: Polish & Documentation (1 week)

**Goal**: Finalize, document, and prepare for production use

**Deliverables**:
- Updated documentation (system design, code design)
- Tool usage examples in prompts
- Configuration options for execution modes
- Performance optimization
- Security audit

**Success Criteria**:
- All quality gates pass (`check_all.sh`)
- Documentation is complete and accurate
- Example workflows in README
- No security vulnerabilities identified

---

## Detailed Checklists

### Phase 1: Tool Integration Foundation

#### Week 1: Core Types & Infrastructure

**1.1 Update Core Types** (`crates/core/`)
- [ ] Define `ExecutionLimits` struct in `src/agent.rs`
- [ ] Add fields to `AgentContext`:
  - [ ] `tools: Arc<ToolRegistry>`
  - [ ] `working_dir: PathBuf`
  - [ ] `model_registry: Option<Arc<ModelRegistry>>`
  - [ ] `execution_limits: ExecutionLimits`
- [ ] Update `AgentContext::new()` constructor
- [ ] Add `AgentContext::with_tools()` builder method
- [ ] Add `AgentContext::with_working_dir()` builder method
- [ ] Add `AgentContext::with_registry()` builder method
- [ ] Write unit tests for new constructors
- [ ] Update documentation comments
- [ ] Run `cargo test` - ensure existing tests still pass

**1.2 Create CodeAgentTools Wrapper** (`crates/agent-code/`)
- [ ] Create `src/tools.rs` file
- [ ] Define `CodeAgentTools` struct
- [ ] Define `CommandOutput` struct
- [ ] Define `ExecutionStats` struct
- [ ] Implement `CodeAgentTools::new()`
- [ ] Implement file operations:
  - [ ] `read_file(path)` - wrap FilesystemTool
  - [ ] `write_file(path, content)` - wrap FilesystemTool
  - [ ] `list_files(pattern)` - wrap FilesystemTool
  - [ ] `file_exists(path)` - wrap FilesystemTool
- [ ] Implement command execution:
  - [ ] `run_command(cmd, args)` - wrap ShellTool
  - [ ] `run_cargo(subcommand, args)` - convenience wrapper
- [ ] Add path resolution logic (relative to working_dir)
- [ ] Add execution statistics tracking
- [ ] Write comprehensive unit tests (use TempDir)
- [ ] Add integration tests
- [ ] Update `src/lib.rs` to export new module
- [ ] Run `cargo test --package bodhya-agent-code`

**1.3 Update Controller** (`crates/controller/`)
- [ ] Modify `src/controller.rs`:
  - [ ] Add `tools: Arc<ToolRegistry>` field to `Controller`
  - [ ] Update `Controller::new()` to accept tools
  - [ ] Update `Controller::with_defaults()` to create default tools
- [ ] Modify `src/orchestrator.rs`:
  - [ ] Pass `working_dir` when creating `AgentContext`
  - [ ] Pass `tools` to `AgentContext`
  - [ ] Pass `model_registry` to `AgentContext`
- [ ] Add `ExecutionLimits::default()` with safe defaults
- [ ] Write tests for controller with tools
- [ ] Run `cargo test --package bodhya-controller`

#### Week 2: Agent Integration & CLI

**1.4 Update CodeAgent** (`crates/agent-code/`)
- [ ] Modify `src/lib.rs`:
  - [ ] Add `use tools::CodeAgentTools` import
  - [ ] Update `handle()` to accept `ctx: AgentContext` (not ignore it)
  - [ ] Extract `tools` from context
  - [ ] Extract `working_dir` from context
- [ ] Modify `generate_with_tdd()`:
  - [ ] Add `ctx: &AgentContext` parameter
  - [ ] Create `CodeAgentTools` instance from context
  - [ ] After generating test code, write it to file:
    - [ ] Determine test file path (e.g., `tests/generated_test.rs`)
    - [ ] Call `tools.write_file()`
  - [ ] After generating impl code, write it to file:
    - [ ] Determine impl file path (e.g., `src/lib.rs` or `src/generated.rs`)
    - [ ] Call `tools.write_file()`
  - [ ] Execute `cargo test` via `tools.run_cargo("test", &[])`
  - [ ] Parse test output to check success/failure
  - [ ] Update output to include execution results
- [ ] Add error handling for file/command operations
- [ ] Update fallback behavior (graceful degradation)
- [ ] Write tests for new execution behavior:
  - [ ] Test file writing
  - [ ] Test command execution
  - [ ] Test error handling
- [ ] Run `cargo test --package bodhya-agent-code`

**1.5 Update CLI** (`crates/cli/`)
- [ ] Modify `src/run_cmd.rs`:
  - [ ] Add `--working-dir <PATH>` flag
  - [ ] Default to current directory (`std::env::current_dir()`)
  - [ ] Validate working directory exists
  - [ ] Create `ToolRegistry::with_defaults()`
  - [ ] Pass tools to `Controller::new()`
  - [ ] Pass working_dir when executing tasks
- [ ] Add `--execution-mode <MODE>` flag:
  - [ ] Options: `generate-only`, `execute`, `execute-with-retry`
  - [ ] Default: `execute`
- [ ] Update help text and examples
- [ ] Write integration tests:
  - [ ] Test CLI with working directory
  - [ ] Test CLI with execution modes
  - [ ] Test end-to-end file generation
- [ ] Run `cargo test --package bodhya-cli`

**1.6 Integration Testing**
- [ ] Create `tests/integration/tool_integration_test.rs`
- [ ] Test scenario: Generate hello world and verify files created
- [ ] Test scenario: Run cargo test and observe output
- [ ] Test scenario: Handle missing working directory gracefully
- [ ] Test scenario: Respect execution limits
- [ ] Run full test suite: `cargo test --all`
- [ ] Run quality gates: `./scripts/check_all.sh`

---

### Phase 2: Advanced Tool Capabilities

#### Week 3: Edit & Search Tools

**2.1 Implement EditTool** (`crates/tools-mcp/`)
- [ ] Create `src/edit_tool.rs` file
- [ ] Define `EditTool` struct
- [ ] Define `EditOperation` enum:
  - [ ] `Replace { old: String, new: String }`
  - [ ] `Patch { diff: String }`
  - [ ] `InsertAtLine { line_num: usize, content: String }`
  - [ ] `DeleteLines { start: usize, end: usize }`
- [ ] Implement `Tool` trait for `EditTool`
- [ ] Implement operations:
  - [ ] `replace` - find exact match and replace
  - [ ] `patch` - apply unified diff
  - [ ] `insert_at_line` - insert at line number
  - [ ] `delete_lines` - remove lines
- [ ] Add validation:
  - [ ] Ensure old text exists for replace
  - [ ] Validate line numbers
  - [ ] Check file exists before editing
- [ ] Add dry-run mode (preview changes)
- [ ] Write comprehensive tests:
  - [ ] Test replace operation
  - [ ] Test multi-line replace
  - [ ] Test patch operation
  - [ ] Test insert operation
  - [ ] Test delete operation
  - [ ] Test error cases
- [ ] Update `src/lib.rs` to export `EditTool`
- [ ] Register in `ToolRegistry::with_defaults()`
- [ ] Run tests: `cargo test --package bodhya-tools-mcp`

**2.2 Implement SearchTool** (`crates/tools-mcp/`)
- [ ] Create `src/search_tool.rs` file
- [ ] Define `SearchTool` struct
- [ ] Define `SearchMatch` struct:
  - [ ] `file: PathBuf`
  - [ ] `line_num: usize`
  - [ ] `line_content: String`
  - [ ] `match_start: usize`
  - [ ] `match_end: usize`
- [ ] Define `Location` struct (for definitions)
- [ ] Implement `Tool` trait for `SearchTool`
- [ ] Implement operations:
  - [ ] `grep` - search for pattern in files
  - [ ] `grep_recursive` - search in directory tree
  - [ ] `find_definition` - find function/struct definitions
  - [ ] `find_references` - find symbol usage
- [ ] Add support for:
  - [ ] Regex patterns
  - [ ] Case-insensitive search
  - [ ] File type filtering (e.g., only .rs files)
  - [ ] Context lines (before/after)
- [ ] Write comprehensive tests:
  - [ ] Test basic grep
  - [ ] Test recursive search
  - [ ] Test regex patterns
  - [ ] Test find definition
  - [ ] Test case sensitivity
- [ ] Update `src/lib.rs` to export `SearchTool`
- [ ] Register in `ToolRegistry::with_defaults()`
- [ ] Run tests: `cargo test --package bodhya-tools-mcp`

**2.3 Add Tool Methods to CodeAgentTools** (`crates/agent-code/`)
- [ ] Update `src/tools.rs`:
  - [ ] Add `edit_file(path, old, new)` method
  - [ ] Add `patch_file(path, diff)` method
  - [ ] Add `search_code(pattern)` method
  - [ ] Add `find_definition(symbol)` method
- [ ] Write tests for new methods
- [ ] Run tests: `cargo test --package bodhya-agent-code`

#### Week 4: Git Tool & Integration

**2.4 Implement GitTool** (`crates/tools-mcp/`)
- [ ] Create `src/git_tool.rs` file
- [ ] Define `GitTool` struct
- [ ] Define `GitStatus` struct:
  - [ ] `branch: String`
  - [ ] `staged_files: Vec<PathBuf>`
  - [ ] `unstaged_files: Vec<PathBuf>`
  - [ ] `untracked_files: Vec<PathBuf>`
  - [ ] `is_clean: bool`
- [ ] Implement `Tool` trait for `GitTool`
- [ ] Implement operations:
  - [ ] `status` - get repository status
  - [ ] `diff` - show unstaged changes
  - [ ] `diff_staged` - show staged changes
  - [ ] `add` - stage files
  - [ ] `commit` - create commit
  - [ ] `push` - push to remote (with safety checks)
  - [ ] `pull` - pull from remote
  - [ ] `branch` - list/create branches
  - [ ] `log` - show commit history
- [ ] Add safety features:
  - [ ] Require confirmation for destructive operations
  - [ ] Check if in git repository
  - [ ] Validate branch names
  - [ ] Prevent force push without explicit flag
- [ ] Write comprehensive tests:
  - [ ] Test status parsing
  - [ ] Test add/commit flow
  - [ ] Test diff generation
  - [ ] Test safety checks
  - [ ] Use temp git repo for testing
- [ ] Update `src/lib.rs` to export `GitTool`
- [ ] Register in `ToolRegistry::with_defaults()`
- [ ] Run tests: `cargo test --package bodhya-tools-mcp`

**2.5 Add Git Methods to CodeAgentTools** (`crates/agent-code/`)
- [ ] Update `src/tools.rs`:
  - [ ] Add `git_status()` method
  - [ ] Add `git_diff()` method
  - [ ] Add `git_add(files)` method
  - [ ] Add `git_commit(message)` method
  - [ ] Add `git_push()` method (with confirmation)
- [ ] Write tests for git methods
- [ ] Run tests: `cargo test --package bodhya-agent-code`

**2.6 Integration Testing**
- [ ] Create `tests/integration/advanced_tools_test.rs`
- [ ] Test scenario: Edit file and verify changes
- [ ] Test scenario: Search code and find matches
- [ ] Test scenario: Git add/commit workflow
- [ ] Test all tools together in realistic workflow
- [ ] Run full test suite: `cargo test --all`
- [ ] Run quality gates: `./scripts/check_all.sh`

---

### Phase 3: Agentic Execution Loop

#### Week 5: Executor Implementation

**3.1 Create Agentic Executor** (`crates/agent-code/`)
- [ ] Create `src/executor.rs` file
- [ ] Define `AgenticExecutor` struct
- [ ] Define `ExecutionPlan` struct
- [ ] Define `ExecutionStep` enum
- [ ] Define `ExecutionResult` struct:
  - [ ] `success: bool`
  - [ ] `iterations: usize`
  - [ ] `files_modified: Vec<PathBuf>`
  - [ ] `commands_executed: Vec<String>`
  - [ ] `final_output: String`
  - [ ] `error_history: Vec<String>`
- [ ] Implement `AgenticExecutor::new()`
- [ ] Implement `execute_plan()`:
  - [ ] Execute each step sequentially
  - [ ] Track execution state
  - [ ] Handle errors gracefully
- [ ] Implement `execute_with_retry()`:
  - [ ] Execute plan
  - [ ] If fails, analyze errors
  - [ ] Generate refinement plan
  - [ ] Retry with refined plan
  - [ ] Repeat up to max_iterations
- [ ] Add error analysis logic:
  - [ ] Parse compiler errors
  - [ ] Parse test failures
  - [ ] Extract actionable insights
- [ ] Write tests:
  - [ ] Test successful execution
  - [ ] Test retry on failure
  - [ ] Test max iterations limit
  - [ ] Test error analysis
- [ ] Update `src/lib.rs` to export executor
- [ ] Run tests: `cargo test --package bodhya-agent-code`

**3.2 Integrate Executor into CodeAgent** (`crates/agent-code/`)
- [ ] Modify `src/lib.rs`:
  - [ ] Add `executor: Option<AgenticExecutor>` field
  - [ ] Update `with_registry()` to create executor
  - [ ] Add `generate_with_execution()` method:
    - [ ] Generate plan
    - [ ] Generate BDD/TDD/Impl
    - [ ] Create ExecutionPlan
    - [ ] Execute via AgenticExecutor
    - [ ] Return results with iteration history
- [ ] Update `handle()` to use executor when available
- [ ] Add configuration for execution mode:
  - [ ] `generate_only` - old behavior
  - [ ] `execute_once` - Phase 1 behavior
  - [ ] `execute_with_retry` - Phase 3 behavior (new)
- [ ] Write tests:
  - [ ] Test generate_only mode
  - [ ] Test execute_once mode
  - [ ] Test execute_with_retry mode
  - [ ] Test successful multi-iteration scenario
- [ ] Run tests: `cargo test --package bodhya-agent-code`

**3.3 Update Prompts** (`prompts/code/`)
- [ ] Create `coder_with_tools.txt`:
  - [ ] List available tools
  - [ ] Show tool usage examples
  - [ ] Add guidelines for when to use each tool
  - [ ] Include error recovery strategies
- [ ] Create `error_analyzer.txt`:
  - [ ] Template for analyzing compilation errors
  - [ ] Template for analyzing test failures
  - [ ] Guidelines for generating fixes
- [ ] Update `reviewer.txt`:
  - [ ] Add section on tool usage validation
  - [ ] Check if files were created correctly
  - [ ] Verify commands executed successfully
- [ ] Test prompts with sample inputs

**3.4 Add Configuration** (`crates/core/`)
- [ ] Update `src/config.rs`:
  - [ ] Add `ExecutionMode` enum to config
  - [ ] Add `max_iterations` setting
  - [ ] Add `max_file_writes` setting
  - [ ] Add `max_command_executions` setting
  - [ ] Add `enable_git_operations` flag
  - [ ] Add defaults
- [ ] Update config YAML templates
- [ ] Write tests for config parsing
- [ ] Run tests: `cargo test --package bodhya-core`

**3.5 Update CLI** (`crates/cli/`)
- [ ] Modify `src/run_cmd.rs`:
  - [ ] Update `--execution-mode` to support `execute-with-retry`
  - [ ] Add `--max-iterations <N>` flag
  - [ ] Add `--enable-git` flag
  - [ ] Load execution config from file if provided
  - [ ] Pass config to controller
- [ ] Update help text with new options
- [ ] Write tests for new CLI options
- [ ] Run tests: `cargo test --package bodhya-cli`

**3.6 Integration Testing**
- [ ] Create `tests/integration/agentic_execution_test.rs`
- [ ] Test scenario: Generate code with syntax error → auto-fix → success
- [ ] Test scenario: Generate code with test failure → refine → pass
- [ ] Test scenario: Respect max iterations limit
- [ ] Test scenario: Complete workflow with git commit
- [ ] Test edge cases and error conditions
- [ ] Run full test suite: `cargo test --all`
- [ ] Run quality gates: `./scripts/check_all.sh`

---

### Phase 4: Polish & Documentation

#### Week 6: Finalization

**4.1 Update Design Documentation**
- [ ] Update `documents/bodhya_system_design.md`:
  - [ ] Add section on Tool Layer architecture
  - [ ] Update AgentContext description
  - [ ] Add Agentic Execution Loop flow diagram
  - [ ] Update data flow diagrams
  - [ ] Document safety mechanisms
- [ ] Update `documents/bodhya_code_design.md`:
  - [ ] Add new modules to workspace layout
  - [ ] Document CodeAgentTools API
  - [ ] Document ExecutionPlan structure
  - [ ] Add code examples
- [ ] Create `documents/bodhya_tool_usage_guide.md`:
  - [ ] Overview of all tools
  - [ ] Usage examples for each tool
  - [ ] Best practices
  - [ ] Troubleshooting guide
- [ ] Update Gherkin scenarios if needed

**4.2 Add Examples & Tutorials**
- [ ] Create `examples/hello_world_agent/`:
  - [ ] Sample task configuration
  - [ ] Expected output
  - [ ] README with walkthrough
- [ ] Create `examples/test_driven_agent/`:
  - [ ] TDD workflow example
  - [ ] Show iteration process
  - [ ] README with explanation
- [ ] Create `examples/git_workflow_agent/`:
  - [ ] End-to-end workflow with git
  - [ ] Show commit generation
  - [ ] README with explanation
- [ ] Update main `README.md`:
  - [ ] Add "Features" section highlighting tool capabilities
  - [ ] Add "Quick Start" with tool examples
  - [ ] Add "Architecture" section with diagrams
  - [ ] Update CLI usage examples

**4.3 Performance Optimization**
- [ ] Profile tool operations:
  - [ ] Identify slow operations
  - [ ] Add caching where appropriate
  - [ ] Optimize file I/O
- [ ] Optimize search operations:
  - [ ] Consider using ripgrep library directly
  - [ ] Add indexing for large codebases
- [ ] Optimize git operations:
  - [ ] Cache git status between calls
  - [ ] Batch operations where possible
- [ ] Add benchmarks for critical paths
- [ ] Document performance characteristics

**4.4 Security Audit**
- [ ] Review file operation sandboxing:
  - [ ] Ensure paths don't escape working_dir
  - [ ] Test with malicious path inputs (../, symlinks)
  - [ ] Add comprehensive path validation tests
- [ ] Review command execution safety:
  - [ ] Validate command injection protection
  - [ ] Test with malicious inputs
  - [ ] Document allowed/disallowed commands
- [ ] Review git operation safety:
  - [ ] Ensure no force push without confirmation
  - [ ] Protect against accidental data loss
  - [ ] Test rollback mechanisms
- [ ] Add security section to documentation
- [ ] Run security audit tools (cargo-audit, cargo-deny)

**4.5 Final Testing & Quality Gates**
- [ ] Run full test suite: `cargo test --all`
- [ ] Run with verbose output: `cargo test --all -- --nocapture`
- [ ] Check code coverage:
  - [ ] Install tarpaulin: `cargo install cargo-tarpaulin`
  - [ ] Run: `cargo tarpaulin --all`
  - [ ] Ensure CodeAgent coverage ≥ 80%
  - [ ] Ensure new tools coverage ≥ 80%
- [ ] Run quality gates: `./scripts/check_all.sh`
- [ ] Test on different platforms (Linux, macOS)
- [ ] Create release checklist
- [ ] Tag release: `v1.1.0-tool-integration`

**4.6 Documentation Review**
- [ ] Review all updated documents for accuracy
- [ ] Check code examples compile and run
- [ ] Verify all links work
- [ ] Ensure consistent terminology
- [ ] Add table of contents where needed
- [ ] Spell check and grammar check
- [ ] Get peer review on documentation

---

## Testing Strategy

### Unit Testing
Each module must have comprehensive unit tests:

**Coverage Targets:**
- Core types: 90%+
- Tools: 85%+
- CodeAgent: 80%+
- Executor: 85%+

**Test Categories:**
1. **Happy Path**: Normal operations succeed
2. **Error Handling**: Invalid inputs handled gracefully
3. **Edge Cases**: Boundary conditions (empty files, large files, etc.)
4. **Security**: Path traversal, command injection, etc.

### Integration Testing

**Test Scenarios:**
1. **End-to-End Workflows**:
   - Generate hello world → verify files created
   - Generate with tests → verify tests run
   - Multi-iteration scenario → verify convergence

2. **Tool Combinations**:
   - Read → Edit → Write workflow
   - Search → Edit → Test workflow
   - Generate → Test → Git workflow

3. **Error Recovery**:
   - Missing working directory
   - Command execution failure
   - Test failure → retry → success

### BDD Testing

Create Gherkin scenarios for major features:

```gherkin
Feature: Agentic Code Generation with Tool Execution

  Scenario: Generate hello world and create files
    Given a working directory "/tmp/test-project"
    When I run "bodhya run --domain code --task 'generate hello world'"
    Then files should be created:
      | path           | exists |
      | src/lib.rs     | true   |
      | tests/test.rs  | true   |
    And "cargo test" should pass

  Scenario: Auto-fix failing tests
    Given a task that initially generates incorrect code
    When CodeAgent executes with retry enabled
    Then it should iterate up to 3 times
    And tests should eventually pass
    And execution result should include iteration history
```

### Performance Testing

**Benchmarks:**
- File read/write operations
- Command execution overhead
- Search performance on large codebases
- End-to-end task completion time

**Targets:**
- Single file operation: < 100ms
- Command execution: < 2s
- Code search: < 1s for 10K files
- Full generation cycle: < 30s

---

## Risk Mitigation

### Technical Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Tool operations fail silently | High | Medium | Comprehensive error handling, logging |
| Path traversal vulnerability | High | Low | Strict path validation, sandboxing |
| Command injection | Critical | Low | Allowlist commands, input sanitization |
| Infinite iteration loop | Medium | Medium | Max iteration limits, timeout enforcement |
| Large file performance issues | Medium | Medium | File size limits, streaming operations |
| Git conflicts | Medium | Medium | Pre-check status, require clean working tree |

### Implementation Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Breaking existing functionality | High | Medium | Extensive regression testing, backward compatibility |
| Scope creep | Medium | High | Strict phase boundaries, clear acceptance criteria |
| Integration complexity | Medium | Medium | Incremental integration, continuous testing |
| Performance degradation | Medium | Low | Profiling, benchmarks, optimization phase |

### Safety Mechanisms

1. **Execution Limits**:
   ```rust
   pub struct ExecutionLimits {
       pub max_iterations: usize,        // Default: 3
       pub max_file_writes: usize,       // Default: 20
       pub max_command_executions: usize, // Default: 10
       pub timeout_secs: u64,            // Default: 300
   }
   ```

2. **Sandboxing**:
   - All file operations relative to `working_dir`
   - Path validation prevents `../` escape
   - Canonical path checking

3. **Confirmation Gates**:
   - Git push requires explicit confirmation
   - Destructive operations log warnings
   - Dry-run mode available for testing

4. **Rollback Support**:
   - Track all file modifications
   - Enable undo/rollback on failure
   - Git integration for version control

---

## Success Metrics

### Functional Metrics
- [ ] CodeAgent can create files in working directory
- [ ] CodeAgent can execute cargo commands
- [ ] CodeAgent can iterate on test failures
- [ ] All tools integrated and functional
- [ ] End-to-end workflows complete successfully

### Quality Metrics
- [ ] Test coverage ≥ 80% for CodeAgent
- [ ] Test coverage ≥ 85% for tools
- [ ] All quality gates pass
- [ ] Zero security vulnerabilities
- [ ] Documentation complete

### Performance Metrics
- [ ] File operations < 100ms
- [ ] Command execution < 2s
- [ ] Full generation cycle < 30s
- [ ] Search operations < 1s (10K files)

### User Experience Metrics
- [ ] CLI intuitive and well-documented
- [ ] Error messages clear and actionable
- [ ] Examples demonstrate capabilities
- [ ] Tool usage feels natural

---

## Next Steps

1. **Review this plan** with team/stakeholders
2. **Set up project tracking** (GitHub project board, JIRA, etc.)
3. **Assign ownership** for each phase
4. **Create feature branch**: `feature/tool-integration`
5. **Begin Phase 1, Week 1** implementation
6. **Schedule weekly reviews** to track progress

---

## Appendix

### File Modification Summary

**New Files (25 total):**
```
crates/agent-code/src/tools.rs
crates/agent-code/src/executor.rs
crates/tools-mcp/src/edit_tool.rs
crates/tools-mcp/src/search_tool.rs
crates/tools-mcp/src/git_tool.rs
crates/tools-mcp/src/tool_helpers.rs
prompts/code/coder_with_tools.txt
prompts/code/error_analyzer.txt
documents/bodhya_tool_integration_plan.md (this file)
documents/bodhya_tool_usage_guide.md
examples/hello_world_agent/*
examples/test_driven_agent/*
examples/git_workflow_agent/*
tests/integration/tool_integration_test.rs
tests/integration/advanced_tools_test.rs
tests/integration/agentic_execution_test.rs
+ BDD scenarios, benchmarks, etc.
```

**Modified Files (15 total):**
```
crates/core/src/agent.rs
crates/core/src/config.rs
crates/controller/src/controller.rs
crates/controller/src/orchestrator.rs
crates/agent-code/src/lib.rs
crates/tools-mcp/src/lib.rs
crates/cli/src/run_cmd.rs
documents/bodhya_system_design.md
documents/bodhya_code_design.md
prompts/code/reviewer.txt
README.md
Cargo.toml (workspace members)
+ Test files, config templates, etc.
```

### Dependencies to Add

```toml
# Cargo.toml additions

[workspace.dependencies]
# For SearchTool
regex = "1.10"
ignore = "0.4"  # gitignore-aware directory traversal

# For GitTool
git2 = "0.18"  # libgit2 bindings

# For EditTool
similar = "2.4"  # diff/patch algorithms

# For testing
tempfile = "3.10"  # already present
```

---

**End of Tool Integration Plan**
