Feature: Mobile smoke test
  Source: roadmap M5-06; the flows in `e2e/mobile/*.yaml` (Maestro, `npm run test:mobile`) implement these scenarios
  on an emulator, simulator or device. No cucumber runner executes this file.

  # e2e/mobile/smoke.yaml
  @e2e @REQ-2.1 @mobile
  Scenario: A todo can be created and a timer started on a phone
    Given the app is installed with a clean state
    When I add the todo "(A) Maestro-Test +e2e"
    And I start a pomodoro
    And I pause and stop the timer
    Then the todo is listed and the timer is idle again

  # e2e/mobile/background-notification.yaml
  @e2e @REQ-13 @mobile
  Scenario: The end of a session in the background is announced
    Given the pomodoro lasts 1 minute and is running
    When the app goes to the background
    And the end time passes
    Then a notification "Pomodoro geschafft" is shown
    And after tapping it the app shows an idle timer
