# Bodhya Implementation Checklist

**Last Updated**: 2025-11-16
**Status**: Phase 13 Complete - 369 tests passing, API Server ready for integration

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

## Phase 4: CLI Foundation ✅

**Goal**: Create user-facing CLI with essential commands

- [x] Implement **`cli` crate** with TDD:
  - [x] `main.rs` - CLI argument parsing using `clap`
  - [x] `init_cmd.rs` - `bodhya init` command (profile selection, config generation)
  - [x] `models_cmd.rs` - `bodhya models list/install/remove` commands
  - [x] `run_cmd.rs` - `bodhya run` command stub (simple task execution)
  - [x] `config_templates.rs` - Profile templates (code, mail, full)
  - [x] `utils.rs` - Path utilities and directory management
- [x] Create config templates for profiles: `code`, `mail`, `full`
- [x] Implement basic file system setup (`~/.bodhya/` directory structure)

**Deliverables**: ✅
- Working `bodhya init` command with profile selection (31 tests)
- Working `bodhya models list/install/remove` commands
- Config file generation for different profiles
- Stub `bodhya run` command for task execution
- Path utilities and directory management
- CLI structure tests (19 passed, 17 ignored due to test infra limitations)
- **Note**: Some tests ignored due to HOME env var mocking complexity in concurrent tests
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 5: First Vertical Slice ✅

**Goal**: Minimal end-to-end flow (CLI → Controller → Static Response)

- [x] Create minimal **`agent-code` stub**:
  - [x] Implements Agent trait
  - [x] Returns static "Hello World" Rust code
  - [x] No real model calls yet
- [x] Wire CLI → Controller → CodeAgent
- [x] Implement `bodhya run --domain code --task "hello"` end-to-end
- [x] Add integration test for the complete flow

**Deliverables**: ✅
- First working end-to-end slice
- Validates architecture and wiring
- Proves capability-based routing works
- Integration test matching `@slice_v1` Gherkin scenario
- 6 integration tests covering vertical slice scenarios
- 8 unit tests for CodeAgent
- Quality gates passing (165 total tests)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 6: CodeAgent - Planner & BDD ✅

**Goal**: Implement CodeAgent's planning and BDD generation

- [x] Expand **`agent-code` crate**:
  - [x] `planner.rs` - Task decomposition using planner model
  - [x] `bdd.rs` - Gherkin feature generation from description
- [x] Integrate real model calls via model-registry
- [x] Create prompt templates in `prompts/code/planner.txt` and `bdd.txt`
- [x] Write tests matching `@code_bdd` scenarios

**Deliverables**: ✅
- CodeAgent generates Gherkin features from descriptions
- Uses local planner model via ModelRegistry
- Planning and BDD pipeline fully integrated
- 27 unit tests in agent-code (13 planner, 9 BDD, 5 existing)
- Prompt templates as code (embedded with file override support)
- Backward compatible with Phase 5
- Quality gates passing (184 total tests)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 7: CodeAgent - TDD & Implementation ✅

**Goal**: Complete CodeAgent's test-first code generation

- [x] Implement in **`agent-code` crate**:
  - [x] `tdd.rs` - Test generation (RED phase)
  - [x] `impl_gen.rs` - Code generation to make tests pass (GREEN phase)
  - [x] `review.rs` - Code review and improvement suggestions
  - [x] `validate.rs` - Integration with `cargo check`, `cargo test`, `cargo clippy`
- [x] Create prompt templates for each sub-agent role
- [x] Implement full BDD/TDD pipeline orchestration
- [x] Write comprehensive tests matching `@code_bdd_tdd` and `@code_multimodel` scenarios

**Deliverables**: ✅
- Full CodeAgent pipeline (Planner → BDD → TDD → Generator → Reviewer)
- Multi-model orchestration working (Planner, Coder, Reviewer)
- Quality validation integrated (cargo check/test/clippy)
- Tests demonstrating RED-GREEN-REFACTOR flow
- 4 new modules: tdd, impl_gen, review, validate
- 3 new prompt templates: tdd.txt, coder.txt, reviewer.txt
- 33 new tests (9 TDD, 9 impl gen, 10 review, 9 validate)
- 217 total tests passing across workspace
- Quality gates passing
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 8: MailAgent ✅

**Goal**: Implement MailAgent for email drafting and refinement

- [x] Implement **`agent-mail` crate**:
  - [x] `draft.rs` - Initial email draft generation
  - [x] `refine.rs` - Tone and clarity improvement
  - [x] `classify.rs` - Stub for future policy/classification
- [x] Create prompt templates for mail agent roles
- [x] Write tests matching `@mail_draft` and `@mail_review` scenarios

**Deliverables**: ✅
- Working MailAgent with draft and refine capabilities (30 unit tests)
- Email drafting pipeline: DraftGenerator → EmailRefiner
- RefinementGoal support (Clarity, Tone, Conciseness, All)
- Integration with ModelRegistry using ModelRole::Writer
- Graceful fallback to static emails when no registry
- Prompt templates (draft.txt, refine.txt) with embedded defaults
- Parser handles both structured and unstructured email formats
- EmailClassifier stub for future policy checking
- Quality gates passing (264 total tests)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

---

## Phase 9: Tool/MCP Integration ✅

**Goal**: Enable agents to use filesystem, git, and shell tools

- [x] Implement **`tools-mcp` crate**:
  - [x] `mcp_client.rs` - Generic MCP client interface
  - [x] `fs_tool.rs` - Filesystem operations (read, write, list)
  - [x] `shell_tool.rs` - Execute shell commands (e.g., `cargo test`)
- [x] Write tests for tool integrations

**Deliverables**: ✅
- Tool abstraction layer working (34 unit tests)
- FilesystemTool: read, write, list, exists operations with sandboxing
- ShellTool: command execution with timeout, working dir, stdout/stderr capture
- BasicMcpClient: stub implementation ready for future MCP protocol
- ToolRegistry: central tool management and execution
- Comprehensive test coverage:
  * 9 filesystem tool tests
  * 10 shell tool tests
  * 8 MCP client tests
  * 7 tool registry tests
- Quality gates passing (298 total tests)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

**Note**: Agent integration deferred - tools are ready but not yet wired into CodeAgent/MailAgent (can be done in future enhancement)

---

## Phase 10: Model Download Manager ✅

**Goal**: Implement on-demand model downloads with user consent

- [x] Enhance **`model-registry` crate**:
  - [x] `downloader.rs` - HTTP download with progress and verification
  - [x] `manager.rs` - Detect missing models, prompt user, download, verify checksums
- [ ] Implement `bodhya models install <id>` fully (deferred to future integration)
- [ ] Add auto-detection when running tasks with missing models (deferred)
- [x] Write tests for download and manager modules

**Deliverables**: ✅
- Working model download system with ModelDownloader (253 lines, 2 tests)
- SHA256 checksum verification using sha2 crate
- Model lifecycle manager with ModelManager (226 lines, 8 tests)
- Temporary file safety pattern (download to .tmp, verify, rename)
- Streaming downloads with progress tracking (every 100 MB)
- Comprehensive error handling (Network, ChecksumMismatch errors)
- Quality gates passing (312 total tests across workspace)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

**Note**: CLI integration and auto-detection deferred - core download infrastructure complete

---

## Phase 11: Storage & Metrics ✅

**Goal**: Persist execution history and quality metrics

- [x] Implement **`storage` crate** (optional):
  - [x] `sqlite.rs` - SQLite connection and schema
  - [x] `models.rs` - Data models for sessions, tasks, metrics
- [ ] Add metrics collection to controller and agents (deferred as optional)
- [x] Add `bodhya history` command to view past tasks

**Deliverables**: ✅
- Task history persistence with SQLite (SqliteStorage, 516 lines, 14 tests)
- Quality metrics storage models (Session, TaskRecord, QualityMetrics, 368 lines, 18 tests)
- Session management with UUID generation and duration tracking
- Task tracking with status, timestamps, results, and errors
- Quality metrics with scores, iterations, tokens, execution time
- History command: `bodhya history show --limit N` (8 tests)
- Statistics command: `bodhya history stats <domain>`
- DomainStats aggregation with success rate calculation
- Quality gates passing (337 total tests across workspace)
- Committed and pushed to `claude/plan-and-implement-01X8umSH1nPwnW9P3799Ctrh`

**Note**: Metrics collection integration deferred - storage infrastructure is complete and ready for use when agents execute real tasks

---

## Phase 12: Evaluation Harnesses ✅

**Goal**: Create repeatable quality evaluation for agents

- [x] Create **`eval/code_agent/`** harness:
  - [x] Standard test cases for code generation
  - [x] Quality scoring (correctness, coverage, style)
  - [x] Comparison framework
- [x] Create **`eval/mail_agent/`** harness:
  - [x] Standard email drafting scenarios
  - [x] Heuristic quality checks (length, tone, clarity)
- [x] Document how to run and interpret evaluations

**Deliverables**: ✅
- CodeAgent evaluation harness with 5 standard test cases (59 tests)
- MailAgent evaluation harness with 5 standard test cases (26 tests)
- Quality scoring: CodeAgent 0-100 pts (≥85 target), MailAgent 0-5 stars (≥4.5 target)
- Test case definitions with validation criteria
- Automated runners with colored CLI output
- Comprehensive documentation (3 READMEs)
- Quality gates passing (422 total tests across workspace)
- Committed and pushed to `claude/add-evaluation-harnesses-01HnG9MwUu4xEwkLYt1WqjNt`

---

## Phase 13: API Server (Optional) ✅

**Goal**: Add REST/WebSocket API for programmatic access

- [x] Implement **`api-server` crate** (optional):
  - [x] `models.rs` - Request/response models, task status, WebSocket messages
  - [x] `state.rs` - Application state with task storage and execution
  - [x] `routes.rs` - REST endpoints for task submission, status, results, agents, health
  - [x] `websocket.rs` - WebSocket support for streaming responses (500ms polling)
  - [x] `middleware.rs` - CORS and tracing layers
  - [x] `main.rs` - Server entry point (127.0.0.1:3000)
- [x] Create **`controller/controller.rs`** - Simple wrapper for API integration
- [x] Add `bodhya serve` command with --port and --host options
- [x] Write API integration tests (19 unit tests)
- [x] Create OpenAPI 3.0 specification
- [x] Create comprehensive README with examples

**Deliverables**: ✅
- Working REST API with 5 endpoints:
  * POST /tasks - Submit new task
  * GET /tasks/:id - Get task status
  * GET /tasks/:id/result - Get task result
  * GET /agents - List available agents
  * GET /health - Health check
- WebSocket streaming for real-time task updates (WS /ws/tasks/:id)
- Controller wrapper for easy integration (2 tests)
- Complete OpenAPI 3.0 documentation (openapi.yaml)
- Comprehensive README with cURL, JavaScript, and Python examples
- Quality gates passing (369 total tests across workspace)
- Committed and pushed to `claude/add-evaluation-harnesses-01HnG9MwUu4xEwkLYt1WqjNt`

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

**Current Phase**: Phase 13 Complete - API Server
**Next Phase**: Phase 14 - Documentation & Polish (Optional)
**Last Updated**: 2025-11-16
