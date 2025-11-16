# MailAgent Evaluation Harness

Automated quality evaluation for Bodhya's MailAgent.

## Purpose

Provides repeatable, objective assessment of email drafting quality across:
- **Tone**: Is it appropriately formal/professional?
- **Clarity**: Is it well-structured and easy to read?
- **Length**: Is it concise yet complete?
- **Completeness**: Does it have all required elements?

## Target KPI

**≥4.5/5.0** stars average rating across standard test cases

## Rating System

Total rating: **0-5.0 stars**

### Tone (0-1.5 stars)
- No forbidden words/phrases
- Expected tone markers present
- Type-appropriate formality

### Clarity (0-1.5 stars)
- Paragraph structure
- Readable sentence length
- Flow and transitions
- Appropriate punctuation

### Length (0-1.0 stars)
- Word count within bounds
- Neither too brief nor verbose

### Completeness (0-1.0 stars)
- Subject line present
- Greeting included
- Proper closing

## Standard Test Cases

| ID | Name | Type | Key Requirements |
|----|------|------|------------------|
| `team_intro` | Team Introduction | Professional | 50-200 words, polite |
| `customer_inquiry` | Customer Inquiry Response | Customer | 80-250 words, helpful |
| `meeting_request` | Meeting Request | Formal | 60-180 words, respectful |
| `project_update` | Project Status Update | Professional | 100-300 words, clear |
| `apology` | Service Apology | Customer | 80-220 words, empathetic |

## Running

```bash
# From workspace root
cargo run --bin bodhya-eval-mail-agent

# Or with specific verbosity
RUST_LOG=info cargo run --bin bodhya-eval-mail-agent
```

## Example Output

```
Starting MailAgent Evaluation
Test cases: 5

Running: Team Introduction (team_intro)
  Rating: 4.6/5.0 ★

Running: Customer Inquiry Response (customer_inquiry)
  Rating: 4.8/5.0 ★

...

================================================================================
MailAgent Evaluation Summary
================================================================================

Total Cases: 5
Passed: 5 (100.0%)
Failed: 0 (0.0%)

Average Rating: 4.65/5.0 ★

✓ EVALUATION PASSED (≥4.5/5.0)
```

## Adding Test Cases

```rust
// In src/standard_cases.rs
pub fn my_test_case() -> MailTestCase {
    MailTestCase::new(
        "my_case",
        "My Test Case",
        "Email context description",
        "Email purpose"
    )
    .with_email_type(EmailType::Professional)
    .with_validation(
        EmailValidation::new()
            .require_greeting()
            .require_closing()
            .require_subject()
            .min_words(50)
            .max_words(200)
            .expect_tone("thank")
            .forbid_word("hey")
    )
}
```

## Library Usage

```rust
use bodhya_eval_mail_agent::{
    MailTestCase, EvaluationRunner, get_standard_cases
};
use bodhya_agent_mail::MailAgent;

#[tokio::main]
async fn main() {
    let agent = MailAgent::new();
    let runner = EvaluationRunner::new(agent);

    let cases = get_standard_cases();
    let summary = runner.run_all(&cases).await;

    println!("Average: {:.2} ★", summary.average_rating);
}
```

## CI Integration

```bash
#!/bin/bash
# In your CI script
cargo run --bin bodhya-eval-mail-agent || exit 1
```

Exit code 0 = passed, 1 = failed

## Files

- `src/main.rs` - CLI entry point
- `src/lib.rs` - Public API
- `src/test_case.rs` - Test case types
- `src/scorer.rs` - Rating logic
- `src/standard_cases.rs` - Standard test cases
- `src/runner.rs` - Evaluation orchestration
