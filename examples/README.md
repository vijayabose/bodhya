# Bodhya Examples

This directory contains example tasks and use cases for Bodhya agents.

## Directory Structure

```
examples/
├── code_tasks/          # Code generation examples
├── mail_tasks/          # Email writing examples
└── README.md            # This file
```

## Code Generation Examples

### Running Code Task Examples

```bash
# Example 1: Simple function
bodhya run --domain code --task "$(cat examples/code_tasks/fibonacci.txt)"

# Example 2: Data structure
bodhya run --domain code --task "$(cat examples/code_tasks/binary_tree.txt)"

# Example 3: Algorithm
bodhya run --domain code --task "$(cat examples/code_tasks/quicksort.txt)"
```

## Email Writing Examples

### Running Email Task Examples

```bash
# Example 1: Customer response
bodhya run --domain mail --task "$(cat examples/mail_tasks/customer_followup.txt)"

# Example 2: Team introduction
bodhya run --domain mail --task "$(cat examples/mail_tasks/team_intro.txt)"

# Example 3: Business inquiry
bodhya run --domain mail --task "$(cat examples/mail_tasks/vendor_inquiry.txt)"
```

## Batch Processing

Process multiple tasks at once:

```bash
# Process all code tasks
for task in examples/code_tasks/*.txt; do
    echo "Processing: $task"
    bodhya run --domain code --task "$(cat $task)"
done

# Process all email tasks
for task in examples/mail_tasks/*.txt; do
    echo "Processing: $task"
    bodhya run --domain mail --task "$(cat $task)"
done
```

## Creating Your Own Examples

1. Create a `.txt` file in the appropriate directory
2. Write a clear, detailed task description
3. Run with Bodhya using the command above

## Task Description Best Practices

### For Code Tasks

- Be specific about requirements
- Mention edge cases to handle
- Specify expected input/output types
- Include performance considerations if relevant

Example:
```
Create a function to calculate the nth Fibonacci number.

Requirements:
- Use dynamic programming for efficiency
- Handle negative inputs (return error)
- Support values up to i64::MAX
- Include comprehensive unit tests
- Add documentation comments
```

### For Email Tasks

- Specify the context and purpose
- Mention the tone (formal, casual, friendly, etc.)
- Include recipient information
- Note any specific points to cover

Example:
```
Write a follow-up email to a customer who recently purchased our product.

Context:
- Customer purchased "Pro Plan" 2 weeks ago
- We want to check their satisfaction
- Encourage feedback

Tone: Professional but friendly
Include: Thank you, request for feedback, offer help
```

## API Integration Examples

See `api_examples/` directory for programmatic usage examples in various languages.

## Contributing

Have a great example? Feel free to contribute:

1. Create a new `.txt` file with your task description
2. Test it with Bodhya
3. Submit a pull request

Make sure your example:
- Has a clear, descriptive filename
- Includes a good task description
- Produces useful output
- Demonstrates a real-world use case
