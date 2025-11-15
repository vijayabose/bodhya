# BODHYA – Gherkin Feature Specifications

## 1. Global Multi-Agent Behavior

```gherkin
Feature: Central controller orchestrates domain agents
  As a user
  I want Bodhya to route my task to the right agent
  So that code tasks, email tasks, and future tasks are handled by specialists

  Background:
    Given Bodhya is installed and initialized
    And the active agents are "Code" and "Mail"
    And local models are configured for each agent
    And remote model engagement is set to "minimum"

  @routing
  Scenario: Route a coding task to the Code agent
    Given I request "Generate a Rust struct and its unit tests"
    When Bodhya receives the task
    Then the central controller selects the Code agent
    And the Code agent executes its internal workflow
    And the output contains Rust code and tests

  @routing
  Scenario: Route a mail task to the Mail agent
    Given I request "Write a polite follow-up email about a delayed invoice"
    When Bodhya receives the task
    Then the central controller selects the Mail agent
    And the Mail agent generates a draft email

  @modularity
  Scenario: Dynamically enable or disable an agent
    Given the Mail agent is disabled in configuration
    When I request an email drafting task
    Then Bodhya returns a clear error indicating the agent is unavailable

  @extensibility
  Scenario: Add a new domain agent without changing core logic
    Given a new "Summarization" agent is installed and configured
    When I request "Summarize this document"
    Then the central controller routes the task to the Summarization agent
    And no changes were required to existing agent code
```

## 2. Installation & Model Management

```gherkin
Feature: Single installer and on-demand model downloads
  As a new user
  I want Bodhya to install cleanly and fetch models when needed
  So that I can start quickly without manual dependency hell

  @installer
  Scenario: Run Bodhya installer
    Given I run the Bodhya installer
    When the installation completes
    Then the "bodhya" binary is available on my PATH
    And the base config and folder structure are created
    And the models directory exists but may be empty

  @init
  Scenario: Initialize Bodhya with a profile
    Given the "bodhya" binary is installed
    When I run "bodhya init"
    Then I can choose between "code", "mail", or "full" profiles
    And a config file is generated for the selected profile

  @model_download
  Scenario: Download a missing local model on demand
    Given I run a code generation task
    And the required planner model is not installed
    When Bodhya detects the missing model
    Then it shows the estimated size and source
    And it asks for confirmation to download
    And after confirmation, it downloads and verifies the model
    And it retries the task using the newly installed model
```

## 3. Code Generation Agent (Local-first, Multi-model)

```gherkin
Feature: Code agent uses multiple local models and sub-agents
  As a developer
  I want a code agent with planners, generators, and reviewers
  So that generated code is robust, tested, and refined locally

  Background:
    Given the Code agent is enabled
    And the local models for "planner", "coder", and "reviewer" are configured
    And remote engagement is "minimum"

  @code_bdd_tdd
  Scenario: Generate a Rust module using BDD/TDD
    Given I describe a Rust config loader
    When the Code agent runs
    Then it generates Gherkin features from the description
    And it generates failing tests first (RED)
    And it generates code to make tests pass (GREEN)
    And it refactors while keeping tests passing
    And it reports test coverage and quality metrics

  @code_multimodel
  Scenario: Use different local models for planning and code generation
    Given the planner model is configured in models.yaml
    And the coder model is configured in models.yaml
    When the Code agent handles a non-trivial task
    Then the planner model is used for decomposition and BDD
    And the coder model is used for implementation
    And a reviewer model is used to critique and improve the result

  @local_checking
  Scenario: Improve results using local checkers
    Given the Code agent has run once
    And the reviewer finds potential improvements
    When the refiner sub-agent runs
    Then the code quality score increases
    And tests remain passing
```

## 4. Mail Agent (Local-first, Tone & Policy)

```gherkin
Feature: Mail agent for email drafting and refinement
  As a user
  I want Bodhya to write and refine emails using local models
  So that I can communicate clearly without exposing data remotely

  Background:
    Given the Mail agent is enabled
    And a local model specialized in writing is configured

  @mail_draft
  Scenario: Draft a polite follow-up email
    Given I provide a brief description of the context
    When the Mail agent generates an email
    Then the result follows a polite, professional tone
    And it is concise and clear

  @mail_review
  Scenario: Improve clarity and tone
    Given I provide an existing email draft
    When the Mail agent review sub-agent runs
    Then it suggests improvements for clarity, tone, and brevity
```

## 5. Configurable Remote Model Engagement (Design)

```gherkin
Feature: Configurable remote model engagement (design)
  As an architect
  I want Bodhya to optionally use remote models
  So that harder tasks can leverage external capacity when allowed

  Background:
    Given remote model connectors are configured but disabled by default

  @remote_min
  Scenario: Minimum engagement mode
    Given remote_engagement = "minimum"
    When a complex task is requested
    Then Bodhya must still prefer local models
    And only log where remote escalation might be beneficial

  @remote_max
  Scenario: Maximum engagement mode (future behavior)
    Given remote_engagement = "maximum"
    When a complex task is requested
    Then Bodhya may call remote models for planning and refinement
    And still run local checkers before returning the final result
```

## 6. Vertical Slice Development Use Case

```gherkin
Feature: Thin vertical slice for initial implementation
  As a developer
  I want a minimal end-to-end slice
  So that I can validate architecture and wiring early

  @slice_v1
  Scenario: Minimal CLI to CodeAgent pipeline
    Given the CLI is available
    And the controller and Code agent are wired
    When I run a simple "hello" code-generation task
    Then the controller routes the task to the Code agent
    And the Code agent returns a static placeholder implementation
    And the program exits successfully
```
