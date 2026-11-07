Feature: Active session
  Source: docs/ui-umbau-plan.md E2, ADR 0031
  A running session is shown in a compact bar in every main area. The big timer is the focus view; free activities
  (meetings, support) start without a task.

  @ui @US-UM02
  Scenario: A running session shows a compact bar with the task and the time
    Given the todos:
      | rawText      | completed |
      | Write report | false     |
    And the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total | todoId |
      | running | pomodoro | work        | 754       | 1500  | 1      |
    When I open the app
    Then I see the active session bar
    And I see "Write report"
    And I see "12:34"

  @ui @US-UM02
  Scenario: The bar is visible in the other main areas too
    Given the backend reports the timer as:
      | state   | mode      | sessionType | remaining | total | note    |
      | running | stopwatch | work        | 1394      | 0     | Meeting |
    When I open the app at "/tasks"
    Then I see the active session bar
    And I see "Meeting"
    And I see "23:14"

  @ui @US-UM02
  Scenario: There is no bar without a session
    When I open the app
    Then I do not see the active session bar

  @ui @US-UM02
  Scenario: The focus view does not repeat the bar
    Given the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 754       | 1500  |
    When I open the app at "/focus"
    Then I do not see the active session bar

  @ui @US-UM02
  Scenario: The bar pauses, resumes and stops the session
    Given the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 754       | 1500  |
    When I open the app
    And I click "Pause"
    Then the backend received "pause_timer"
    When the backend reports the timer as:
      | state  | mode     | sessionType | remaining | total |
      | paused | pomodoro | work        | 700       | 1500  |
    And I click "Fortsetzen"
    Then the backend received "resume_timer"
    When I click "Stop"
    Then the backend received "stop_timer"

  @ui @US-UM02
  Scenario: Tapping the bar opens the focus view
    Given the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total | note    |
      | running | pomodoro | work        | 754       | 1500  | Meeting |
    When I open the app
    And I click "Meeting"
    Then I see the heading "Timer"
    And the ring shows "12:34"

  @ui @US-UM02
  Scenario: A break is shown as a break
    Given the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | short_break | 240       | 300   |
    When I open the app
    Then I see the active session bar
    And I see "Kurze Pause"

  @ui @US-UM02
  Scenario: A task is started from the day plan
    Given the todos:
      | rawText                   |
      | Bericht plan:2026-03-04   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned | 60       | 0     |
    When I open the app
    And I click "Starten: Bericht"
    And I click "Start"
    Then the backend received "start_timer" with:
      """
      { "mode": "pomodoro", "todoId": 1 }
      """

  @ui @US-UM02
  Scenario: A free activity is started without a task
    Given the setting "activeMode" is "stopwatch"
    When I open the app
    And I click "Freie Tätigkeit starten"
    Then I see a dialog with the text "Tätigkeit"
    When I enter "Architekturabstimmung" into the field "Tätigkeit"
    And I enter "+Produkt" into the field "Projekt hinzufügen"
    And I press the key "Enter" in the field "Projekt hinzufügen"
    And I enter "@meeting" into the field "Kontext hinzufügen"
    And I press the key "Enter" in the field "Kontext hinzufügen"
    And I click "Starten"
    Then the backend received "start_timer" with:
      """
      { "mode": "stopwatch", "todoId": null, "labels": { "projectTags": ["+Produkt"], "contextTags": ["@meeting"], "note": "Architekturabstimmung" } }
      """
    And I see the heading "Timer"

  @ui @US-UM02
  Scenario: The mode of a free activity can be chosen
    When I open the app
    And I click "Freie Tätigkeit starten"
    And I click "Stoppuhr"
    And I click "Starten"
    Then the backend received "start_timer" with:
      """
      { "mode": "stopwatch", "todoId": null }
      """

  @ui @US-UM02
  Scenario: A free activity can be cancelled
    When I open the app
    And I click "Freie Tätigkeit starten"
    And I click "Abbrechen"
    Then I do not see a dialog
    And the backend did not receive "start_timer"

  @ui @US-UM02
  Scenario: The empty day offers to start a free activity
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app
    Then I see the buttons "Freie Tätigkeit starten"
