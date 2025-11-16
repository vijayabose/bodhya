# Bodhya Tool Integration - Quick Reference Checklist

**Version**: 1.0
**Status**: Planning Phase
**Target**: v1.1 Release
**Duration**: 6 weeks (4 phases)

---

## Phase 1: Tool Integration Foundation (Weeks 1-2)

### Week 1: Core Types & Infrastructure

**Core Module Updates** (`crates/core/`)
- [ ] Add `ExecutionLimits` struct to `src/agent.rs`
- [ ] Add `tools: Arc<ToolRegistry>` to `AgentContext`
- [ ] Add `working_dir: PathBuf` to `AgentContext`
- [ ] Add `model_registry: Option<Arc<ModelRegistry>>` to `AgentContext`
- [ ] Add `execution_limits: ExecutionLimits` to `AgentContext`
- [ ] Update `AgentContext` builder methods
- [ ] Write unit tests
- [ ] Documentation updated

**CodeAgentTools Wrapper** (`crates/agent-code/`)
- [ ] Create `src/tools.rs`
- [ ] Define `CodeAgentTools` struct
- [ ] Implement `read_file()`
- [ ] Implement `write_file()`
- [ ] Implement `list_files()`
- [ ] Implement `file_exists()`
- [ ] Implement `run_command()`
- [ ] Implement `run_cargo()`
- [ ] Add execution statistics tracking
- [ ] Write comprehensive tests
- [ ] Export from `lib.rs`

**Controller Integration** (`crates/controller/`)
- [ ] Add `tools` field to `Controller`
- [ ] Update `Controller::new()`
- [ ] Update `Controller::with_defaults()`
- [ ] Pass tools to `AgentContext` in orchestrator
- [ ] Pass working_dir to `AgentContext`
- [ ] Pass model_registry to `AgentContext`
- [ ] Write integration tests

### Week 2: Agent & CLI Integration

**CodeAgent Execution** (`crates/agent-code/`)
- [ ] Update `handle()` to use `AgentContext`
- [ ] Update `generate_with_tdd()` signature
- [ ] Extract tools from context
- [ ] Write test file to disk
- [ ] Write implementation file to disk
- [ ] Execute `cargo test`
- [ ] Parse test output
- [ ] Handle execution errors
- [ ] Update fallback behavior
- [ ] Write execution tests

**CLI Updates** (`crates/cli/`)
- [ ] Add `--working-dir` flag
- [ ] Add `--execution-mode` flag
- [ ] Validate working directory
- [ ] Create `ToolRegistry::with_defaults()`
- [ ] Pass tools to controller
- [ ] Update help text
- [ ] Write CLI tests

**Integration Testing**
- [ ] Create `tests/integration/tool_integration_test.rs`
- [ ] Test hello world file generation
- [ ] Test command execution
- [ ] Test error handling
- [ ] Run full test suite
- [ ] Run quality gates

---

## Phase 2: Advanced Tool Capabilities (Weeks 3-4)

### Week 3: Edit & Search Tools

**EditTool** (`crates/tools-mcp/`)
- [ ] Create `src/edit_tool.rs`
- [ ] Define `EditTool` struct
- [ ] Define `EditOperation` enum
- [ ] Implement `replace` operation
- [ ] Implement `patch` operation
- [ ] Implement `insert_at_line` operation
- [ ] Implement `delete_lines` operation
- [ ] Add validation logic
- [ ] Add dry-run mode
- [ ] Write comprehensive tests
- [ ] Register in `ToolRegistry`

**SearchTool** (`crates/tools-mcp/`)
- [ ] Create `src/search_tool.rs`
- [ ] Define `SearchTool` struct
- [ ] Define `SearchMatch` struct
- [ ] Implement `grep` operation
- [ ] Implement `grep_recursive` operation
- [ ] Implement `find_definition`
- [ ] Implement `find_references`
- [ ] Add regex support
- [ ] Add file filtering
- [ ] Write comprehensive tests
- [ ] Register in `ToolRegistry`

**CodeAgentTools Extensions**
- [ ] Add `edit_file()` method
- [ ] Add `patch_file()` method
- [ ] Add `search_code()` method
- [ ] Add `find_definition()` method
- [ ] Write tests for new methods

### Week 4: Git Tool & Integration

**GitTool** (`crates/tools-mcp/`)
- [ ] Create `src/git_tool.rs`
- [ ] Define `GitTool` struct
- [ ] Define `GitStatus` struct
- [ ] Implement `status` operation
- [ ] Implement `diff` operations
- [ ] Implement `add` operation
- [ ] Implement `commit` operation
- [ ] Implement `push` operation (with safety)
- [ ] Implement `pull` operation
- [ ] Implement `branch` operations
- [ ] Implement `log` operation
- [ ] Add safety checks
- [ ] Write comprehensive tests
- [ ] Register in `ToolRegistry`

**CodeAgentTools Git Extensions**
- [ ] Add `git_status()` method
- [ ] Add `git_diff()` method
- [ ] Add `git_add()` method
- [ ] Add `git_commit()` method
- [ ] Add `git_push()` method
- [ ] Write git integration tests

**Advanced Integration Testing**
- [ ] Create `tests/integration/advanced_tools_test.rs`
- [ ] Test file editing workflow
- [ ] Test code search workflow
- [ ] Test git workflow
- [ ] Test combined tool usage
- [ ] Run full test suite
- [ ] Run quality gates

---

## Phase 3: Agentic Execution Loop (Week 5)

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

## Phase 4: Polish & Documentation (Week 6)

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
- [ ] Tools connected to agents
- [ ] CodeAgent writes actual files
- [ ] CodeAgent executes commands
- [ ] CodeAgent iterates on failures
- [ ] EditTool functional
- [ ] SearchTool functional
- [ ] GitTool functional
- [ ] End-to-end workflows complete

### Quality
- [ ] Test coverage ≥ 80%
- [ ] All quality gates pass
- [ ] Zero security issues
- [ ] Documentation complete
- [ ] Examples demonstrate features

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

**New Files: 25+**
- `crates/agent-code/src/tools.rs`
- `crates/agent-code/src/executor.rs`
- `crates/tools-mcp/src/edit_tool.rs`
- `crates/tools-mcp/src/search_tool.rs`
- `crates/tools-mcp/src/git_tool.rs`
- `prompts/code/coder_with_tools.txt`
- `prompts/code/error_analyzer.txt`
- `documents/bodhya_tool_integration_plan.md` ✓
- `documents/bodhya_tool_usage_guide.md`
- `documents/TOOL_INTEGRATION_CHECKLIST.md` ✓
- `examples/` directories and files
- `tests/integration/` test files

**Modified Files: 15+**
- `crates/core/src/agent.rs`
- `crates/core/src/config.rs`
- `crates/controller/src/controller.rs`
- `crates/controller/src/orchestrator.rs`
- `crates/agent-code/src/lib.rs`
- `crates/tools-mcp/src/lib.rs`
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
- ✅ This checklist created

**Next Steps:**
1. Review plan with stakeholders
2. Create feature branch: `feature/tool-integration`
3. Begin Phase 1, Week 1: Core Types & Infrastructure
4. Set up project tracking
5. Assign owners for each phase

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
