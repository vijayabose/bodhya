# CodeAgent Evaluation Harness

Automated quality evaluation for Bodhya's CodeAgent.

## Purpose

Provides repeatable, objective assessment of code generation quality across:
- **Correctness**: Does it meet requirements?
- **Style**: Is it idiomatic and well-documented?
- **Coverage**: Are edge cases and errors handled?

## Target KPI

**≥85/100** average score across standard test cases

## Scoring System

Total score: **0-100 points**

### Correctness (0-40 points)
- Expected patterns present (15 pts)
- No forbidden patterns (10 pts)
- Appropriate length (15 pts)

### Style (0-30 points)
- Proper naming conventions (10 pts)
- Comments/documentation (10 pts)
- Clean formatting (10 pts)

### Coverage (0-30 points)
- Error handling present (15 pts)
- Tests included (15 pts)
- Adjusted by difficulty level

## Standard Test Cases

| ID | Name | Difficulty | Key Requirements |
|----|------|------------|------------------|
| `hello_world` | Hello World Program | Easy | Basic structure, println! |
| `fibonacci` | Fibonacci Calculator | Medium | Functions, Result, tests |
| `file_reader` | File Content Reader | Medium | I/O, error handling, docs |
| `calculator` | Basic Calculator | Medium | Enums, match, tests |
| `hashmap_ops` | HashMap Word Counter | Hard | Complex logic, comprehensive tests |

## Running

```bash
# From workspace root
cargo run --bin bodhya-eval-code-agent

# Or with specific verbosity
RUST_LOG=info cargo run --bin bodhya-eval-code-agent
```

## Example Output

```
Starting CodeAgent Evaluation
Test cases: 5

Running: Hello World Program (hello_world)
  Score: 92.00/100

Running: Fibonacci Calculator (fibonacci)
  Score: 88.50/100

...

================================================================================
CodeAgent Evaluation Summary
================================================================================

Total Cases: 5
Passed: 4 (80.0%)
Failed: 1 (20.0%)

Average Score: 87.50/100

✓ EVALUATION PASSED (≥85/100)
```

## Adding Test Cases

```rust
// In src/standard_cases.rs
pub fn my_test_case() -> CodeTestCase {
    CodeTestCase::new(
        "my_case",
        "My Test Case",
        "Write a function that..."
    )
    .with_difficulty(Difficulty::Medium)
    .with_validation(
        ValidationCriteria::new()
            .must_compile()
            .expect_pattern("fn my_func")
            .expect_pattern("#[test]")
            .forbid_pattern("panic!")
            .min_lines(20)
            .max_lines(100)
    )
}
```

## Library Usage

```rust
use bodhya_eval_code_agent::{
    CodeTestCase, EvaluationRunner, get_standard_cases
};
use bodhya_agent_code::CodeAgent;

#[tokio::main]
async fn main() {
    let agent = CodeAgent::new();
    let runner = EvaluationRunner::new(agent);

    let cases = get_standard_cases();
    let summary = runner.run_all(&cases).await;

    println!("Average: {:.2}", summary.average_score);
}
```

## CI Integration

```bash
#!/bin/bash
# In your CI script
cargo run --bin bodhya-eval-code-agent || exit 1
```

Exit code 0 = passed, 1 = failed

## Files

- `src/main.rs` - CLI entry point
- `src/lib.rs` - Public API
- `src/test_case.rs` - Test case types
- `src/scorer.rs` - Scoring logic
- `src/standard_cases.rs` - Standard test cases
- `src/runner.rs` - Evaluation orchestration
