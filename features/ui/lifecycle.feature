Feature: App lifecycle on mobile
  Source: ADR 0004, REQUIREMENTS 13, roadmap M5-04; the app is suspended in the background

  @ui @REQ-13 @mobile
  Scenario: Returning from the background shows the recalculated timer
    Given the platform is "mobile"
    When I open the app
    And the timer ran in the background and is now:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 754       | 1500  |
    And the app returns to the foreground
    Then the ring shows "12:34"

  @ui @REQ-13 @mobile
  Scenario: Returning from the background also refreshes the todo list
    Given the platform is "mobile"
    When I open the app at "/todos"
    And the todo "Neu im Backend" was added in the backend
    And the app returns to the foreground
    Then the todo list shows "Neu im Backend"

  @ui @REQ-13 @mobile
  Scenario: Going to the background does not reload anything
    Given the platform is "mobile"
    When I open the app at "/todos"
    And the todo "Neu im Backend" was added in the backend
    And the app goes to the background
    Then the todo list is empty
