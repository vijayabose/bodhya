# Bodhya Implementation Checklist

**Last Updated**: 2025-11-15
**Status**: Phase 3 Complete - 120 tests passing, all quality gates green

---

## Phase 1: Foundation & Workspace Setup ✅

**Goal**: Create the Rust workspace structure and core abstractions

- [x] Create workspace `Cargo.toml` with all crates defined
- [x] Set up project scaffolding following the exact structure from `bodhya_code_design.md`
- [x] Create `scripts/check_all.sh` quality gate script
- [x] Implement **`core` crate** with TDD:
  - [x] `errors.rs` - Error types and Result aliases
  - [x] `config.rs` - Configuration structs (AppConfig, AgentConfig, ModelConfig)
  - [x] `model.rs` - ModelBackend trait, ModelRole enum, EngagementMode enum, ModelRequest/Response
  - [x] `agent.rs` - Agent trait, Task struct, AgentResult, AgentCapability, AgentContext
  - [x] `tool.rs` - Tool/MCP interface traits (basic)

**Deliverables**: ✅
- Full workspace structure with empty/skeleton crates
- Core trait definitions with comprehensive unit tests (46 tests)
- Quality gates passing (fmt, clippy, test)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 2: Model Registry & Manifest ✅

**Goal**: Enable model discovery and configuration parsing

- [x] Create sample `models.yaml` manifest following the spec
- [x] Implement **`model-registry` crate** with TDD:
  - [x] `manifest.rs` - Parse and validate `models.yaml`
  - [x] `registry.rs` - Lookup models by (role, domain, engagement)
  - [x] `local_mistral.rs` - Stub implementation of local ModelBackend (returns mock responses initially)
  - [x] `remote_stub.rs` - Placeholder for future remote backends
- [x] Write integration tests matching Gherkin scenarios from `bodhya_gherkin_test_cases.md`

**Deliverables**: ✅
- Model manifest parser with validation (20 tests)
- Registry that maps roles to model configs (12 tests)
- Stub ModelBackend for local and remote (8 tests)
- Integration tests for full workflow (4 tests)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 3: Controller & Routing Logic ✅

**Goal**: Implement intelligent agent selection and task orchestration

- [x] Implement **`controller` crate** with TDD:
  - [x] `routing.rs` - Capability-based agent selection (matches task description to agent capabilities)
  - [x] `engagement.rs` - Engagement mode handling (v1: enforce minimum/local-only)
  - [x] `orchestrator.rs` - Main task execution pipeline with logging
- [x] Write unit tests for:
  - [x] Code task routing to code agent
  - [x] Mail task routing to mail agent
  - [x] Unknown task handling
  - [x] Disabled agent handling

**Deliverables**: ✅
- Controller that routes tasks to agents based on capabilities (38 tests)
- Engagement mode enforcement (local-only in v1)
- Comprehensive routing tests matching Gherkin specs
- Integration tests demonstrating full task execution workflow
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 4: CLI Foundation ⬜

**Goal**: Create user-facing CLI with essential commands

- [ ] Implement **`cli` crate** with TDD:
  - [ ] `main.rs` - CLI argument parsing using `clap`
  - [ ] `init_cmd.rs` - `bodhya init` command (profile selection, config generation)
  - [ ] `models_cmd.rs` - `bodhya models list/install/remove` commands
  - [ ] `commands.rs` - `bodhya run` command stub (simple task execution)
- [ ] Create config templates for profiles: `code`, `mail`, `full`
- [ ] Implement basic file system setup (`~/.bodhya/` directory structure)

**Deliverables**:
- Working `bodhya init` command
- Working `bodhya models list` command
- Config file generation for different profiles
- CLI tests

---

## Phase 5: First Vertical Slice ⬜

**Goal**: Minimal end-to-end flow (CLI → Controller → Static Response)

- [ ] Create minimal **`agent-code` stub**:
  - [ ] Implements Agent trait
  - [ ] Returns static "Hello World" Rust code
  - [ ] No real model calls yet
- [ ] Wire CLI → Controller → CodeAgent
- [ ] Implement `bodhya run --domain code --task "hello"` end-to-end
- [ ] Add integration test for the complete flow

**Deliverables**:
- First working end-to-end slice
- Validates architecture and wiring
- Proves capability-based routing works
- Integration test matching `@slice_v1` Gherkin scenario

---

## Phase 6: CodeAgent - Planner & BDD ⬜

**Goal**: Implement CodeAgent's planning and BDD generation

- [ ] Expand **`agent-code` crate**:
  - [ ] `planner.rs` - Task decomposition using planner model
  - [ ] `bdd.rs` - Gherkin feature generation from description
- [ ] Integrate real model calls via model-registry
- [ ] Create prompt templates in `prompts/code/planner.txt`
- [ ] Write tests matching `@code_bdd` scenarios

**Deliverables**:
- CodeAgent generates Gherkin features from descriptions
- Uses local planner model (mistral.rs integration)
- Tests pass with mock/stub model responses

---

## Phase 7: CodeAgent - TDD & Implementation ⬜

**Goal**: Complete CodeAgent's test-first code generation

- [ ] Implement in **`agent-code` crate**:
  - [ ] `tdd.rs` - Test generation (RED phase)
  - [ ] `impl_gen.rs` - Code generation to make tests pass (GREEN phase)
  - [ ] `review.rs` - Code review and improvement suggestions
  - [ ] `validate.rs` - Integration with `cargo check`, `cargo test`, `cargo clippy`
- [ ] Create prompt templates for each sub-agent role
- [ ] Implement full BDD/TDD pipeline orchestration
- [ ] Write comprehensive tests matching `@code_bdd_tdd` and `@code_multimodel` scenarios

**Deliverables**:
- Full CodeAgent pipeline (Planner → BDD → TDD → Generator → Reviewer)
- Multi-model orchestration working
- Quality validation integrated
- Tests demonstrating RED-GREEN-REFACTOR flow

---

## Phase 8: MailAgent ⬜

**Goal**: Implement MailAgent for email drafting and refinement

- [ ] Implement **`agent-mail` crate**:
  - [ ] `draft.rs` - Initial email draft generation
  - [ ] `refine.rs` - Tone and clarity improvement
  - [ ] `classify.rs` - Stub for future policy/classification
- [ ] Create prompt templates for mail agent roles
- [ ] Write tests matching `@mail_draft` and `@mail_review` scenarios

**Deliverables**:
- Working MailAgent with draft and refine capabilities
- Tests demonstrating email generation and improvement
- Integration with model registry

---

## Phase 9: Tool/MCP Integration ⬜

**Goal**: Enable agents to use filesystem, git, and shell tools

- [ ] Implement **`tools-mcp` crate**:
  - [ ] `mcp_client.rs` - Generic MCP client interface
  - [ ] `fs_tool.rs` - Filesystem operations (read, write, list)
  - [ ] `shell_tool.rs` - Execute shell commands (e.g., `cargo test`)
- [ ] Update agents to use tools instead of direct system calls
- [ ] Write tests for tool integrations

**Deliverables**:
- Tool abstraction layer working
- CodeAgent can invoke `cargo` commands via tools
- MCP server integration foundation ready

---

## Phase 10: Model Download Manager ⬜

**Goal**: Implement on-demand model downloads with user consent

- [ ] Enhance **`model-registry` crate**:
  - [ ] `downloader.rs` - HTTP download with progress and verification
  - [ ] `manager.rs` - Detect missing models, prompt user, download, verify checksums
- [ ] Implement `bodhya models install <id>` fully
- [ ] Add auto-detection when running tasks with missing models
- [ ] Write tests matching `@model_download` scenarios

**Deliverables**:
- Working model download system
- Checksum verification
- User consent flow
- CLI shows size and prompts before downloading

---

## Phase 11: Storage & Metrics ⬜

**Goal**: Persist execution history and quality metrics

- [ ] Implement **`storage` crate** (optional):
  - [ ] `sqlite.rs` - SQLite connection and schema
  - [ ] `models.rs` - Data models for sessions, tasks, metrics
- [ ] Add metrics collection to controller and agents
- [ ] Add `bodhya history` command to view past tasks

**Deliverables**:
- Task history persistence
- Quality metrics storage (for evaluation)
- Simple query interface

---

## Phase 12: Evaluation Harnesses ⬜

**Goal**: Create repeatable quality evaluation for agents

- [ ] Create **`eval/code_agent/`** harness:
  - [ ] Standard test cases for code generation
  - [ ] Quality scoring (correctness, coverage, style)
  - [ ] Comparison framework
- [ ] Create **`eval/mail_agent/`** harness:
  - [ ] Standard email drafting scenarios
  - [ ] Heuristic quality checks (length, tone, clarity)
- [ ] Document how to run and interpret evaluations

**Deliverables**:
- CodeAgent evaluation achieving ≥85/100 target
- MailAgent evaluation achieving ≥4.5/5 target
- Automated scoring scripts

---

## Phase 13: API Server (Optional) ⬜

**Goal**: Add REST/WebSocket API for programmatic access

- [ ] Implement **`api-server` crate** (optional):
  - [ ] `routes.rs` - REST endpoints for task submission, status, results
  - [ ] WebSocket support for streaming responses
- [ ] Add `bodhya serve` command
- [ ] Write API integration tests

**Deliverables**:
- Working REST API
- WebSocket streaming for long-running tasks
- OpenAPI documentation

---

## Phase 14: Documentation & Polish ⬜

**Goal**: Final documentation, examples, and installer

- [ ] Create comprehensive README with examples
- [ ] Write user guide and agent development guide
- [ ] Create installer scripts for Linux, macOS, Windows
- [ ] Add example tasks and use cases
- [ ] Final quality gate verification across all KPIs

**Deliverables**:
- Complete user documentation
- Working installers for all platforms
- Example projects demonstrating CodeAgent and MailAgent
- All KPIs met (coverage ≥80%, quality scores met, etc.)

---

## Key Principles Throughout Implementation

✅ **Inside-Out**: Always implement smallest types/traits first, build upward
✅ **BDD+TDD**: Write failing tests first (from Gherkin), minimal code to pass, refactor
✅ **Quality Gates**: `scripts/check_all.sh` must pass before every commit
✅ **Thin Slices**: Validate architecture early with minimal end-to-end flows
✅ **No Remote Calls**: v1 behavior is strictly local-only (design supports future remote)
✅ **Prompts as Code**: All LLM prompts versioned in `prompts/` directory
✅ **No Document Changes**: Never modify BRD/design docs without explicit request

---

## Success Criteria

Before considering implementation complete, verify:
- [ ] All Gherkin scenarios have corresponding passing tests
- [ ] CodeAgent achieves ≥85/100 quality score
- [ ] MailAgent achieves ≥4.5/5 user rating
- [ ] Test coverage ≥80% for code agent crate
- [ ] `scripts/check_all.sh` passes with 0 warnings
- [ ] Can add new domain agent with just 1 crate + config entry
- [ ] Installer works on Linux, macOS, Windows
- [ ] All models download on-demand with proper checksums

---

## Progress Tracking

Legend: ⬜ Not Started | 🔄 In Progress | ✅ Complete

**Current Phase**: Phase 3 Complete - Controller & Routing Logic
**Next Phase**: Phase 4 - CLI Foundation
**Last Updated**: 2025-11-15
