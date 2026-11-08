Feature: Review in the times view
  Source: ADR 0025
  The view "Zeiten" switches to a review of a week, month or quarter with the comparison to the period before.

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
    And I click "Als Erfolg merken"
    Then the backend received "add_achievement" with "text" equal to "Report"
    And I see "Als Erfolg im Rückblick gemerkt."

  @ui @US-RV02
  Scenario: An open todo cannot be saved as an achievement
    Given the todos:
      | rawText   |
      | Open task |
    When I open the app at "/todos"
    And I click "Aktionen: Open task"
    Then I do not see the menu item "Als Erfolg merken"

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

  @ui @US-RV04
  Scenario: The review lists the tasks worked on with their status
    Given the review is:
      """
      { "tasks": [
        { "todoId": 1, "text": "Write report", "minutes": 90, "pomodoros": 2, "sessions": 3,
          "completed": false, "completedAt": null, "projects": ["+work"], "estimateCheck": null },
        { "todoId": 2, "text": "Plan trip", "minutes": 30, "pomodoros": 1, "sessions": 1,
          "completed": true, "completedAt": "2026-03-04", "projects": [], "estimateCheck": null } ] }
      """
    When I open the app at "/review"
    Then I see "Bearbeitete Aufgaben"
    And I see "Write report"
    And I see "offen"
    And I see "Plan trip"
    And I see "erledigt am 04.03."
    And I do not see "1h 30m"

  @ui @US-RV04
  Scenario: Without worked-on tasks the review says so
    When I open the app at "/review"
    Then I see "Keine Aufgaben bearbeitet."
    And I see "Keine freien Tätigkeiten."

  @ui @US-RV04
  Scenario: The review lists free activities one by one
    Given the review is:
      """
      { "activities": [
        { "sessionId": 1, "date": "2026-03-04", "text": "Daily", "minutes": 30, "mode": "stopwatch",
          "projects": ["+foo"], "contexts": ["@meeting"] },
        { "sessionId": 2, "date": "2026-03-04", "text": "", "minutes": 20, "mode": "stopwatch",
          "projects": [], "contexts": [] } ] }
      """
    When I open the app at "/review"
    Then I see "Daily"
    And I see "+foo @meeting"
    And I see "(ohne Text)"

  @ui @US-RV04
  Scenario: The review starts with the work and ends with the follow-up task
    When I open the app at "/review"
    Then the sections appear in the order "Bearbeitete Aufgaben, Freie Tätigkeiten, Ergebnisse und Beiträge, Notizen aus den Tagen, Reflexion, Für den nächsten Zeitraum"

  @ui @US-RV04
  Scenario: The review has no charts but leads to the statistics
    When I open the app at "/review"
    Then I do not see the charts "bar, pie"
    When I click "Zahlen und Diagramme ansehen"
    Then I see the heading "Auswertung"

  @ui @US-RV04
  Scenario: The review can be switched to a day and a year
    When I open the app at "/review"
    And I click "Tag"
    Then the backend received "get_review"
    When I click "Jahr"
    Then the backend received "get_review"
    And I see "Bearbeitete Aufgaben"
