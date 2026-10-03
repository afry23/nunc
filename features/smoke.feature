@domain @smoke
Feature: BDD runner smoke test
  Verifies that the cucumber runner finds and executes feature files.

  Scenario: Steps are executed
    Given the number 2
    When I add 3
    Then the result is 5
