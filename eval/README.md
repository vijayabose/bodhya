# Bodhya Evaluation Harnesses

This directory contains repeatable quality evaluation harnesses for Bodhya's AI agents.

## Overview

Evaluation harnesses provide:
- **Standard test cases** for consistent quality assessment
- **Automated scoring** based on objective metrics
- **Comparison framework** to track improvements over time
- **Regression prevention** to ensure changes don't degrade quality

## Harnesses

### CodeAgent Evaluation (`code_agent/`)

Evaluates code generation quality using:
- **Correctness** (0-40 points): Requirements compliance, patterns, forbidden code
- **Style** (0-30 points): Idiomatic code, documentation, formatting
- **Coverage** (0-30 points): Error handling, tests, edge cases

**Target**: ≥85/100 average score

**Test Cases**:
1. Hello World (Easy) - Basic program structure
2. Fibonacci Calculator (Medium) - Functions with error handling
3. File Reader (Medium) - I/O operations and errors
4. Calculator (Medium) - Enums, pattern matching, tests
5. HashMap Word Counter (Hard) - Complex logic with comprehensive tests

### MailAgent Evaluation (`mail_agent/`)

Evaluates email drafting quality using:
- **Tone** (0-1.5 stars): Appropriate formality, politeness, forbidden words
- **Clarity** (0-1.5 stars): Structure, sentence length, transitions
- **Length** (0-1.0 stars): Word count appropriateness
- **Completeness** (0-1.0 stars): Subject, greeting, closing

**Target**: ≥4.5/5.0 average rating

**Test Cases**:
1. Team Introduction (Professional) - Self-introduction to new team
2. Customer Inquiry Response (Customer) - Helpful product inquiry response
3. Meeting Request (Formal) - Performance review scheduling
4. Project Update (Professional) - Status update with next steps
5. Service Apology (Customer) - Issue acknowledgment and resolution

## Running Evaluations

### CodeAgent Evaluation

```bash
# From workspace root
cargo run --bin bodhya-eval-code-agent

# Or build and run
cargo build --release --bin bodhya-eval-code-agent
./target/release/bodhya-eval-code-agent
```

### MailAgent Evaluation

```bash
# From workspace root
cargo run --bin bodhya-eval-mail-agent

# Or build and run
cargo build --release --bin bodhya-eval-mail-agent
./target/release/bodhya-eval-mail-agent
```

## Output

Each evaluation produces:
- **Summary statistics**: Total cases, passed/failed, average score
- **Individual results**: Per-test-case scores with detailed feedback
- **Pass/Fail status**: Based on target thresholds
- **Exit code**: 0 if passing, 1 if failing (for CI integration)

### Example Output (CodeAgent)

```
================================================================================
CodeAgent Evaluation Summary
================================================================================

Total Cases: 5
Passed: 4 (80.0%)
Failed: 1 (20.0%)

Average Score: 87.50/100

✓ EVALUATION PASSED (≥85/100)

Individual Results:
--------------------------------------------------------------------------------

[PASS] Hello World Program (hello_world)
  Score: 92.00/100 (C:38.0 S:28.0 Cov:26.0)
  Duration: 145ms
  Feedback:
    ✓ All expected patterns found
    ✓ No forbidden patterns found
    ✓ Length appropriate (8 lines)
    ✓ Contains documentation comments
    ✓ Contains error handling

...
```

## Integration with CI/CD

Add to your CI pipeline:

```yaml
# .github/workflows/quality.yml
- name: Run CodeAgent Evaluation
  run: cargo run --bin bodhya-eval-code-agent

- name: Run MailAgent Evaluation
  run: cargo run --bin bodhya-eval-mail-agent
```

The harnesses exit with code 1 if evaluations fail, causing CI to fail.

## Extending Evaluations

### Adding New Test Cases

**CodeAgent**:
```rust
// In eval/code_agent/src/standard_cases.rs
pub fn my_new_case() -> CodeTestCase {
    CodeTestCase::new(
        "my_case_id",
        "My Case Name",
        "Task description for the agent"
    )
    .with_difficulty(Difficulty::Medium)
    .with_validation(
        ValidationCriteria::new()
            .expect_pattern("fn my_func")
            .forbid_pattern("unsafe")
            .min_lines(10)
            .max_lines(50)
    )
}

// Add to get_standard_cases()
pub fn get_standard_cases() -> Vec<CodeTestCase> {
    vec![
        // existing cases...
        my_new_case(),
    ]
}
```

**MailAgent**:
```rust
// In eval/mail_agent/src/standard_cases.rs
pub fn my_new_case() -> MailTestCase {
    MailTestCase::new(
        "my_case_id",
        "My Case Name",
        "Email context",
        "Email purpose"
    )
    .with_email_type(EmailType::Professional)
    .with_validation(
        EmailValidation::new()
            .require_greeting()
            .require_closing()
            .min_words(50)
            .max_words(150)
            .forbid_word("inappropriate")
    )
}
```

### Customizing Scoring

Modify the scoring logic in:
- `eval/code_agent/src/scorer.rs` - CodeScorer implementation
- `eval/mail_agent/src/scorer.rs` - EmailScorer implementation

## Best Practices

1. **Run evaluations regularly** during development
2. **Track scores over time** to measure improvements
3. **Add test cases** when fixing bugs or adding features
4. **Use in CI** to prevent quality regressions
5. **Adjust thresholds** as agent capabilities improve

## Architecture

```
eval/
├── README.md                    # This file
├── code_agent/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # CLI entry point
│       ├── lib.rs               # Public API
│       ├── test_case.rs         # Test case definitions
│       ├── scorer.rs            # Quality scoring logic
│       ├── standard_cases.rs    # Standard test cases
│       └── runner.rs            # Evaluation runner
└── mail_agent/
    ├── Cargo.toml
    └── src/
        ├── main.rs              # CLI entry point
        ├── lib.rs               # Public API
        ├── test_case.rs         # Test case definitions
        ├── scorer.rs            # Quality rating logic
        ├── standard_cases.rs    # Standard test cases
        └── runner.rs            # Evaluation runner
```

## Dependencies

Both harnesses use:
- `bodhya-core` - Agent trait and types
- `bodhya-agent-{code,mail}` - Agent implementations
- `tokio` - Async runtime
- `serde` - Serialization for results
- `colored` - Terminal output formatting

## Future Enhancements

- [ ] JSON output format for programmatic analysis
- [ ] Historical comparison (track scores across commits)
- [ ] Differential evaluation (compare two agent versions)
- [ ] Custom test case loading from YAML/JSON
- [ ] Model registry integration for real model evaluation
- [ ] Performance benchmarking (latency, token usage)
- [ ] Automated report generation (HTML/PDF)

## Questions?

Refer to the main [CLAUDE.md](../CLAUDE.md) for development guidelines and architecture details.
