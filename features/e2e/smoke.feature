Feature: Desktop smoke test
  Source: roadmap M4-12; runs against the debug build via WebdriverIO and tauri-driver (`npm run test:e2e`)

  @e2e @REQ-2.1 @desktop
  Scenario: Run a short pomodoro and see it in the statistics
    Given the app is started with a short pomodoro
    When I add the todo "E2E Bericht +work"
    And I start a pomodoro for the todo "E2E Bericht"
    And the pomodoro has ended
    Then the statistics show 1 pomodoro
