# BODHYA – Business Requirements Document (BRD)

**Project Name:** Bodhya  
**Version:** 1.2  
**Vision:** A local-first, modular, multi-agent AI platform that routes tasks to domain-specific agents (code, email, etc.), using the best-suited local models first, with optional remote models when configured.

---

## 1. Business Objectives

1. **Local-first intelligence**  
   - Use multiple local models specialized by domain (code-gen, email, summarization, etc.).  
   - Keep data on-device for privacy, cost savings, and latency.

2. **Quality through layered checking**  
   - Use local models to critique, review, and refine outputs from other models.  
   - Compose “checker” sub-agents around producer agents (e.g., code generator → code reviewer → test synthesizer).

3. **Expandable multi-domain platform**  
   - Initial domains:  
     - **Code Generation** (agents + sub-agents)  
     - **Email Writing & Communication** (agents + sub-agagents)  
   - Architecture must support future domains (summarization, document Q&A, planning, etc.) without core rewrites.

4. **Optional remote model integration (future-ready)**  
   - Configurable modes:  
     - **Minimum engagement:** local only  
     - **Medium engagement:** local primary, remote for difficult tasks  
     - **Maximum engagement:** remote heavily used  
   - Not required in v1 runtime behavior, but must be cleanly supported in design (traits, enums, configuration).

5. **Pluggable, modular design (MUST)**  
   - Agents as plug-in modules (config-driven).  
   - Support MCP servers and tools as first-class citizens.  
   - Easy to enable/disable agents and models per environment.  
   - New domain agents must be addable through:  
     1. A new crate implementing a common `Agent` trait.  
     2. A configuration entry (no controller code changes).

6. **Single installer, on-demand models (“batteries included but lightweight”)**  
   - Provide a **single installer / binary** that sets up Bodhya, config, scripts, and folders.  
   - Do **not** ship all models inside the installer.  
   - Instead, ship a **model manifest** and a **model manager** that:  
     - Detects missing models at first use.  
     - Prompts user for consent and downloads required models on demand.  
     - Verifies checksums and caches models locally.  

---

## 2. In Scope (v1)

- **Central controller agent** to:  
  - Parse tasks.  
  - Select domain agent (code, mail, etc.) using a routing strategy and capability metadata.  
  - Decide local-model strategy and log where remote escalation *would* be used (design only).  

- **Domain agents:**  
  1. **Bodhya.CodeAgent**
     - Multi-model orchestration (planner + writer + tester + refiner).
     - BDD/TDD pipeline for code using an inside-out approach.
  2. **Bodhya.MailAgent**
     - Draft emails, replies, and summaries.
     - Tone/format control, with checker sub-agent for clarity + brevity.

- **Model registry & model manager (local-first):**  
  - Local inference via **mistral.rs** for multiple models.  
  - Roles: `planner`, `coder`, `reviewer`, `writer`, etc.  
  - A model manifest file (e.g., `models.yaml`) describing:  
    - Logical role → model definition (name, size, source URL, checksum).  
  - CLI support for:  
    - `bodhya models list`  
    - `bodhya models install <role>`  
    - `bodhya models remove <id>`

- **Installation & initialization flow:**  
  - Single installer installs the `bodhya` binary, config templates, `scripts/check_all.sh`, and base folder structure (`~/.bodhya/`).  
  - `bodhya init` command:  
    - Creates user config.  
    - Offers profiles (code-only, mail-only, full).  
    - Optionally pre-downloads recommended models.

- **Config system** to control:  
  - Which agents are active.  
  - Which models are bound to which roles (planner/coder/reviewer/writer).  
  - Remote engagement policy (min/medium/max – config-wise, v1 behavior = minimum only).

- **Evaluation & quality gates harness:**  
  - A `scripts/check_all.sh` script that runs:  
    - `cargo fmt --check`  
    - `cargo clippy --all-targets -- -D warnings`  
    - `cargo test --all`  
    - `cargo audit` / `cargo deny` (optional strictness)  
  - Initial evaluation harness for:  
    - CodeAgent quality.  
    - MailAgent style/clarity checks (heuristic).  

---

## 3. Out of Scope (v1)

- Full remote LLM orchestration beyond design stubs and traits.  
- Collaborative multi-user environment.  
- Rich web UI (focus on CLI + simple API in v1).  
- Advanced scheduling/orchestration (multi-node, cluster mode).

---

## 4. KPIs

| KPI                            | Target                                  |
|--------------------------------|------------------------------------------|
| Local code-gen quality         | ≥ 85/100 internal quality score         |
| Local email quality            | ≥ 4.5/5 user rating                     |
| Coverage (code agent)          | ≥ 80%                                   |
| Remote model calls (min mode)  | 0 in v1 (design must support > 0 later) |
| Agent pluggability             | Add new domain with 1 new crate + config entry |
| `check_all.sh` pass rate       | 100% before any tagged release          |
| Installer success rate         | 95% of users can run `bodhya init` without manual fixes |

---

## 5. Constraints & Assumptions

- Rust 1.75+ and async ecosystem (Tokio).  
- Local inference via mistral.rs for multiple models.  
- Consumer hardware: 16–32 GB RAM, single GPU or M-series SoC.  
- Test tools available: `cargo test`, `cargo clippy`, `cargo-audit`, `cargo-deny`, coverage tooling.  
- Data remains local by default; remote connectors must be explicit and configurable.  
- The system is primarily used via CLI by individual developers or power users.

---

## 6. Development Approach (For Human + AI Co-Development)

1. **Thin vertical slices first**  
   - Start with a minimal slice: CLI → Controller → CodeAgent → static response.  
   - Gradually layer in model registry, BDD/TDD, and sub-agents.

2. **Inside-out implementation**  
   - Implement smallest pure types and traits first (`core` crate).  
   - Then implement controller routing and model registry.  
   - Then add domain agents with the simplest possible behavior.  
   - Finally, add more complex agent internal workflows.

3. **Strict TDD + BDD**  
   - Derive Rust test modules from Gherkin scenarios.  
   - Write failing tests first, then implement minimal code to pass, then refactor.

4. **Prompt as code**  
   - Keep LLM prompts in versioned files (e.g., `prompts/`) per domain/role.  
   - Treat changes to prompts similarly to code changes (with review and history).

5. **Do-not-break rules**  
   - `scripts/check_all.sh` must pass before merging/releasing.  
   - BRD and design documents must not be modified by automated agents unless explicitly requested.  
   - No remote network calls in v1 behavior without explicit configuration and clear user consent.  
   - Installer must not silently download large models; always show size + prompt user.
