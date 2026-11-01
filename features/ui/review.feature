Feature: Review in the times view
  Source: ADR 0025
  The view "Zeiten" switches to a review of a week, month or quarter with the comparison to the period before.

  @ui @US-RV01
  Scenario: The review shows focus time, comparison and sessions
    Given the review is:
      """
      { "focusMin": 85, "previousFocusMin": 40, "completedSessions": 2,
        "abortedSessions": 1, "abortRatePercent": 33 }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    Then the backend received "get_review"
    And I see "1h 25m"
    And I see "+45m gegenüber der Vorperiode (40m)"
    And I see "2 abgeschlossen, 1 abgebrochen (33 % Abbrüche)"

  @ui @US-RV01
  Scenario: Projects, contexts and completed tasks are listed
    Given the review is:
      """
      { "projects": [{ "group": "+work", "minutes": 85 }, { "group": null, "minutes": 20 }],
        "contexts": [{ "group": "@desk", "minutes": 60 }],
        "completedTodos": [{ "completedAt": "2026-03-04", "text": "Plan trip +home" }] }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "+work"
    And I see "ohne Zuordnung"
    And I see "@desk"
    And I see "Plan trip +home"

  @ui @US-RV01
  Scenario: Without completed tasks the list says so
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "Keine Aufgaben erledigt."

  @ui @US-RV01
  Scenario: The period can be switched to a month and moved back
    When I open the app at "/times"
    And I click "Rückblick"
    And I click "Monat"
    Then the review was requested for a whole month
    When I click "Voriger Zeitraum"
    Then the review was requested for a whole month
    And the review was requested 2 times for different ranges

  @ui @US-RV01
  Scenario: Saving the review writes a Markdown file
    Given the review is saved to "/tmp/nunc-review.md"
    When I open the app at "/times"
    And I click "Rückblick"
    And I click "Als Markdown exportieren"
    Then the backend received "save_review_markdown"
    And I see "Rückblick gespeichert: /tmp/nunc-review.md"

  @ui @US-RV01
  Scenario: The review shows pomodoros, techniques, tasks and days
    Given the review is:
      """
      { "pomodoroCount": 3,
        "modes": [{ "mode": "pomodoro", "sessions": 3, "minutes": 75 },
                  { "mode": "flowtime", "sessions": 1, "minutes": 40 },
                  { "mode": "stopwatch", "sessions": 2, "minutes": 90 }],
        "tasks": [{ "todoId": 1, "text": "Write report +work", "minutes": 100, "pomodoros": 3, "sessions": 4 }],
        "days": [{ "date": "2026-03-02", "minutes": 65, "pomodoros": 2 }] }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "3 Sessions, 1h 15m"
    And I see "1 Sessions, 40m"
    And I see "2 Sessions, 1h 30m"
    And I see "Write report +work"
    And I see "1h 40m · 3 Pomodoros · 4 Sessions"
    And I see "1h 5m · 2 Pomodoros"

  @ui @US-RV01
  Scenario: Projects and contexts are shown as they occurred together
    Given the review is:
      """
      { "combinations": [
          { "projects": ["+foo"], "contexts": ["@meeting", "@work"], "sessions": 2, "minutes": 60 },
          { "projects": ["+foo"], "contexts": ["@work"], "sessions": 1, "minutes": 20 },
          { "projects": [], "contexts": [], "sessions": 1, "minutes": 5 }] }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "+foo @meeting @work"
    And I see "1h · 2 Sessions"
    And I see "+foo @work"
    And I see "20m · 1 Sessions"
    And I see "ohne Zuordnung"
