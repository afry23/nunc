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
