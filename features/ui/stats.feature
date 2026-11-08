Feature: Statistics view
  Source: REQUIREMENTS 10, 11; ADR 0037
  The statistics show how long was worked on what: tasks, free activities, projects and contexts for a day, week,
  month, quarter or year, with the charts always visible.

  @ui @REQ-10
  Scenario: The period can be switched from day to year
    When I open the app at "/stats"
    Then I see "Tag"
    And I see "Woche"
    And I see "Monat"
    And I see "Quartal"
    And I see "Jahr"
    When I click "Jahr"
    Then the backend received "get_review"
    And the statistics were requested for the period "year"
    When I click "Quartal"
    Then the statistics were requested for the period "month"
    When I click "Tag"
    Then the statistics were requested for the period "day"

  @ui @REQ-10
  Scenario: The charts are visible without opening anything
    Given the review has the days:
      | daysAgo | minutes | pomodoros |
      | 0       | 100     | 4         |
    And the review is:
      """
      { "projects": [{ "group": "+work", "minutes": 100 }] }
      """
    When I open the app at "/stats"
    Then I see the charts "bar, pie"
    And I see "1h 40m"
    And I see "+work"

  @ui @REQ-10
  Scenario: Contexts are listed with their time
    Given the review is:
      """
      { "focusMin": 100, "contexts": [{ "group": "@desk", "minutes": 100 }, { "group": null, "minutes": 20 }] }
      """
    When I open the app at "/stats"
    Then I see "@desk"
    And I see "ohne Projekt"

  @ui @REQ-10
  Scenario: An empty period shows an empty state
    When I open the app at "/stats"
    Then I see "Keine Sitzungen im gewählten Zeitraum."
    And I do not see the charts "bar, pie"

  @ui @REQ-10
  Scenario: The total is compared with the period before
    Given the review is:
      """
      { "focusMin": 85, "previousFocusMin": 40 }
      """
    When I open the app at "/stats"
    Then I see "1h 25m"
    And I see "+45m gegenüber der Vorperiode (40m)"

  @ui @REQ-10
  Scenario: The bars are stacked by technique
    Given the review has the days:
      | daysAgo | minutes | pomodoros | pomodoroMin | flowtimeMin | stopwatchMin |
      | 0       | 160     | 4         | 100         | 40          | 20           |
    When I open the app at "/stats"
    And I click "Tag"
    Then the bar chart stacks the segments "pomodoro, flowtime, stopwatch" on the last day

  @ui @REQ-10
  Scenario: The time of day is shown as hours with work
    Given the review is:
      """
      { "focusMin": 90, "hours": [0,0,0,0,0,0,0,0,0,15,60,15,0,0,0,0,0,0,0,0,0,0,0,0] }
      """
    When I open the app at "/stats"
    Then I see the charts "bar, hours"
    And I see "Tageszeit"

  @ui @US-ST01
  Scenario: Tasks are listed with time and status
    Given the review is:
      """
      { "focusMin": 120, "tasks": [
        { "todoId": 1, "text": "Write report", "minutes": 90, "pomodoros": 2, "sessions": 3,
          "completed": false, "completedAt": null, "projects": ["+work"], "estimateCheck": null },
        { "todoId": 2, "text": "Plan trip", "minutes": 30, "pomodoros": 1, "sessions": 1,
          "completed": true, "completedAt": "2026-03-04", "projects": ["+home"], "estimateCheck": null } ] }
      """
    When I open the app at "/stats"
    Then I see "Write report"
    And I see "1h 30m · 3 Sessions"
    And I see "offen"
    And I see "Plan trip"
    And I see "erledigt am 04.03."

  @ui @US-ST01
  Scenario: Tasks and activities can be filtered by project
    Given the review is:
      """
      { "focusMin": 120, "tasks": [
        { "todoId": 1, "text": "Write report", "minutes": 90, "pomodoros": 2, "sessions": 3,
          "completed": false, "completedAt": null, "projects": ["+work"], "estimateCheck": null },
        { "todoId": 2, "text": "Plan trip", "minutes": 30, "pomodoros": 1, "sessions": 1,
          "completed": false, "completedAt": null, "projects": [], "estimateCheck": null } ],
        "activities": [
        { "sessionId": 1, "date": "2026-03-04", "text": "Daily", "minutes": 30, "mode": "stopwatch",
          "projects": ["+work"], "contexts": [] },
        { "sessionId": 2, "date": "2026-03-04", "text": "Lunch talk", "minutes": 20, "mode": "stopwatch",
          "projects": [], "contexts": [] } ] }
      """
    When I open the app at "/stats"
    And I click "Projekt"
    And I click "+work"
    Then I see "Write report"
    And I see "Daily"
    And I do not see "Plan trip"
    And I do not see "Lunch talk"

  @ui @US-ST01
  Scenario: Free activities are listed one by one with their time
    Given the review is:
      """
      { "focusMin": 50, "activities": [
        { "sessionId": 1, "date": "2026-03-04", "text": "Daily", "minutes": 30, "mode": "stopwatch",
          "projects": ["+foo"], "contexts": ["@meeting"] },
        { "sessionId": 2, "date": "2026-03-05", "text": "Daily", "minutes": 20, "mode": "stopwatch",
          "projects": ["+foo"], "contexts": ["@meeting"] } ] }
      """
    When I open the app at "/stats"
    Then I see "Freie Tätigkeiten"
    And I see "04.03."
    And I see "05.03."
    And I see "30m"
    And I see "20m"

  @ui @REQ-10.4
  Scenario: The streaks and further figures are folded
    Given the review is:
      """
      { "focusMin": 25 }
      """
    And the statistics have a current streak of 3 and a longest streak of 8
    When I open the app at "/stats"
    Then I do not see "Aktuelle Serie"
    When I click "Weitere Kennzahlen"
    Then the statistic "Aktuelle Serie" is "3 Tage"
    And the statistic "Längste Serie" is "8 Tage"

  @ui @REQ-11
  Scenario: The export button offers CSV and JSON
    Given the review is:
      """
      { "focusMin": 25 }
      """
    When I open the app at "/stats"
    And I click "Exportieren"
    Then I see the buttons "CSV, JSON"

  @ui @REQ-11 @desktop
  Scenario: Saving the export writes a file
    Given the review is:
      """
      { "focusMin": 25 }
      """
    And the export is saved to "/tmp/nunc-export.json"
    When I open the app at "/stats"
    And I click "Exportieren"
    And I click "JSON"
    Then the backend received "save_export_to_file" with:
      """
      { "format": "json", "request": { "period": "week" } }
      """
    And I see "Export gespeichert: /tmp/nunc-export.json"

  @ui @US-ST01
  Scenario: Task text is highlighted like in the task list without planning fields
    Given the review is:
      """
      { "focusMin": 30, "tasks": [
        { "todoId": 1, "text": "Write report +work @desk plan:2026-03-04 estp:2", "minutes": 30, "pomodoros": 1,
          "sessions": 1, "completed": false, "completedAt": null, "projects": ["+work"], "estimateCheck": null } ] }
      """
    When I open the app at "/stats"
    Then I see "+work"
    And I see "@desk"
    And I do not see "plan:2026-03-04"
    And I do not see "estp:2"
