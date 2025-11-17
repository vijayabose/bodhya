# Bodhya User Guide

**Version**: 1.0
**Last Updated**: 2025-11-17

Welcome to Bodhya, your local-first multi-agent AI platform! This guide will help you get the most out of Bodhya's powerful features.

## Table of Contents

1. [Installation](#installation)
2. [Getting Started](#getting-started)
3. [Configuration](#configuration)
4. [Using CodeAgent](#using-codeagent)
5. [Using MailAgent](#using-mailagent)
6. [Model Management](#model-management)
7. [Task History](#task-history)
8. [API Server](#api-server)
9. [Advanced Usage](#advanced-usage)
10. [Troubleshooting](#troubleshooting)
11. [FAQ](#faq)

---

## Installation

### System Requirements

- **Operating System**: Linux, macOS, or Windows
- **RAM**: 16-32 GB recommended
- **Disk Space**: 10-50 GB (depending on models)
- **Processor**: Modern multi-core CPU, GPU recommended
- **Rust**: 1.75 or later (for building from source)

### Quick Install

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/vijayabose/bodhya/main/scripts/install.sh | bash
```

**Windows (PowerShell as Administrator):**
```powershell
iwr https://raw.githubusercontent.com/vijayabose/bodhya/main/scripts/install.ps1 | iex
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/vijayabose/bodhya.git
cd bodhya

# Build and install
cargo build --release
cargo install --path crates/cli

# Verify installation
bodhya --version
```

---

## Getting Started

### Initialize Bodhya

After installation, initialize Bodhya with your preferred profile:

```bash
# Code generation profile (CodeAgent only)
bodhya init --profile code

# Mail writing profile (MailAgent only)
bodhya init --profile mail

# Full profile (all agents)
bodhya init --profile full
```

This creates:
- `~/.bodhya/config/default.yaml` - Your configuration
- `~/.bodhya/models.yaml` - Model manifest
- `~/.bodhya/models/` - Directory for downloaded models

### Your First Task

Let's generate some code:

```bash
bodhya run --domain code --task "Create a hello world function"
```

Bodhya will guide you through:
1. Planning the implementation
2. Generating BDD scenarios
3. Writing tests
4. Implementing the code
5. Reviewing and validating

---

## Configuration

### Configuration File

Location: `~/.bodhya/config/default.yaml`

```yaml
engagement_mode: minimum  # local-only in v1

paths:
  models_dir: ~/.bodhya/models
  cache_dir: ~/.bodhya/cache

agents:
  code:
    enabled: true
    models:
      planner: qwen2.5-coder-7b-instruct
      coder: qwen2.5-coder-7b-instruct
      reviewer: qwen2.5-coder-7b-instruct

  mail:
    enabled: true
    models:
      writer: qwen2.5-coder-7b-instruct

logging:
  level: info
  file: ~/.bodhya/logs/bodhya.log
```

### Profile Templates

**Code Profile** (`--profile code`):
- Enables: CodeAgent
- Models: Planner, Coder, Reviewer
- Best for: Software development tasks

**Mail Profile** (`--profile mail`):
- Enables: MailAgent
- Models: Writer
- Best for: Email drafting and refinement

**Full Profile** (`--profile full`):
- Enables: All agents
- Models: All required models
- Best for: General-purpose use

### Customizing Configuration

Edit `~/.bodhya/config/default.yaml`:

```yaml
# Enable/disable agents
agents:
  code:
    enabled: true
  mail:
    enabled: false

# Change model assignments
agents:
  code:
    models:
      planner: custom-planner-model
      coder: custom-coder-model
      reviewer: custom-reviewer-model

# Adjust logging
logging:
  level: debug  # trace, debug, info, warn, error
  file: /custom/path/bodhya.log
```

---

## Using CodeAgent

The CodeAgent specializes in generating high-quality code using a BDD/TDD workflow.

### Basic Usage

```bash
bodhya run --domain code --task "DESCRIPTION"
```

### Examples

**1. Simple Function:**
```bash
bodhya run --domain code --task "Create a function to check if a number is prime"
```

**2. Data Structure:**
```bash
bodhya run --domain code --task "Implement a binary search tree with insert, search, and delete operations"
```

**3. Algorithm:**
```bash
bodhya run --domain code --task "Create a merge sort implementation with generics"
```

**4. File Processing:**
```bash
bodhya run --domain code --task "Write a CSV parser that reads files and returns structured data"
```

### CodeAgent Workflow

1. **Planning Phase**
   - Analyzes your task description
   - Creates implementation plan
   - Identifies components and edge cases

2. **BDD Phase**
   - Generates Gherkin feature files
   - Defines acceptance criteria
   - Documents expected behavior

3. **TDD Phase (RED)**
   - Writes failing unit tests
   - Covers all requirements
   - Tests edge cases

4. **Implementation Phase (GREEN)**
   - Writes minimal code to pass tests
   - Follows best practices
   - Maintains clean architecture

5. **Review Phase (REFACTOR)**
   - Reviews code quality
   - Suggests improvements
   - Checks style and patterns

6. **Validation Phase**
   - Runs `cargo check`
   - Runs `cargo test`
   - Runs `cargo clippy`
   - Reports results

### Understanding the Output

CodeAgent produces:

**1. Plan** (`plan.md`):
```markdown
## Purpose
Create a function to check if a number is prime

## Approach
- Implement trial division algorithm
- Handle edge cases (0, 1, 2, negatives)
- Optimize for small numbers

## Components
- `is_prime(n: i64) -> bool`
```

**2. Gherkin Feature** (`feature.feature`):
```gherkin
Feature: Prime Number Checker
  Scenario: Check prime numbers
    Given a number 7
    When we check if it is prime
    Then the result should be true
```

**3. Test Code** (`tests.rs`):
```rust
#[test]
fn test_is_prime_7() {
    assert!(is_prime(7));
}
```

**4. Implementation** (`lib.rs` or `main.rs`):
```rust
pub fn is_prime(n: i64) -> bool {
    if n <= 1 { return false; }
    if n == 2 { return true; }
    // ... implementation
}
```

**5. Review** (`review.md`):
```markdown
## Strengths
- Clean implementation
- Good test coverage

## Suggestions
- Add documentation comments
- Consider optimizations for large numbers
```

---

## Using MailAgent

The MailAgent helps you draft and refine professional emails.

### Basic Usage

```bash
bodhya run --domain mail --task "DESCRIPTION"
```

### Examples

**1. Customer Response:**
```bash
bodhya run --domain mail --task "Write a follow-up email to a customer asking about their experience with our product"
```

**2. Team Communication:**
```bash
bodhya run --domain mail --task "Draft an email introducing a new team member to the department"
```

**3. Business Inquiry:**
```bash
bodhya run --domain mail --task "Compose an email to a potential vendor requesting pricing information"
```

**4. Thank You Note:**
```bash
bodhya run --domain mail --task "Write a thank you email after a job interview"
```

### MailAgent Workflow

1. **Drafting Phase**
   - Analyzes your request
   - Generates initial email draft
   - Includes subject, greeting, body, signature

2. **Refinement Phase**
   - Improves tone and clarity
   - Adjusts formality level
   - Enhances conciseness
   - Checks completeness

3. **Policy Check** (if configured)
   - Validates against company policies
   - Checks for sensitive information
   - Ensures compliance

### Understanding the Output

MailAgent produces:

**Draft Email:**
```
Subject: Follow-up on Product Experience

Dear [Customer Name],

I hope this email finds you well. I wanted to reach out and see how
your experience has been with [Product Name] so far.

Your feedback is valuable to us and helps us continue improving our
products and services.

If you have a few moments, I would greatly appreciate hearing your
thoughts on:
- Overall satisfaction
- Any challenges you've encountered
- Features you find most useful

Thank you for your time and for being a valued customer.

Best regards,
[Your Name]
```

**Refinement Notes:**
```markdown
## Changes Made
- Improved opening for warmth
- Added specific feedback points
- Enhanced closing for professionalism

## Tone: Professional and friendly
## Length: Appropriate for business email
```

---

## Model Management

### Listing Available Models

```bash
bodhya models list
```

Output:
```
Available Models:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ID                          Role     Size    Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
qwen2.5-coder-7b-instruct  Coder    4.2GB   Not installed
qwen2.5-7b-instruct        Planner  4.0GB   Not installed
mistral-7b-v0.3            Writer   4.5GB   Not installed
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Installing Models

```bash
# Install a specific model
bodhya models install qwen2.5-coder-7b-instruct
```

Bodhya will:
1. Show model size and source
2. Display checksum for verification
3. Ask for confirmation
4. Download with progress tracking
5. Verify checksum
6. Store in `~/.bodhya/models/`

### Removing Models

```bash
# Remove a model you no longer need
bodhya models remove qwen2.5-coder-7b-instruct
```

### On-Demand Downloads

When you run a task that requires a model that isn't installed, Bodhya will:
1. Detect the missing model
2. Show the model information
3. Prompt for installation
4. Proceed with the task after installation

---

## Task History

### Viewing Recent Tasks

```bash
# Show last 10 tasks
bodhya history show --limit 10

# Show last 20 tasks
bodhya history show --limit 20

# Show all tasks
bodhya history show
```

Output:
```
Recent Tasks:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 ID    Domain  Description              Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 123   code    Create prime checker     ✓ Done
 122   mail    Customer follow-up       ✓ Done
 121   code    Binary search tree       ✓ Done
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Domain Statistics

```bash
# Statistics for CodeAgent
bodhya history stats code

# Statistics for MailAgent
bodhya history stats mail
```

Output:
```
Domain Statistics: code
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total Tasks:        42
Successful:         38 (90.5%)
Failed:             4 (9.5%)
Average Duration:   45.2s
Average Quality:    87.3/100
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## API Server

### Starting the Server

```bash
# Default (127.0.0.1:3000)
bodhya serve

# Custom host and port
bodhya serve --host 0.0.0.0 --port 8080
```

### API Endpoints

**1. Submit Task:**
```bash
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "domain": "code",
    "description": "Create a hello world function"
  }'
```

Response:
```json
{
  "task_id": "task_abc123",
  "status": "pending",
  "created_at": "2025-11-16T10:30:00Z"
}
```

**2. Get Task Status:**
```bash
curl http://localhost:3000/tasks/task_abc123
```

**3. Get Task Result:**
```bash
curl http://localhost:3000/tasks/task_abc123/result
```

**4. List Agents:**
```bash
curl http://localhost:3000/agents
```

**5. Health Check:**
```bash
curl http://localhost:3000/health
```

### WebSocket Streaming

Connect to `ws://localhost:3000/ws/tasks/{task_id}` for real-time updates:

```javascript
const ws = new WebSocket('ws://localhost:3000/ws/tasks/task_abc123');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Update:', message);
};
```

See `crates/api-server/README.md` for complete API documentation.

---

## Advanced Usage

### Custom Prompts

Bodhya uses prompt templates that you can customize. Templates are embedded in the binaries but can be overridden by creating files in `~/.bodhya/prompts/`:

```
~/.bodhya/prompts/
├── code/
│   ├── planner.txt
│   ├── bdd.txt
│   ├── tdd.txt
│   ├── coder.txt
│   └── reviewer.txt
└── mail/
    ├── draft.txt
    └── refine.txt
```

### Environment Variables

```bash
# Override config location
export BODHYA_CONFIG=~/custom/path/config.yaml

# Override models directory
export BODHYA_MODELS_DIR=~/custom/models

# Set log level
export BODHYA_LOG_LEVEL=debug
```

### Batch Processing

Process multiple tasks:

```bash
# Create a tasks file
cat > tasks.txt <<EOF
code: Create fibonacci function
code: Implement binary search
mail: Write welcome email
EOF

# Process each task
while IFS=: read -r domain task; do
  bodhya run --domain "$domain" --task "$task"
done < tasks.txt
```

### Integration with CI/CD

```yaml
# .github/workflows/bodhya.yml
name: Generate Code with Bodhya

on: [push]

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Bodhya
        run: curl -sSL https://raw.githubusercontent.com/vijayabose/bodhya/main/scripts/install.sh | bash

      - name: Generate Code
        run: |
          bodhya init --profile code
          bodhya run --domain code --task "Create utility functions"

      - name: Run Tests
        run: cargo test
```

---

## Troubleshooting

### Common Issues

**1. Model Download Fails**

Problem: Network error or checksum mismatch

Solution:
```bash
# Remove partial download
rm -rf ~/.bodhya/models/*.tmp

# Retry installation
bodhya models install qwen2.5-coder-7b-instruct
```

**2. Configuration Not Found**

Problem: `Config file not found`

Solution:
```bash
# Reinitialize
bodhya init --profile full
```

**3. Agent Not Responding**

Problem: Task hangs or times out

Solution:
```bash
# Check logs
tail -f ~/.bodhya/logs/bodhya.log

# Verify model is installed
bodhya models list

# Check system resources
htop  # Ensure sufficient RAM/CPU
```

**4. Poor Code Quality**

Problem: Generated code doesn't meet standards

Solution:
- Use more detailed task descriptions
- Specify requirements explicitly
- Review and adjust prompts in `~/.bodhya/prompts/`
- Check model quality scores with evaluation harness

**5. Permission Denied**

Problem: Cannot write to `~/.bodhya/`

Solution:
```bash
# Fix permissions
chmod -R u+w ~/.bodhya/

# Or change location
export BODHYA_CONFIG=~/writable/path/config.yaml
```

### Debug Mode

Enable detailed logging:

```bash
# Set log level to debug
export BODHYA_LOG_LEVEL=debug

# Or edit config.yaml
logging:
  level: debug

# View logs in real-time
tail -f ~/.bodhya/logs/bodhya.log
```

### Getting Help

1. Check logs: `~/.bodhya/logs/bodhya.log`
2. Verify configuration: `cat ~/.bodhya/config/default.yaml`
3. Check model status: `bodhya models list`
4. Run quality gates: `./scripts/check_all.sh` (if built from source)
5. Open an issue: https://github.com/vijayabose/bodhya/issues

---

## FAQ

**Q: Is my data sent to external servers?**
A: No. Bodhya runs entirely locally on your machine. No data is transmitted to external servers unless you explicitly configure remote model endpoints (future feature).

**Q: How much disk space do I need?**
A: Models range from 4-8 GB each. Plan for 10-50 GB depending on how many models you install.

**Q: Can I use my own models?**
A: Yes! Edit `~/.bodhya/models.yaml` to add custom model paths. See DEVELOPER_GUIDE.md for details.

**Q: How do I update Bodhya?**
A: Run the installer again or rebuild from source: `git pull && cargo build --release`

**Q: Can I run Bodhya in Docker?**
A: Yes! See `examples/docker/` for a Dockerfile.

**Q: What languages does CodeAgent support?**
A: Currently optimized for Rust. Support for other languages is planned.

**Q: How do I backup my configuration?**
A: Copy `~/.bodhya/` directory: `cp -r ~/.bodhya ~/bodhya-backup`

**Q: Can I use Bodhya offline?**
A: Yes! Once models are downloaded, Bodhya works entirely offline.

**Q: How do I contribute?**
A: See DEVELOPER_GUIDE.md and CONTRIBUTING.md for guidelines.

**Q: What's the difference between profiles?**
A: Profiles pre-configure which agents and models are enabled:
- `code`: CodeAgent only
- `mail`: MailAgent only
- `full`: All agents

---

## Next Steps

- Read the [Developer Guide](DEVELOPER_GUIDE.md) to create custom agents
- Explore the [API Documentation](crates/api-server/README.md)
- Check out [examples/](examples/) for sample projects
- Join the community at https://github.com/vijayabose/bodhya/discussions

---

**Happy building with Bodhya!** 🚀
