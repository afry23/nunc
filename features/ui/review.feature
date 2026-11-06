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
    When I click "Arbeitsweise"
    Then I see "2 abgeschlossen, 1 abgebrochen (33 % Abbrüche)"

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
    Given the todos:
      | rawText           | completed |
      | Write report +work | false     |
    And the review is:
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
    Then I see "Write report +work"
    And I see "1h 40m · 3 Pomodoros · 4 Sessions"
    When I click "Arbeitsweise"
    Then I see "3 Sessions, 1h 15m"
    And I see "1 Sessions, 40m"
    And I see "2 Sessions, 1h 30m"
    And I see "1h 5m · 2 Pomodoros"

  @ui @US-RV01
  Scenario: Projects and contexts are shown as they occurred together
    Given the review is:
      """
      { "combinations": [
          { "projects": ["+foo"], "contexts": ["@meeting", "@work"], "sessions": 2, "minutes": 60,
            "notes": [{ "date": "2026-03-02", "text": "Weekly planning" }] },
          { "projects": ["+foo"], "contexts": ["@work"], "sessions": 1, "minutes": 20, "notes": [] },
          { "projects": [], "contexts": [], "sessions": 1, "minutes": 5, "notes": [] }] }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "+foo @meeting @work"
    And I see "1h · 2 Sessions"
    And I see "+foo @work"
    And I see "20m · 1 Sessions"
    And I see "02.03. Weekly planning"
    And I see "ohne Zuordnung"

  @ui @US-RV02
  Scenario: The week review is written into the review of a week
    When I open the app at "/times"
    And I click "Rückblick"
    And I enter "Good week" into the field "Reflexion"
    And I click "Speichern" in the section "Reflexion"
    Then the backend received "save_journal_note"
    And the backend received "save_journal_note" with "kind" equal to "week_note"
    And the backend received "save_journal_note" with "text" equal to "Good week"
    And a field holds "Good week"

  @ui @US-RV02
  Scenario: Saving is only possible after the text changed
    When I open the app at "/times"
    And I click "Rückblick"
    Then the button "Speichern" in the section "Reflexion" is disabled

  @ui @US-RV02
  Scenario: An achievement can be added, changed and deleted
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "Noch keine Erfolge in diesem Zeitraum."
    When I enter "Released the beta" into the field "Neuer Erfolg"
    And I click "Hinzufügen" in the section "Ergebnisse und Beiträge"
    Then the backend received "add_achievement"
    And I see "Released the beta"
    When I click "Bearbeiten"
    And I enter "Released the beta 1.0" into the field "Text"
    And I click "Speichern" in the section "Ergebnisse und Beiträge"
    Then the backend received "update_achievement"
    And I see "Released the beta 1.0"
    When I click "Löschen"
    Then the backend received "delete_achievement"
    And I see "Noch keine Erfolge in diesem Zeitraum."

  @ui @US-RV02
  Scenario: A completed todo is saved as an achievement from the list
    Given the todos:
      | rawText                   | completed |
      | x 2026-03-03 Report +work | true      |
      | Open task                 | false     |
    When I open the app at "/todos"
    And I click "Erledigte anzeigen (1)"
    And I click "Aktionen: Report"
    And I click "Als Erfolg merken: Report"
    Then the backend received "add_achievement" with "text" equal to "Report"
    And I see "Als Erfolg im Rückblick gemerkt."

  @ui @US-RV02
  Scenario: An open todo cannot be saved as an achievement
    Given the todos:
      | rawText   |
      | Open task |
    When I open the app at "/todos"
    And I click "Aktionen: Open task"
    Then I do not see the menu item "Als Erfolg merken: Open task"

  @ui @US-RV02
  Scenario: Adding an achievement needs a text
    When I open the app at "/times"
    And I click "Rückblick"
    Then the button "Hinzufügen" in the section "Ergebnisse und Beiträge" is disabled

  @ui @US-RV02
  Scenario: Day notes are listed and can be added and emptied
    Given the review is:
      """
      { "journal": { "achievements": [], "weekNotes": [],
        "dayNotes": [{ "id": "note:day:2026-03-02", "date": "2026-03-02", "text": "Calm day" }] } }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    Then a field holds "Calm day"
    When I enter "" into the field "Notizen aus den Tagen 02.03."
    And I click "Speichern" in the section "Notizen aus den Tagen"
    Then the backend received "save_journal_note" with "kind" equal to "day_note"
    And the backend received "save_journal_note" with "text" equal to ""
    And I see "Noch keine Tagesnotizen in diesem Zeitraum."
    When I enter "New note" into the field "Neue Tagesnotiz"
    And I click "Hinzufügen" in the section "Notizen aus den Tagen"
    Then a field holds "New note"

  @ui @US-RV02
  Scenario: A month review shows the week reviews of the month read-only
    Given the review is:
      """
      { "journal": { "achievements": [], "dayNotes": [],
        "weekNotes": [{ "id": "note:week:2026-03-08", "date": "2026-03-08", "text": "Good week" }] } }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    And I click "Monat"
    Then I see "Wochenrückblicke"
    And I see "Good week"
    And the field "Reflexion" does not exist


  @ui @US-HB05
  Scenario: The review lists the habits with their quota
    Given the review is:
      """
      { "habits": [
        { "id": "h1", "name": "Lesen", "archived": false, "dueDays": 7, "doneDays": 5, "ratePercent": 71 },
        { "id": "h2", "name": "Sport", "archived": false, "dueDays": 0, "doneDays": 0, "ratePercent": null }
      ] }
      """
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "Lesen"
    And I see "5 von 7 Tagen (71 %)"
    And I see "Sport"
    And I see "0 von 0 Tagen"

  @ui @US-HB05
  Scenario: Without habits the review has no habit section
    When I open the app at "/times"
    And I click "Rückblick"
    Then I see "Erfasste Arbeitszeit"
    And I do not see "Gewohnheiten"

  @ui @US-UM06
  Scenario: The review starts with results and ends with the way of working
    Given the review is:
      """
      { "projects": [{ "group": "+work", "minutes": 85 }],
        "completedTodos": [{ "completedAt": "2026-03-04", "text": "Plan trip +home" }] }
      """
    When I open the app at "/review"
    Then the sections appear in the order "Ergebnisse und Beiträge, Arbeit nach Projekten, Erledigte Aufgaben, Notizen aus den Tagen, Reflexion, Für den nächsten Zeitraum, Arbeitsweise"

  @ui @US-UM06
  Scenario: The total is the tracked work time, the focus time is a part of it
    Given the review is:
      """
      { "focusMin": 150, "previousFocusMin": 100,
        "modes": [{ "mode": "pomodoro", "sessions": 3, "minutes": 60 },
                  { "mode": "flowtime", "sessions": 1, "minutes": 30 },
                  { "mode": "stopwatch", "sessions": 2, "minutes": 60 }] }
      """
    When I open the app at "/review"
    Then I see "Erfasste Arbeitszeit"
    And I see "2h 30m"
    And I see "davon Fokuszeit (Pomodoro und Flowtime): 1h 30m"

  @ui @US-UM06
  Scenario: Tasks that were worked on but are still open are listed apart
    Given the todos:
      | rawText      | completed |
      | Write report | false     |
      | Plan trip    | true      |
    And the review is:
      """
      { "tasks": [{ "todoId": 1, "text": "Write report", "minutes": 90, "pomodoros": 2, "sessions": 3 },
                  { "todoId": 2, "text": "Plan trip", "minutes": 30, "pomodoros": 1, "sessions": 1 }] }
      """
    When I open the app at "/review"
    Then I see "Bearbeitet, aber noch offen"
    And I see "Write report"
    And I do not see "Plan trip"

  @ui @US-UM06
  Scenario: Without open work in progress there is no such section
    When I open the app at "/review"
    Then I do not see "Bearbeitet, aber noch offen"

  @ui @US-UM06
  Scenario: The reflection of a week asks three questions
    When I open the app at "/review"
    Then I see "Was lief gut?"
    And I see "Was war schwierig?"
    And I see "Was nehme ich mit?"

  @ui @US-UM06
  Scenario: The week review can be started from the top
    When I open the app at "/review"
    And I click "Wochenrückblick beginnen"
    Then the field "Reflexion" has the focus

  @ui @US-UM06
  Scenario: The month review has no week review to start
    When I open the app at "/review"
    And I click "Monat"
    Then I do not see the button "Wochenrückblick beginnen"

  @ui @US-UM06
  Scenario: A follow-up task is created for the next period
    When I open the app at "/review"
    And I enter "Bericht nachreichen +work" into the field "Aufgabe für den nächsten Zeitraum"
    And I click "Aufgabe erstellen"
    Then the backend received "add_todo" with:
      """
      { "rawText": "Bericht nachreichen +work" }
      """
    And the field "Aufgabe für den nächsten Zeitraum" is empty

  @ui @US-UM06
  Scenario: An empty follow-up task cannot be created
    When I open the app at "/review"
    And I click "Aufgabe erstellen"
    Then the backend did not receive "add_todo"

  @ui @US-UM06
  Scenario: The way of working is folded and leads to the timer statistics
    Given the review is:
      """
      { "pomodoroCount": 3, "completedSessions": 4, "abortedSessions": 1, "abortRatePercent": 20 }
      """
    When I open the app at "/review"
    Then I do not see "abgebrochen"
    When I click "Arbeitsweise"
    Then I see "abgebrochen"
    And I see "Zeitmessung"
    When I click "Timerstatistik öffnen"
    Then I see the heading "Arbeitsweise"

