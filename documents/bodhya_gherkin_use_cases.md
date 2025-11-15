# BODHYA – Gherkin Use Cases

## Use Case 1 – End-to-end Config Loader via Code Agent

```gherkin
Feature: Build a Rust configuration loader using Bodhya
  As a developer
  I want Bodhya to build a modular config loader
  So that applications can load validated settings easily

  Scenario: End-to-end config loader generation
    Given I describe a config loader module
    And the required code models are installed
    When requirements, tests, modules, and implementation are generated
    Then I receive a working Rust module
    And coverage and quality gates pass
```

## Use Case 2 – Professional Email Drafting via Mail Agent

```gherkin
Feature: Use Bodhya to draft a professional email
  As a busy professional
  I want Bodhya to draft and refine my emails
  So that my communication is clear, polite, and efficient

  Scenario: Draft and refine a follow-up email
    Given I provide the context of a delayed invoice
    When the Mail agent generates a first draft
    And the review sub-agent refines tone and clarity
    Then the final email is polite, clear, and concise
```

## Use Case 3 – Future Domain Agent (Summarization)

```gherkin
Feature: Summarization agent usage
  As a user
  I want Bodhya to summarize documents
  So that I can quickly understand large texts

  Scenario: Summarize a long technical document
    Given a Summarization agent is configured and enabled
    And I provide a long technical document
    When I request a summary
    Then the controller routes the task to the Summarization agent
    And a concise, accurate summary is returned
```

## Use Case 4 – Evaluation Harness for CodeAgent

```gherkin
Feature: Evaluate CodeAgent quality
  As a maintainer
  I want a repeatable evaluation harness
  So that I can compare different model and prompt configurations

  Scenario: Run a standard evaluation task set
    Given a set of standard code-generation tasks
    And a baseline configuration for the Code agent
    When I run the evaluation harness
    Then I obtain metrics for correctness, coverage, and quality
    And I can compare them against previous runs
```

## Use Case 5 – Installer and Model Management

```gherkin
Feature: Install and prepare Bodhya for use

  Scenario: Install and initialize Bodhya
    Given I have downloaded the Bodhya installer
    When I run the installer
    And I run "bodhya init"
    Then the config and folders are set up
    And I can choose a profile that preconfigures agents and model roles

  Scenario: On-demand installation of models
    Given I requested a code-generation task without having models installed
    When Bodhya detects missing models for the Code agent
    Then it prompts me to download the required models
    And after I confirm, the download and verification succeeds
    And the task is retried using the newly installed models
```
