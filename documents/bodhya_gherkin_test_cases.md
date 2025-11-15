# BODHYA – Gherkin Test Cases

```gherkin
Feature: Agent selection and model routing

  @unit @controller
  Scenario: controller selects Code agent for code-like tasks
    Given a task with description containing "generate Rust code"
    When select_agent(task) is called
    Then "code" is returned as the agent_id

  @unit @controller
  Scenario: controller selects Mail agent for mail-like tasks
    Given a task with description containing "write an email"
    When select_agent(task) is called
    Then "mail" is returned as the agent_id

  @unit @models
  Scenario: local model registry returns appropriate engine
    Given a role "planner" and domain "code"
    When get_model(role, domain, engagement="minimum") is called
    Then a local engine config is returned
    And the type is "local"

  @unit @plug
  Scenario: dynamically load agents from configuration
    Given an agent config file listing "code" and "mail"
    When Bodhya starts
    Then both agents are registered
    And no code changes were needed to add/remove agents
```

```gherkin
Feature: Code agent internal behavior

  @unit @code_bdd
  Scenario: code agent produces Gherkin from description
    Given a natural language description of a Rust module
    When the Code agent planner runs
    Then a Gherkin feature file is produced

  @unit @code_tdd
  Scenario: code agent creates failing tests first
    Given a Gherkin feature for a Rust module
    When the Code agent TDD sub-agent runs
    Then tests compile but initially fail

  @unit @code_refine
  Scenario: code agent invokes reviewer and refiner
    Given code has been generated once
    When the reviewer sub-agent runs
    Then suggestions for improvement are produced
    And the refiner applies safe changes
```

```gherkin
Feature: Mail agent behavior

  @unit @mail_draft
  Scenario: mail agent generates a draft
    Given a short description of the email purpose
    When the Mail agent draft sub-agent runs
    Then a structured email body is produced

  @unit @mail_refine
  Scenario: mail agent refines an existing draft
    Given an email draft with unclear tone
    When the Mail agent refine sub-agent runs
    Then a clearer and more polite version is produced
```

```gherkin
Feature: Quality and evaluation scripts

  @unit @scripts
  Scenario: check_all script covers required tools
    Given the scripts/check_all.sh file
    When I inspect its commands
    Then it runs cargo fmt in check mode
    And it runs cargo clippy with warnings as errors
    And it runs cargo test for all crates
```

```gherkin
Feature: Installer and model manager

  @unit @installer
  Scenario: installer creates base structure
    Given I run the Bodhya installer in a clean environment
    When the installation completes
    Then the bodhya binary exists
    And the config directory exists
    And the models directory exists

  @unit @models_cli
  Scenario: models list shows installed models
    Given at least one model is installed
    When I run "bodhya models list"
    Then the installed model appears in the output

  @unit @models_download
  Scenario: models install downloads a missing model
    Given the planner model is not installed
    When I run "bodhya models install code_planner"
    Then the model is downloaded
    And the checksum is verified
    And "bodhya models list" shows the planner model
```
