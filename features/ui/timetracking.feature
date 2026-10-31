Feature: Time tracking in the statistics view
  Source: ADR 0017
  The view "Zeiten" in the navigation shows the weekly time report and the day timeline with editing.

  Background:
    Given the todos:
      | rawText      |
      | Write report |

  @ui @US-TT01
  Scenario: The weekly report shows time per project and weekday
    Given the timesheet grouped by "project" contains:
      | group | Mo  | Di | Mi | Do | Fr | Sa | So |
      | +work | 120 | 90 | 0  | 60 | 0  | 0  | 0  |
      | +home | 0   | 0  | 45 | 0  | 0  | 0  | 0  |
      |       | 30  | 0  | 0  | 0  | 0  | 0  | 0  |
    When I open the app at "/times"
    Then the timesheet shows:
      | Gruppe          | Mo     | Di     | Mi  | Do | Fr | Sa | So | Summe  |
      | +work           | 2h     | 1h 30m | –   | 1h | –  | –  | –  | 4h 30m |
      | +home           | –      | –      | 45m | –  | –  | –  | –  | 45m    |
      | ohne Zuordnung  | 30m    | –      | –   | –  | –  | –  | –  | 30m    |
      | Gesamt          | 2h 30m | 1h 30m | 45m | 1h | –  | –  | –  | 5h 45m |

  @ui @US-TT01
  Scenario: The report can be switched between project, context and todo
    Given the timesheet grouped by "project" contains:
      | group | Mo | Di | Mi | Do | Fr | Sa | So |
      | +work | 60 | 0  | 0  | 0  | 0  | 0  | 0  |
    And the timesheet grouped by "context" contains:
      | group | Mo | Di | Mi | Do | Fr | Sa | So |
      | @desk | 30 | 0  | 0  | 0  | 0  | 0  | 0  |
    And the timesheet grouped by "todo" contains:
      | group        | Mo | Di | Mi | Do | Fr | Sa | So |
      | Write report | 15 | 0  | 0  | 0  | 0  | 0  | 0  |
    When I open the app at "/times"
    Then the timesheet was requested grouped by "project"
    And I see "+work"
    When I click "Kontext"
    Then the timesheet was requested grouped by "context"
    And I see "@desk"
    When I click "Aufgabe"
    Then the timesheet was requested grouped by "todo"
    And I see "15m"

  @ui @US-TT01
  Scenario: I can move to the previous and the next week
    When I open the app at "/times"
    Then the week shown starts 0 weeks from the current week
    When I click "Vorherige Woche"
    Then the week shown starts -1 weeks from the current week
    When I click "Nächste Woche"
    And I click "Nächste Woche"
    Then the week shown starts 1 weeks from the current week

  @ui @US-TT01
  Scenario: The today button returns to the current week
    When I open the app at "/times"
    Then the button "Heute" is disabled
    When I click "Vorherige Woche"
    Then the week shown starts -1 weeks from the current week
    And the button "Heute" is enabled
    When I click "Heute"
    Then the week shown starts 0 weeks from the current week
    And the button "Heute" is disabled

  @ui @US-TT02
  Scenario: Tapping a block opens the edit dialog
    Given the timeline of today has the sessions:
      | start | end   | type | todo         | note | origin |
      | 09:00 | 10:30 | work | Write report | Spec | timer  |
    When I open the app at "/times"
    And I click "09:00–10:30 Write report"
    Then I see a dialog with the text "Eintrag bearbeiten"
    And the field "Beginn" shows today at "09:00"
    And the field "Ende" shows today at "10:30"
    And the field "Notiz" has the value "Spec"
    When I enter "Specs" into the field "Notiz"
    And I click "Speichern"
    Then the last "update_session" call has "input.note" equal to "Specs"
    And I do not see a dialog
    And I see "Specs"

  @ui @US-TT02
  Scenario: Deleting a session needs a confirmation
    Given the timeline of today has the sessions:
      | start | end   | type | todo         | note | origin |
      | 09:00 | 10:30 | work | Write report | Spec | timer  |
    When I open the app at "/times"
    And I click "09:00–10:30 Write report"
    And I click "Löschen"
    Then I see a dialog with the text "Eintrag löschen?"
    And the backend did not receive "delete_session"
    When I click "Abbrechen"
    Then I see a dialog with the text "Eintrag bearbeiten"
    And the backend did not receive "delete_session"
    When I click "Löschen"
    And I click "Löschen"
    Then the backend received "delete_session"
    And I do not see a dialog
    And I do not see "Spec"

  @ui @US-TT03
  Scenario: I can add a missing entry
    When I open the app at "/times"
    And I click "Eintrag hinzufügen"
    Then I see a dialog with the text "Eintrag hinzufügen"
    When I enter "10:00" for today into the field "Beginn"
    And I enter "11:30" for today into the field "Ende"
    And I select "Write report" in the field "Aufgabe"
    And I enter "Workshop" into the field "Notiz"
    And I click "Speichern"
    Then the last "create_manual_session" call has "input.note" equal to "Workshop"
    And I do not see a dialog
    And I see "Workshop"
    And I see "nachgetragen"

  @ui @US-TT03
  Scenario: The task choice offers only open tasks
    Given the todos:
      | rawText      | completed |
      | Write report | false     |
      | Old task     | true      |
    When I open the app at "/times"
    And I click "Eintrag hinzufügen"
    Then the field "Aufgabe" has the option "Write report"
    And the field "Aufgabe" has no option "Old task"

  @ui @US-TT02
  Scenario: An entry keeps its completed task when edited
    Given the todos:
      | rawText      | completed |
      | Write report | false     |
      | Old task     | true      |
    And the timeline of today has the sessions:
      | start | end   | type | todo     | note | origin |
      | 09:00 | 10:30 | work | Old task | Spec | timer  |
    When I open the app at "/times"
    And I click "09:00–10:30 Old task"
    Then the field "Aufgabe" has the option "Old task"
    And the field "Aufgabe" has the option "Write report"

  @ui @US-TT03
  Scenario: An invalid range shows an error in the dialog
    Given saving a session fails with "session_invalid_range": "Das Ende muss nach dem Beginn liegen."
    When I open the app at "/times"
    And I click "Eintrag hinzufügen"
    And I enter "11:00" for today into the field "Beginn"
    And I enter "10:00" for today into the field "Ende"
    And I click "Speichern"
    Then I see a dialog with the text "Das Ende muss nach dem Beginn liegen."
    And the backend received "create_manual_session"

  @ui @US-TT04
  Scenario: The timeline shows the sessions of the chosen day
    Given the timeline of the weekday "Mo" has the sessions:
      | start | end   | type | todo         | note | origin |
      | 09:00 | 10:30 | work | Write report | Mon  | timer  |
    And the timeline of the weekday "Di" has the sessions:
      | start | end   | type        | todo | note | origin |
      | 14:00 | 15:00 | work        |      | Tue  | manual |
      | 15:00 | 15:05 | short_break |      |      | timer  |
    When I open the app at "/times"
    And I choose the weekday "Di"
    Then I see "Tagesverlauf Di"
    And I see "Tue"
    And I see "Kurze Pause"
    And I do not see "Mon"
