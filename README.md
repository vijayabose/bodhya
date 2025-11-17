# Bodhya

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)]()
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-458%20passing-brightgreen.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Bodhya** is a local-first, multi-agent AI platform that intelligently routes tasks to specialized domain agents. Built in Rust for performance and reliability, Bodhya prioritizes privacy by running AI models locally on your hardware, with optional remote model integration when configured.

## ✨ Features

- **🏠 Local-First**: All AI processing happens on your device by default
- **🎯 Domain-Specific Agents**: Specialized agents for code generation, email writing, and more
- **🔄 Multi-Model Orchestration**: Uses different models for planning, generation, and review
- **🧪 Quality Through Testing**: Built-in BDD/TDD workflow for code generation
- **🔌 Pluggable Architecture**: Easy to add new agents via configuration
- **📊 Built-in Evaluation**: Quality scoring harnesses for continuous improvement
- **🌐 REST & WebSocket API**: Optional API server for programmatic access
- **💾 Task History**: SQLite-based storage for execution history and metrics

## 🚀 Quick Start

### Prerequisites

- Rust 1.75 or later
- 16-32 GB RAM recommended
- GPU or Apple Silicon for optimal performance

### Installation

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/vijayabose/bodhya/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr https://raw.githubusercontent.com/vijayabose/bodhya/main/scripts/install.ps1 | iex
```

**From Source:**
```bash
git clone https://github.com/vijayabose/bodhya.git
cd bodhya
cargo build --release
cargo install --path crates/cli
```

### Initialize Bodhya

```bash
# Initialize with code generation profile
bodhya init --profile code

# Or initialize with full profile (all agents)
bodhya init --profile full
```

This creates:
- `~/.bodhya/config/default.yaml` - Configuration file
- `~/.bodhya/models.yaml` - Model manifest
- `~/.bodhya/models/` - Model storage directory

## 📖 Usage

### Code Generation

Generate Rust code with built-in BDD/TDD workflow:

```bash
bodhya run --domain code --task "Create a function to calculate fibonacci numbers"
```

Bodhya will:
1. **Plan** the implementation approach
2. **Generate BDD scenarios** (Gherkin features)
3. **Write failing tests** (TDD - Red phase)
4. **Implement the code** (Green phase)
5. **Review and refine** (Refactor phase)
6. **Validate** with cargo check, test, and clippy

### Email Writing

Draft and refine professional emails:

```bash
bodhya run --domain mail --task "Write a follow-up email to a customer about their inquiry"
```

The MailAgent will:
1. **Draft** the initial email
2. **Refine** for tone and clarity
3. **Check** against policy (if configured)

### Model Management

```bash
# List available models
bodhya models list

# Install a specific model
bodhya models install qwen2.5-coder-7b-instruct

# Remove a model
bodhya models remove qwen2.5-coder-7b-instruct
```

Models are downloaded on-demand with checksum verification.

### View Task History

```bash
# Show recent tasks
bodhya history show --limit 10

# Show statistics by domain
bodhya history stats code
bodhya history stats mail
```

### API Server

Start the REST/WebSocket API server:

```bash
bodhya serve --host 127.0.0.1 --port 3000
```

Or run the standalone server:

```bash
cargo run --bin bodhya-server
```

API Endpoints:
- `POST /tasks` - Submit a new task
- `GET /tasks/:id` - Get task status
- `GET /tasks/:id/result` - Get task result
- `GET /agents` - List available agents
- `GET /health` - Health check
- `WS /ws/tasks/:id` - WebSocket for real-time updates

See `crates/api-server/README.md` for API documentation.

## 🏗️ Architecture

Bodhya follows a modular, capability-based architecture:

```
┌─────────────────────────────────────┐
│         CLI / API Layer             │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│    Central Controller Agent         │
│  • Task classification & routing    │
│  • Agent selection (capability)     │
│  • Engagement mode management       │
└────────────┬────────────────────────┘
             │
      ┌──────┴──────┐
      │             │
┌─────▼─────┐  ┌───▼──────┐
│CodeAgent  │  │MailAgent │  ... Future Agents
│• Planner  │  │• Drafter │
│• BDD/TDD  │  │• Refiner │
│• Generator│  │          │
│• Reviewer │  │          │
└─────┬─────┘  └───┬──────┘
      │            │
      └──────┬─────┘
             │
┌────────────▼────────────────────────┐
│   Model Registry & Inference        │
│  • Local models (mistral.rs)        │
│  • Role-based selection              │
│  • On-demand downloads               │
└────────────┬────────────────────────┘
             │
┌────────────▼────────────────────────┐
│      Tool / MCP Layer               │
│  • Filesystem, Git, Shell           │
│  • MCP server integrations          │
└─────────────────────────────────────┘
```

### Key Concepts

**Agents**: Domain-specific modules that handle tasks (CodeAgent, MailAgent, etc.)

**Capabilities**: Metadata describing what an agent can do (domain, intents, description)

**Engagement Modes**:
- `Minimum`: Local-only (v1 default)
- `Medium`: Local primary, remote fallback (future)
- `Maximum`: Remote heavily used (future)

**Model Roles**: Planner, Coder, Reviewer, Writer - specialized models for different tasks

## 🧪 Quality & Testing

Bodhya includes comprehensive evaluation harnesses:

**Code Agent Evaluation:**
```bash
cd eval/code_agent
cargo run
```

Scores generated code on:
- Correctness (0-40 points)
- Style (0-30 points)
- Coverage (0-30 points)
- **Target**: ≥85/100

**Mail Agent Evaluation:**
```bash
cd eval/mail_agent
cargo run
```

Scores emails on:
- Tone (0-1.5 stars)
- Clarity (0-1.5 stars)
- Length appropriateness (0-1.0 stars)
- Completeness (0-1.0 stars)
- **Target**: ≥4.5/5.0

## 📊 Development Status

| Component | Status | Tests | Coverage |
|-----------|--------|-------|----------|
| Core Abstractions | ✅ Complete | 65 | High |
| Model Registry | ✅ Complete | 46 | High |
| Controller | ✅ Complete | 44 | High |
| CLI | ✅ Complete | 63 | Medium |
| CodeAgent | ✅ Complete | 80 | High |
| MailAgent | ✅ Complete | 30 | High |
| Tools/MCP | ✅ Complete | 67 | High |
| Storage | ✅ Complete | 25 | High |
| API Server | ✅ Complete | 19 | Medium |
| Evaluation | ✅ Complete | 34 | High |
| **Total** | **✅** | **458** | **~80%** |

## 🛠️ Development

### Prerequisites

- Rust 1.75+
- SQLite 3
- Git

### Build from Source

```bash
git clone https://github.com/vijayabose/bodhya.git
cd bodhya

# Run quality gates (fmt, clippy, tests)
./scripts/check_all.sh

# Build all crates
cargo build --release

# Run specific crate
cargo run -p bodhya-cli -- --help
```

### Project Structure

```
bodhya/
├── crates/
│   ├── core/              # Shared traits & types
│   ├── controller/        # Task routing & orchestration
│   ├── model-registry/    # Model manifest & backends
│   ├── tools-mcp/         # Tool integrations
│   ├── agent-code/        # Code generation agent
│   ├── agent-mail/        # Email writing agent
│   ├── storage/           # SQLite persistence
│   ├── cli/               # CLI application
│   └── api-server/        # REST/WebSocket API
├── eval/
│   ├── code_agent/        # Code quality evaluation
│   └── mail_agent/        # Email quality evaluation
├── documents/             # Design documentation
└── scripts/               # Build & install scripts
```

### Adding a New Agent

1. Create new crate: `crates/agent-myagent/`
2. Implement the `Agent` trait from `bodhya_core`
3. Define `AgentCapability` with domain and intents
4. Add to workspace `Cargo.toml`
5. Register in config file
6. Write tests

See `documents/bodhya_code_design.md` for detailed guidelines.

## 📚 Documentation

- **[User Guide](USER_GUIDE.md)** - Comprehensive usage instructions
- **[Developer Guide](DEVELOPER_GUIDE.md)** - Agent development guide
- **[API Documentation](crates/api-server/README.md)** - REST/WebSocket API
- **[Design Documents](documents/)** - Architecture and design specs
- **[CLAUDE.md](CLAUDE.md)** - AI assistant development guide

## 🔒 Privacy & Security

- **Local-First**: All processing happens on your device by default
- **No Telemetry**: Bodhya does not send any data to external servers
- **Optional Remote**: Remote models only used when explicitly configured
- **Model Verification**: SHA256 checksums for all downloaded models
- **Open Source**: Fully auditable codebase

## 🎯 Roadmap

- [x] Phase 1-11: Core platform, agents, tools, storage
- [x] Phase 12: Evaluation harnesses
- [x] Phase 13: API Server
- [x] Phase 14: Documentation & Polish
- [ ] Phase 15: Model fine-tuning support
- [ ] Phase 16: Additional domain agents (research, data analysis)
- [ ] Phase 17: GUI application
- [ ] Phase 18: Cloud-sync for sessions (optional)

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run quality gates (`./scripts/check_all.sh`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

Please ensure:
- All tests pass
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- Test coverage ≥80% for new code

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Model inference via [mistral.rs](https://github.com/EricLBuehler/mistral.rs)
- Inspired by multi-agent AI systems and local-first principles

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/vijayabose/bodhya/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vijayabose/bodhya/discussions)
- **Documentation**: [docs/](documents/)

## ⭐ Star History

If you find Bodhya useful, please consider giving it a star on GitHub!

---

**Made with ❤️ by developers who value privacy and quality.**
