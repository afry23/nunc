@ui @smoke
Feature: Frontend BDD runner smoke test
  Verifies that quickpickle, Testing Library and the mocked Tauri IPC work together.

  Scenario: App shell renders
    When I open the app
    Then I see the heading "nunc"

  Scenario: Backend call goes through the typed API
    Given the backend reports version "9.9.9"
    When I ask the backend for the app info
    Then the app info version is "9.9.9"
