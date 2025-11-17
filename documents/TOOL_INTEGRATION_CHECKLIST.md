# Bodhya Tool Integration - Quick Reference Checklist

**Version**: 1.1
**Status**: Phase 1 Complete, Phase 2 Week 3 Complete - Ready for MCP
**Target**: v1.1 Release
**Duration**: 6-7 weeks (revised: skip custom GitTool, use MCP)
**Last Updated**: 2025-11-17
**Scope Decision**: ✅ MCP extensibility (Phase 2.5) INCLUDED in v1.1
**Revised Approach**: ✅ Skip custom GitTool - use git via MCP server

---

## Phase 1: Tool Integration Foundation (Weeks 1-2)

### Week 1: Core Types & Infrastructure

**Core Module Updates** (`crates/core/`)
- [x] Add `ExecutionLimits` struct to `src/agent.rs`
- [x] Add `tools: Arc<ToolRegistry>` to `AgentContext`
- [x] Add `working_dir: PathBuf` to `AgentContext`
- [x] Add `model_registry: Option<Arc<ModelRegistry>>` to `AgentContext`
- [x] Add `execution_limits: ExecutionLimits` to `AgentContext`
- [x] Update `AgentContext` builder methods
- [x] Write unit tests
- [x] Documentation updated

**CodeAgentTools Wrapper** (`crates/agent-code/`)
- [x] Create `src/tools.rs`
- [x] Define `CodeAgentTools` struct
- [x] Implement `read_file()`
- [x] Implement `write_file()`
- [x] Implement `list_files()`
- [x] Implement `file_exists()`
- [x] Implement `run_command()`
- [x] Implement `run_cargo()`
- [x] Add execution statistics tracking
- [x] Write comprehensive tests
- [x] Export from `lib.rs`

**Controller Integration** (`crates/controller/`)
- [x] Add `tools` field to `Controller`
- [x] Update `Controller::new()`
- [x] Update `Controller::with_defaults()`
- [x] Pass tools to `AgentContext` in orchestrator
- [x] Pass working_dir to `AgentContext`
- [x] Pass model_registry to `AgentContext`
- [x] Write integration tests

### Week 2: Agent & CLI Integration

**CodeAgent Execution** (`crates/agent-code/`)
- [x] Update `handle()` to use `AgentContext`
- [x] Extract tools from context via `get_tools_from_context()`
- [x] Write test file to disk via `tools.write_file()`
- [x] Write implementation file to disk via `tools.write_file()`
- [x] Execute `cargo test` via `tools.run_cargo()`
- [x] Parse test output (success/failure detection)
- [x] Handle execution errors (graceful error reporting)
- [x] Update fallback behavior (falls back to model-based)
- [x] Write execution tests (`test_determine_file_paths()`)
- [x] Implement full 7-step TDD workflow in `execute_with_tools()`

**CLI Updates** (`crates/cli/`)
- [x] Add `--working-dir` flag
- [ ] Add `--execution-mode` flag
- [x] Validate working directory
- [x] Create `ToolRegistry::with_defaults()`
- [x] Pass tools to controller
- [x] Update help text
- [x] Write CLI tests

**Integration Testing**
- [x] Create `tests/integration/tool_integration_test.rs`
- [x] Test hello world file generation
- [x] Test command execution
- [x] Test error handling
- [x] Run full test suite (427 tests passing)
- [x] Run quality gates

---

## Phase 2: Advanced Tool Capabilities (Weeks 3-4)

### Week 3: Edit & Search Tools

**EditTool** (`crates/tools-mcp/`)
- [x] Create `src/edit_tool.rs`
- [x] Define `EditTool` struct
- [x] Define `EditOperation` enum
- [x] Implement `replace` operation
- [x] Implement `patch` operation
- [x] Implement `insert_at_line` operation
- [x] Implement `delete_lines` operation
- [x] Add validation logic
- [x] Add dry-run mode
- [x] Write comprehensive tests
- [x] Register in `ToolRegistry`

**SearchTool** (`crates/tools-mcp/`)
- [x] Create `src/search_tool.rs`
- [x] Define `SearchTool` struct
- [x] Define `SearchMatch` struct
- [x] Implement `grep` operation
- [x] Implement `grep_recursive` operation
- [x] Implement `find_definition`
- [x] Implement `find_references`
- [x] Add regex support
- [x] Add file filtering
- [x] Write comprehensive tests
- [x] Register in `ToolRegistry`

**CodeAgentTools Extensions**
- [x] Add `edit_file()` method
- [ ] Add `patch_file()` method
- [x] Add `search_code()` method
- [ ] Add `find_definition()` method
- [x] Write tests for new methods

### Week 4: ~~Git Tool~~ SKIPPED - Using Git via MCP Instead

> **DECISION**: Skip custom GitTool implementation
> **RATIONALE**: Use MCP server for git functionality instead
> **BENEFITS**:
> - Validates MCP architecture early with real-world use case
> - Saves 2-3 hours of implementation time
> - Leverages existing git MCP servers (proven and tested)
> - Demonstrates extensibility vision
> - Smaller core codebase

**GitTool** (`crates/tools-mcp/`) - ⏭️ **SKIPPED - Will use MCP**
- [~] ~~Create `src/git_tool.rs`~~ - Use git MCP server instead
- [~] ~~Define `GitTool` struct~~ - Use MCP client
- [~] ~~Define `GitStatus` struct~~ - MCP handles this
- [~] ~~Implement `status` operation~~ - Via MCP
- [~] ~~Implement `diff` operations~~ - Via MCP
- [~] ~~Implement `add` operation~~ - Via MCP
- [~] ~~Implement `commit` operation~~ - Via MCP
- [~] ~~Implement `push` operation (with safety)~~ - Via MCP
- [~] ~~Implement `pull` operation~~ - Via MCP
- [~] ~~Implement `branch` operations~~ - Via MCP
- [~] ~~Implement `log` operation~~ - Via MCP
- [~] ~~Add safety checks~~ - MCP server handles this
- [~] ~~Write comprehensive tests~~ - MCP integration tests instead
- [~] ~~Register in `ToolRegistry`~~ - MCP auto-discovery

**CodeAgentTools Git Extensions** - ⏭️ **DEFERRED to post-MCP**
- [ ] Add git support via MCP client (Phase 2.5)
- [ ] Test git operations via MCP (Phase 2.5)

**Advanced Integration Testing** - ⏭️ **MOVED to Phase 2.5**
- [ ] Test git workflow via MCP server
- [ ] Test combined tool usage with MCP tools

---

## Phase 2.5: MCP Server Extensibility (Week 5-6)

> **Status**: ✅ INCLUDED in v1.1 Release
> **Timeline**: Can be implemented in parallel with Phase 3, or sequentially
> **Purpose**: Enables users to extend Bodhya with external tools via CLI without code changes

### Quick Summary

**What MCP Extensibility Adds:**
- 🔧 **CLI Tool Management**: `bodhya tools add-mcp`, `remove-mcp`, `list-mcp`, `test-mcp`
- 🔌 **MCP Protocol Support**: Full JSON-RPC 2.0 stdio and HTTP MCP client
- 📦 **External Tool Discovery**: Automatically discover tools from MCP servers
- ⚙️ **Configuration-Driven**: No code changes needed - just YAML config
- 🌐 **Ecosystem Integration**: Connect to GitHub MCP, Brave Search, filesystem servers, etc.

**Key Benefits:**
- Users can extend Bodhya without modifying source code
- Plug into existing MCP ecosystem (20+ servers available)
- Enable/disable external tools via CLI
- Test MCP connections before using in production

### Configuration System

**Core Config Updates** (`crates/core/`)
- [ ] Add `ToolsConfig` struct to `src/config.rs`
- [ ] Add `builtin: Vec<String>` field
- [ ] Add `mcp_servers: Vec<McpServerConfig>` field
- [ ] Add `ToolsConfig` to `AppConfig`
- [ ] Enhance `McpServerConfig` with:
  - [ ] `enabled: bool` field
  - [ ] `headers: Option<HashMap<String, String>>` for HTTP
  - [ ] Support for environment variable expansion
- [ ] Write config serialization tests
- [ ] Update default config template

### Full MCP Client Implementation

**StdioMcpClient** (`crates/tools-mcp/`)
- [ ] Create enhanced `src/mcp_client.rs`
- [ ] Implement JSON-RPC 2.0 protocol
- [ ] Add process spawning with stdin/stdout
- [ ] Implement `initialize` request
- [ ] Implement `tools/list` for discovery
- [ ] Implement `tools/call` for execution
- [ ] Add environment variable expansion (`${VAR}`)
- [ ] Add connection management
- [ ] Add error handling and retries
- [ ] Write comprehensive tests
- [ ] Test with mock MCP server

**HttpMcpClient** (Optional)
- [ ] Create `src/mcp_client_http.rs`
- [ ] Implement HTTP-based MCP protocol
- [ ] Add header support
- [ ] Add authentication
- [ ] Write tests

### CLI Tool Management Commands

**Tools Command Module** (`crates/cli/`)
- [ ] Create `src/tools_cmd.rs`
- [ ] Define `ToolsCommand` enum with subcommands:
  - [ ] `List { mcp: bool }` - list tools
  - [ ] `AddMcp { ... }` - add MCP server
  - [ ] `RemoveMcp { name }` - remove server
  - [ ] `ToggleMcp { name, enable }` - enable/disable
  - [ ] `ListMcp` - show configured servers
  - [ ] `TestMcp { name }` - test connection
- [ ] Implement `list_tools()` function
- [ ] Implement `add_mcp_server()` function
- [ ] Implement `remove_mcp_server()` function
- [ ] Implement `toggle_mcp_server()` function
- [ ] Implement `list_mcp_servers()` function
- [ ] Implement `test_mcp_server()` function
- [ ] Add to main CLI router in `main.rs`
- [ ] Write CLI tests

### Integration with Tool System

**ToolRegistry MCP Loading** (`crates/tools-mcp/`)
- [ ] Add `load_mcp_servers()` method to `ToolRegistry`
- [ ] Connect to each enabled MCP server from config
- [ ] Discover tools from each server
- [ ] Wrap MCP tools to match `Tool` trait
- [ ] Register MCP tools in registry
- [ ] Add error handling for failed connections
- [ ] Write integration tests

**Controller Integration** (`crates/controller/`)
- [ ] Load MCP servers when creating `ToolRegistry`
- [ ] Pass MCP tools to `AgentContext`
- [ ] Add MCP connection status to metrics
- [ ] Handle MCP server failures gracefully

### Testing & Documentation

**MCP Integration Tests**
- [ ] Create `tests/integration/mcp_integration_test.rs`
- [ ] Test MCP server connection
- [ ] Test tool discovery
- [ ] Test tool execution via MCP
- [ ] Test with real MCP server (filesystem)
- [ ] Test error handling
- [ ] Test enable/disable workflow

**Documentation**
- [ ] Create MCP configuration guide
- [ ] Document available MCP servers
- [ ] Add troubleshooting section
- [ ] Add examples to README
- [ ] Update user guide with MCP workflows

**Example MCP Configurations**
- [ ] Add example for GitHub MCP server
- [ ] Add example for filesystem MCP server
- [ ] Add example for Brave Search MCP server
- [ ] Add example for custom HTTP server
- [ ] Document environment variable usage

---

## Phase 3: Agentic Execution Loop (Week 5 or 6)

**Executor Implementation** (`crates/agent-code/`)
- [ ] Create `src/executor.rs`
- [ ] Define `AgenticExecutor` struct
- [ ] Define `ExecutionPlan` struct
- [ ] Define `ExecutionStep` enum
- [ ] Define `ExecutionResult` struct
- [ ] Implement `execute_plan()`
- [ ] Implement `execute_with_retry()`
- [ ] Add error analysis logic
- [ ] Add refinement generation
- [ ] Write executor tests

**CodeAgent Integration**
- [ ] Add `executor` field to `CodeAgent`
- [ ] Create `generate_with_execution()` method
- [ ] Add execution mode config
- [ ] Support `generate_only` mode
- [ ] Support `execute_once` mode
- [ ] Support `execute_with_retry` mode
- [ ] Write mode switching tests

**Prompts for Agentic Behavior**
- [ ] Create `prompts/code/coder_with_tools.txt`
- [ ] Create `prompts/code/error_analyzer.txt`
- [ ] Update `prompts/code/reviewer.txt`
- [ ] Test prompts with samples

**Configuration**
- [ ] Add `ExecutionMode` to config
- [ ] Add execution limits to config
- [ ] Add git operation flags
- [ ] Update config templates
- [ ] Write config tests

**CLI Execution Support**
- [ ] Update `--execution-mode` for retry
- [ ] Add `--max-iterations` flag
- [ ] Add `--enable-git` flag
- [ ] Update help text
- [ ] Write CLI tests

**Agentic Integration Testing**
- [ ] Create `tests/integration/agentic_execution_test.rs`
- [ ] Test auto-fix scenario
- [ ] Test iteration limits
- [ ] Test complete workflow
- [ ] Test edge cases
- [ ] Run full test suite
- [ ] Run quality gates

---

## Phase 4: Polish & Documentation (Week 6-7)

**Documentation Updates**
- [ ] Update `bodhya_system_design.md` (already started)
- [ ] Update `bodhya_code_design.md`
- [ ] Create `bodhya_tool_usage_guide.md`
- [ ] Update Gherkin scenarios if needed
- [ ] Review all documentation

**Examples & Tutorials**
- [ ] Create `examples/hello_world_agent/`
- [ ] Create `examples/test_driven_agent/`
- [ ] Create `examples/git_workflow_agent/`
- [ ] Update main `README.md`
- [ ] Add architecture diagrams

**Performance Optimization**
- [ ] Profile tool operations
- [ ] Optimize file I/O
- [ ] Optimize search operations
- [ ] Optimize git operations
- [ ] Add benchmarks
- [ ] Document performance

**Security Audit**
- [ ] Review file operation sandboxing
- [ ] Test path traversal protection
- [ ] Review command execution safety
- [ ] Test command injection protection
- [ ] Review git operation safety
- [ ] Run `cargo-audit`
- [ ] Run `cargo-deny`
- [ ] Document security measures

**Final Testing**
- [ ] Run full test suite with verbose
- [ ] Check code coverage (target: 80%)
- [ ] Run quality gates
- [ ] Test on Linux
- [ ] Test on macOS
- [ ] Create release checklist

**Documentation Review**
- [ ] Verify accuracy
- [ ] Test code examples
- [ ] Check links
- [ ] Ensure consistency
- [ ] Spell/grammar check
- [ ] Peer review

---

## Dependencies to Add

```toml
# Add to workspace Cargo.toml

[workspace.dependencies]
regex = "1.10"        # SearchTool - pattern matching
ignore = "0.4"        # SearchTool - gitignore-aware traversal
git2 = "0.18"         # GitTool - libgit2 bindings
similar = "2.4"       # EditTool - diff/patch algorithms
shell-words = "1.1"   # MCP - command parsing
reqwest = "0.11"      # MCP - HTTP client (for HttpMcpClient)
```

---

## Quality Gates (Must Pass Before Each Phase Completion)

- [ ] `cargo fmt --check` - all code formatted
- [ ] `cargo clippy --all-targets -- -D warnings` - no warnings
- [ ] `cargo test --all` - all tests pass
- [ ] `cargo tarpaulin` - ≥80% coverage for new code
- [ ] `cargo audit` - no security vulnerabilities
- [ ] `./scripts/check_all.sh` - quality gates pass
- [ ] Integration tests pass
- [ ] Documentation updated
- [ ] Examples working

---

## Success Criteria

### Functional
- [x] Tools infrastructure exists
- [x] Tools connected to agents (via AgentContext)
- [x] CodeAgent writes actual files (✅ COMPLETE - writes test & impl files)
- [x] CodeAgent executes commands (✅ COMPLETE - runs cargo test)
- [ ] CodeAgent iterates on failures (pending Phase 3)
- [x] EditTool functional
- [x] SearchTool functional
- [~] GitTool functional (⏭️ SKIPPED - using git via MCP instead)
- [ ] MCP server integration working (Phase 2.5 - NEXT)
- [ ] External tools loadable via CLI (Phase 2.5 - NEXT)
- [ ] Git operations via MCP (Phase 2.5 - validates architecture)
- [ ] End-to-end workflows complete (pending Phase 3 & 4)

### Quality
- [x] Test coverage ≥ 80% (427 tests passing)
- [x] All quality gates pass (fmt, clippy, test, audit)
- [x] Zero security issues (cargo audit clean)
- [ ] Documentation complete (in progress, updated checklists)
- [ ] Examples demonstrate features (pending Phase 4)

### Performance
- [ ] File ops < 100ms
- [ ] Commands < 2s
- [ ] Search < 1s (10K files)
- [ ] Full cycle < 30s

---

## Risk Tracking

| Risk | Mitigation Status |
|------|------------------|
| Tool failures | [ ] Error handling implemented |
| Path traversal | [ ] Sandboxing validated |
| Command injection | [ ] Input sanitization tested |
| Infinite loops | [ ] Max iterations enforced |
| Performance issues | [ ] Profiling completed |
| Git conflicts | [ ] Pre-checks implemented |

---

## File Creation Summary

**New Files: 30+**
- `crates/agent-code/src/tools.rs`
- `crates/agent-code/src/executor.rs`
- `crates/tools-mcp/src/edit_tool.rs`
- `crates/tools-mcp/src/search_tool.rs`
- `crates/tools-mcp/src/git_tool.rs`
- `crates/cli/src/tools_cmd.rs` (MCP management)
- `prompts/code/coder_with_tools.txt`
- `prompts/code/error_analyzer.txt`
- `documents/bodhya_tool_integration_plan.md` ✓
- `documents/bodhya_tool_usage_guide.md`
- `documents/tool_extensibility_design.md` ✓
- `documents/TOOL_INTEGRATION_CHECKLIST.md` ✓
- `documents/IMPLEMENTATION_SUMMARY.md` ✓
- `examples/` directories and files
- `examples/mcp_servers/` (MCP configuration examples)
- `tests/integration/` test files
- `tests/integration/mcp_integration_test.rs`

**Modified Files: 20+**
- `crates/core/src/agent.rs`
- `crates/core/src/config.rs` (add ToolsConfig)
- `crates/core/src/tool.rs` (enhance McpServerConfig)
- `crates/controller/src/controller.rs`
- `crates/controller/src/orchestrator.rs`
- `crates/agent-code/src/lib.rs`
- `crates/tools-mcp/src/lib.rs`
- `crates/tools-mcp/src/mcp_client.rs` (full implementation)
- `crates/cli/src/main.rs` (add tools command)
- `crates/cli/src/run_cmd.rs`
- `documents/bodhya_system_design.md` ✓
- `documents/bodhya_code_design.md`
- `README.md`
- `Cargo.toml`

---

## Current Status

**Completed:**
- ✅ Analysis of current state
- ✅ Gap identification
- ✅ Comprehensive plan created
- ✅ System design updated (partial)
- ✅ Tool integration checklist created
- ✅ Tool extensibility design created
- ✅ Implementation summary created
- ✅ **Phase 1 Week 1: Core Types & Infrastructure - COMPLETE**
- ✅ **Phase 1 Week 2: Agent & CLI Integration - COMPLETE** (except --execution-mode flag)
- ✅ **Phase 2 Week 3: Edit & Search Tools - COMPLETE**
- ⏭️ **Phase 2 Week 4: Git Tool - SKIPPED** (using git via MCP instead)

**Implementation Summary (as of 2025-11-17):**
- ✅ ExecutionLimits added to AgentContext with defaults (3 max iterations, 20 file writes, 10 commands, 300s timeout)
- ✅ CodeAgentTools wrapper implemented with 13 comprehensive tests
- ✅ **CodeAgent execution COMPLETE** - Full 7-step TDD workflow:
  * Step 1: Planning (Planner)
  * Step 2: BDD Features (BddGenerator)
  * Step 3: Tests/RED Phase (TddGenerator)
  * Step 4: Implementation/GREEN Phase (ImplGenerator)
  * Step 5: Write files to disk (tools.write_file)
  * Step 6: Run tests (tools.run_cargo)
  * Step 7: Code Review (CodeReviewer)
- ✅ File path determination (fibonacci, factorial, hello, default patterns)
- ✅ EditTool fully implemented with replace, patch, insert, delete operations
- ✅ SearchTool fully implemented with grep, recursive search, regex, filtering
- ✅ --working-dir CLI flag added and functional
- ✅ Controller integrated with ToolRegistry
- ✅ Integration tests added and passing (controller→agent→tools flow)
- ✅ Total tests passing: **427 tests** (17 ignored)
- ✅ All quality gates passing (fmt, clippy, test, audit)
- ✅ Eval harnesses updated for new AgentContext structure

**Next Steps (REVISED APPROACH):**
1. ✅ ~~Phase 1 Complete~~
2. ✅ ~~Phase 2 Week 3 Complete~~
3. ⏭️ **Add `--execution-mode` CLI flag** (15-30 min) - Completes Phase 1 entirely
4. ⏭️ **Implement MCP Extensibility (Phase 2.5)** (2-3 days) - PRIORITY
   - Full MCP client (JSON-RPC 2.0)
   - CLI tool management (`bodhya tools add-mcp`, etc.)
   - **Use git MCP server as first integration** (validates architecture)
   - External tool discovery and loading
5. ⏭️ **Implement Agentic Execution Loop (Phase 3)** (1-2 days)
   - Observe-retry-fix workflow
   - Error analysis and refinement
6. ⏭️ **Polish & Documentation (Phase 4)** (1-2 days)

**Recommended Implementation Order:**
- ✅ **Revised approach adopted**: Skip custom GitTool, jump to MCP
- **Estimated Timeline**: 6-7 weeks total (saved 1-2 weeks)
- **Target Completion**: Early 2025
- **Git functionality**: Via MCP server (cleaner, faster, proves extensibility)

## Implementation Options

### ✅ Selected: Revised Approach - Skip GitTool, Prioritize MCP (6-7 weeks)
**DECISION**: Skip custom GitTool, use MCP server for git functionality
- **Timeline**: 6-7 weeks total (saved 1-2 weeks)
- **Scope**: All phases (1-4) + Phase 2.5 (MCP extensibility)
- **Outcome**: Complete tool integration with external tool extensibility
- **Revised Approach**:
  - ✅ Phase 1: Complete (Core + CodeAgent execution)
  - Week 4: Add --execution-mode flag + Start MCP implementation
  - Weeks 5-6: MCP extensibility (Phase 2.5) with git MCP as proof-of-concept
  - Week 6-7: Agentic Execution Loop (Phase 3)
  - Week 7: Polish & Documentation (Phase 4)

**Why This Is Better:**
- ✅ Validates MCP architecture early with real git use case
- ✅ Saves 2-3 hours of GitTool implementation time
- ✅ Leverages proven git MCP servers from ecosystem
- ✅ Smaller core codebase (less maintenance)
- ✅ Better demonstrates extensibility vision
- ✅ Git functionality via MCP server (proven and tested)

### Alternative Options (Not Selected)

**Option A: Full Feature Set Sequential (9 weeks)**
- All phases implemented sequentially
- More thorough but slower
- Includes custom GitTool (2-3 hours extra work)

**Option B: Core Features Only (6 weeks)**
- Defer MCP to v1.2
- Faster initial release but incomplete extensibility story
- Missing validation of MCP architecture

**Original Option C: GitTool then MCP (7-8 weeks)**
- Implement custom GitTool first
- Then do MCP extensibility
- Duplicates functionality that MCP provides

---

## Notes

- Follow inside-out implementation approach
- Write tests before code (TDD)
- Keep commits focused and atomic
- Run quality gates frequently
- Document as you go
- Backward compatibility maintained
- Graceful degradation ensured

---

**Legend:**
- [ ] Not started
- [x] In progress
- ✅ Complete
