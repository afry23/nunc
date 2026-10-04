Feature: Keyboard shortcuts
  Source: REQUIREMENTS 9; desktop only

  @ui @REQ-9 @desktop
  Scenario: Space starts, pauses and resumes the timer
    When I open the app
    And I press the key "Space"
    Then the backend received "start_timer" with:
      """
      { "mode": "pomodoro" }
      """
    When the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 1400      | 1500  |
    And I press the key "Space"
    Then the backend received "pause_timer"
    When the backend reports the timer as:
      | state  | mode     | sessionType | remaining | total |
      | paused | pomodoro | work        | 1400      | 1500  |
    And I press the key "Space"
    Then the backend received "resume_timer"

  @ui @REQ-9 @desktop
  Scenario: Escape stops the timer
    Given the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 1400      | 1500  |
    When I open the app
    And I press the key "Escape"
    Then the backend received "stop_timer"

  @ui @REQ-9 @desktop
  Scenario: N focuses the new todo input
    When I open the app
    And I press the key "n"
    Then the field "Neue Aufgabe" has the focus

  @ui @REQ-9 @desktop
  Scenario: 1, 2 and 3 switch the views
    When I open the app
    And I press the key "2"
    Then I see the heading "Aufgaben"
    When I press the key "3"
    Then I see the heading "Statistik"
    When I press the key "1"
    Then the ring shows "25:00"

  @ui @REQ-9 @desktop
  Scenario: Comma opens the settings
    When I open the app
    And I press the key ","
    Then I see the heading "Einstellungen"

  @ui @REQ-9 @desktop
  Scenario: Shortcuts are ignored while typing in an input
    When I open the app at "/todos"
    And I press the key "2" in the field "Neue Aufgabe"
    And I press the key "Space" in the field "Neue Aufgabe"
    Then I see the heading "Aufgaben"
    And the backend did not receive "start_timer"
