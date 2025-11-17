# CLAUDE.md - AI Assistant Guide for Bodhya

**Last Updated**: 2025-11-17
**Project**: Bodhya - Local-first Multi-Agent AI Platform
**Language**: Rust 1.75+
**Stage**: v1.0 Complete - Production Ready

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Repository Structure](#repository-structure)
3. [Core Documents Reference](#core-documents-reference)
4. [Development Philosophy](#development-philosophy)
5. [Architecture Overview](#architecture-overview)
6. [Implementation Guidelines](#implementation-guidelines)
7. [Quality Gates & Testing](#quality-gates--testing)
8. [Do-Not-Do List](#do-not-do-list)
9. [Git Workflow](#git-workflow)
10. [Common Tasks](#common-tasks)

---

## Project Overview

### What is Bodhya?

Bodhya is a **local-first, modular, multi-agent AI platform** designed to route tasks to domain-specific agents (code generation, email writing, etc.) using specialized local models first, with optional remote model integration when configured.

### Key Characteristics

- **Local-First Intelligence**: Uses multiple local models specialized by domain to keep data on-device for privacy, cost savings, and latency
- **Quality Through Layered Checking**: Uses local models to critique, review, and refine outputs from other models
- **Expandable Multi-Domain Platform**: Initially supports Code Generation and Email Writing, with architecture supporting future domains
- **Pluggable, Modular Design**: Agents as plug-in modules (config-driven), supporting MCP servers and tools as first-class citizens
- **Single Installer, On-Demand Models**: One installer with model manifest and manager that downloads required models on demand

### Target Users

- Individual developers and power users
- Consumer hardware: 16-32 GB RAM, single GPU or M-series SoC
- Primary interface: CLI (with optional API server)

---

## Repository Structure

### Current State (as of 2025-11-17)

**Implementation Status**: ✅ All phases complete (v1.0 ready)

```
bodhya/
├── README.md                    # Project overview & quick start
├── CLAUDE.md                    # This file - AI assistant guide
├── USER_GUIDE.md                # Comprehensive user manual
├── DEVELOPER_GUIDE.md           # Agent development guide
├── checklist.md                 # Implementation progress tracker
├── Cargo.toml                   # Workspace root
├── .git/                        # Git repository
├── documents/                   # Complete design documentation
├── crates/                      # All implemented crates
├── eval/                        # Evaluation harnesses
├── examples/                    # Example tasks and usage
├── prompts/                     # LLM prompt templates
└── scripts/                     # Build & install scripts
```

### Implemented Structure

```
bodhya/
├── Cargo.toml                   # Workspace root
├── scripts/
│   └── check_all.sh            # Quality gate script
├── eval/                        # Evaluation harnesses
│   ├── code_agent/
│   └── mail_agent/
└── crates/
    ├── core/                   # Shared traits & types
    ├── controller/             # Central controller agent
    ├── model-registry/         # Model manifest & backends
    ├── tools-mcp/              # MCP and tool integrations
    ├── agent-code/             # Code generation agent
    ├── agent-mail/             # Email writing agent
    ├── storage/                # Metrics & sessions (optional)
    ├── cli/                    # User-facing CLI
    └── api-server/             # REST/WebSocket API (optional)
```

**Status**: ✅ All crates fully implemented with 369 passing tests. v1.0 production ready.

---

## Core Documents Reference

### When to Read Each Document

| Document | Read When... | Key Contents |
|----------|-------------|--------------|
| `bodhya_brd.md` | Understanding business goals, KPIs, scope | Objectives, in/out of scope, constraints, development approach |
| `bodhya_system_design.md` | Designing architecture, understanding data flows | High-level architecture, agent contracts, installation design, NFRs |
| `bodhya_code_design.md` | Writing Rust code, creating crates | Workspace layout, core traits, CLI design, implementation strategy |
| `bodhya_claude_prompt.md` | Starting any implementation work | Core principles, do-not-do list, workflow instructions |
| `bodhya_gherkin_features.md` | Understanding user-facing features | High-level BDD scenarios for all major features |
| `bodhya_gherkin_use_cases.md` | Understanding end-to-end workflows | Complete use case scenarios |
| `bodhya_gherkin_test_cases.md` | Writing tests | Unit-level test scenarios mapped to Gherkin |

### Document Authority

**These documents are the source of truth.** Do not modify them unless explicitly requested by the user. Your role is to implement code that fulfills these documents, not to rewrite the design.

---

## Development Philosophy

### 1. Inside-Out Implementation

**Always start with the smallest internal types, traits, and pure functions, then build outward.**

Implementation order:
1. Core types and traits (`core` crate)
2. Model registry and manifest parsing (`model-registry`)
3. Controller routing logic (`controller`)
4. CLI commands (`cli`)
5. Domain agents (`agent-code`, `agent-mail`)
6. Full integration and workflows

**Never implement the UI layer first.**

### 2. Strict BDD + TDD

For every feature:
1. **Write failing tests first** (derived from Gherkin scenarios)
2. **Implement minimal code** to make tests pass (RED → GREEN)
3. **Refactor** to improve quality while keeping tests passing

Use the Gherkin files (`bodhya_gherkin_*.md`) as guidance for test design.

### 3. Thin Vertical Slices

Start with minimal end-to-end slices to validate architecture early:
- First slice: CLI → Controller → CodeAgent → static response
- Gradually layer in: model registry, BDD/TDD, sub-agents

### 4. Prompts as Code

- Keep LLM prompts in versioned files (e.g., `prompts/`) per domain/role
- Treat changes to prompts like code changes (with review and history)

---

## Architecture Overview

### High-Level Components

```
┌─────────────────────────────────────────────────────────┐
│                    CLI / API Layer                       │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────┐
│          Central Controller Agent                        │
│  • Task classification & routing                         │
│  • Agent selection (capability-based)                    │
│  • Engagement mode management                            │
│  • Logging & metrics                                     │
└─────────────────┬───────────────────────────────────────┘
                  │
        ┌─────────┴─────────┐
        │                   │
┌───────▼────────┐  ┌──────▼────────┐
│  CodeAgent     │  │  MailAgent    │  ... Future Agents
│  • Planner     │  │  • Drafter    │
│  • BDD/TDD     │  │  • Refiner    │
│  • Generator   │  │  • Classifier │
│  • Reviewer    │  │               │
└───────┬────────┘  └──────┬────────┘
        │                  │
        └─────────┬────────┘
                  │
┌─────────────────▼───────────────────────────────────────┐
│            Model Registry & Inference                    │
│  • Local models via mistral.rs                           │
│  • Model manifest (models.yaml)                          │
│  • Role-based model selection                            │
│  • On-demand model downloads                             │
└─────────────────┬───────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────┐
│              Tool / MCP Layer                            │
│  • Filesystem, Git, Shell                                │
│  • MCP server integrations                               │
└──────────────────────────────────────────────────────────┘
```

### Core Traits (Conceptual)

```rust
// Agent trait - all domain agents implement this
pub trait Agent: Send + Sync {
    fn id(&self) -> &'static str;
    fn capability(&self) -> AgentCapability;
    async fn handle(&self, task: Task, ctx: AgentContext)
        -> anyhow::Result<AgentResult>;
}

// Model backend trait
pub trait ModelBackend: Send + Sync {
    fn id(&self) -> &'static str;
    async fn generate(&self, req: ModelRequest)
        -> anyhow::Result<ModelResponse>;
}
```

### Agent Capability Contract

Agents expose metadata for intelligent routing:
- **Domain**: e.g., "code", "mail", "summarization"
- **Intents**: e.g., ["generate", "refine", "test"]
- **Description**: Human-readable explanation

This allows the controller to:
- Match task descriptions to agents without hardcoding
- Support dynamic agent registration via configuration

### Engagement Modes

```rust
pub enum EngagementMode {
    Minimum,  // Local only (v1 behavior)
    Medium,   // Local primary, remote fallback (future)
    Maximum,  // Remote heavily used (future)
}
```

**v1 Constraint**: Must be `Minimum` (local-only) in runtime behavior.

---

## Implementation Guidelines

### Starting Fresh Implementation

When beginning implementation:

1. **Create workspace structure** following `bodhya_code_design.md`
2. **Implement in this order**:
   - `core/src/errors.rs` - Error types
   - `core/src/config.rs` - Configuration structs
   - `core/src/model.rs` - Model traits and types
   - `core/src/agent.rs` - Agent trait and types
   - `core/src/tool.rs` - Tool/MCP interfaces
   - `model-registry/src/manifest.rs` - Parse models.yaml
   - `model-registry/src/registry.rs` - Model lookup
   - `controller/src/routing.rs` - Agent selection
   - `cli/src/main.rs` - Basic CLI structure

3. **Write tests first** for each module
4. **Keep each PR/commit focused** on a single crate or feature

### Rust Code Style

- **Idiomatic Rust**: Use `Result<T, E>` with meaningful error types
- **Composition over inheritance**: Prefer traits and composition
- **No global mutable state**: Use dependency injection
- **Async/await**: Use Tokio for async runtime
- **Error handling**: Use `anyhow` for application errors, custom types for library errors

### Configuration Management

- Default config location: `~/.bodhya/config/default.yaml`
- Model manifest: `~/.bodhya/models.yaml` (or bundled in binary)
- Profiles: `code`, `mail`, `full`

### Model Management

Models are **not** bundled in the installer. The flow is:

1. User runs `bodhya init` → creates config
2. User runs a task requiring a model
3. Model manager detects missing model
4. Shows size, source, checksum
5. Prompts for confirmation
6. Downloads and verifies model
7. Caches in `~/.bodhya/models/`

---

## Quality Gates & Testing

### Quality Gate Script: `scripts/check_all.sh`

**All changes must pass these checks before commit/merge:**

```bash
#!/bin/bash
set -e

echo "Running cargo fmt check..."
cargo fmt --check

echo "Running cargo clippy..."
cargo clippy --all-targets -- -D warnings

echo "Running cargo test..."
cargo test --all

echo "Running cargo audit (optional)..."
cargo audit || true

echo "All checks passed!"
```

### Testing Strategy

1. **Unit Tests**: In `#[cfg(test)]` modules within each crate
2. **Integration Tests**: In `tests/` directory at workspace root
3. **BDD Tests**: Derive from Gherkin scenarios in `bodhya_gherkin_test_cases.md`
4. **Evaluation Harnesses**: In `eval/` directory for CodeAgent and MailAgent quality

### Coverage Target

- Code Agent: ≥ 80% coverage
- Other crates: Aim for high coverage of critical paths

---

## Do-Not-Do List

**CRITICAL: Never do these things without explicit user permission:**

1. ❌ **Do not modify BRD or design documents** unless explicitly instructed
2. ❌ **Do not hardcode model paths** or ship models inside the binary
3. ❌ **Do not add remote network calls** or remote LLM usage in v1 behavior
4. ❌ **Do not invent new crates** beyond what `bodhya_code_design.md` describes
5. ❌ **Do not introduce OS-specific hacks** unless absolutely necessary
6. ❌ **Do not skip quality gates** (`check_all.sh` must pass)
7. ❌ **Do not silently download large models** - always show size and prompt
8. ❌ **Do not break existing tests** - keep them passing during refactoring
9. ❌ **Do not implement UI first** - always inside-out approach
10. ❌ **Do not use remote engagement** in v1 runtime (design only)

---

## Git Workflow

### Branch Strategy

- **Development branch**: `claude/claude-md-mi0uztrucnnqwkm8-01K5uSnvHzoXGyGFXC8jv4XL`
- All development work happens on this branch
- Never push to main without explicit permission

### Commit Guidelines

1. **Write clear, descriptive commit messages**
   - Focus on "why" rather than "what"
   - Follow repository's existing style (check `git log`)

2. **Commit frequently** with focused changes
   - One feature/fix per commit when possible
   - Keep commits atomic and reversible

3. **Before committing**:
   ```bash
   # Always run quality gates
   ./scripts/check_all.sh

   # Check what will be committed
   git status
   git diff --staged
   ```

4. **Commit format** (example):
   ```bash
   git commit -m "$(cat <<'EOF'
   Implement core Agent trait and Task types

   - Add Agent trait with capability metadata
   - Define Task, AgentResult, and AgentContext structs
   - Include comprehensive unit tests
   - All tests passing, clippy clean
   EOF
   )"
   ```

### Push Guidelines

- Use: `git push -u origin claude/claude-md-mi0uztrucnnqwkm8-01K5uSnvHzoXGyGFXC8jv4XL`
- Retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s) on network errors
- Branch must start with 'claude/' and end with matching session id

---

## Common Tasks

### Task 1: Understanding the Project

```bash
# Read documents in this order:
1. README.md - Quick overview
2. CLAUDE.md - This file
3. documents/bodhya_brd.md - Business context
4. documents/bodhya_system_design.md - Architecture
5. documents/bodhya_code_design.md - Code structure
```

### Task 2: Starting Implementation

1. Read all documents first
2. Create workspace structure from `bodhya_code_design.md`
3. Start with `core` crate
4. Write tests first (TDD)
5. Implement minimal code to pass tests
6. Run `check_all.sh` frequently

### Task 3: Adding a New Agent

1. Create new crate in `crates/agent-<name>/`
2. Implement `Agent` trait from `core`
3. Define agent's `AgentCapability`
4. Add configuration entry
5. Write comprehensive tests
6. Update documentation if needed

### Task 4: Working with Models

1. Define model role in `core/src/model.rs`
2. Add model entry to `models.yaml` manifest
3. Implement model backend (local via mistral.rs)
4. Register in model registry
5. Test model selection and inference

### Task 5: Before Pushing Changes

```bash
# Run quality gates
./scripts/check_all.sh

# Review changes
git status
git diff

# Stage and commit
git add <files>
git commit -m "descriptive message"

# Push to development branch
git push -u origin claude/claude-md-mi0uztrucnnqwkm8-01K5uSnvHzoXGyGFXC8jv4XL
```

---

## Key Performance Indicators (KPIs)

Track these metrics during development:

| KPI | Target |
|-----|--------|
| Local code-gen quality | ≥ 85/100 internal quality score |
| Local email quality | ≥ 4.5/5 user rating |
| Coverage (code agent) | ≥ 80% |
| Remote model calls (min mode) | 0 in v1 |
| Agent pluggability | Add new domain with 1 crate + config |
| `check_all.sh` pass rate | 100% before release |
| Installer success rate | 95% can run `bodhya init` |

---

## Quick Reference

### Important Files

- `documents/bodhya_brd.md` - What and why
- `documents/bodhya_system_design.md` - How (architecture)
- `documents/bodhya_code_design.md` - How (code)
- `documents/bodhya_claude_prompt.md` - AI instructions

### Key Principles

1. Inside-out implementation (smallest pieces first)
2. BDD + TDD (tests first, minimal code, refactor)
3. Quality gates must pass (fmt, clippy, test, audit)
4. Local-first (no remote calls in v1)
5. Modular and pluggable (config-driven agents)

### Common Commands

```bash
# Initialize project (future)
bodhya init

# Manage models (future)
bodhya models list
bodhya models install <id>
bodhya models remove <id>

# Run tasks (future)
bodhya run --domain code --task "..."

# Development
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test --all
./scripts/check_all.sh
```

---

## Questions or Issues?

1. **Unclear requirement?** → Check BRD and system design docs
2. **Code structure question?** → Check code design doc
3. **Test guidance?** → Check Gherkin test cases doc
4. **Process question?** → Check Claude prompt doc
5. **Still unclear?** → Ask the user for clarification

---

## Version History

- **2025-11-17**: Updated for v1.0 release - all phases complete, production ready
- **2025-11-15**: Initial CLAUDE.md created - documentation phase complete, implementation pending

---

**Remember**: This project values **correctness, clarity, and modularity** over cleverness. When in doubt, prefer the simpler, more explicit approach that future maintainers (human or AI) will easily understand.
