Feature: Today
  Source: ADR 0027
  The day plan lists the open todos that are overdue, due today or planned for today, and compares their
  estimates with the daily capacity.

  @ui @US-TP01
  Scenario: The plan shows each entry with its reason and the capacity
    Given the todos:
      | rawText                            |
      | Bericht due:2026-03-02 est:30      |
      | Steuer due:2026-03-04              |
      | Einkaufen plan:2026-03-04 est:60   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | overdue | 30       | 15    |
      | 2    | due     |          | 0     |
      | 3    | planned | 60       | 0     |
    When I open the app at "/"
    Then I see the heading "Heute"
    And I see "Überfällig"
    And I see "Heute fällig"
    And I see "Eingeplant"
    And I see "Geplant: 1h 30m von 4h"
    And I see "Noch 2h 30m frei"
    And I see "1 ohne Schätzung"
    And I see "15m erfasst"

  @ui @US-TP01
  Scenario: The day has an overview and a timer tab
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app at "/"
    Then I see the heading "Heute"
    When I click "Timer"
    Then I see the heading "Timer"

  @ui @US-TP01
  Scenario: Starting a task of the plan opens the timer with the task selected
    Given the todos:
      | rawText                          |
      | Bericht due:2026-03-02 est:30    |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | overdue | 30       | 0     |
    When I open the app at "/"
    And I click "Starten: Bericht"
    Then I see the heading "Timer"
    And the field "Aktive Aufgabe" has the value "1"
    And the backend did not receive "start_timer"

  @ui @US-TP01
  Scenario: Free activities of the day are listed apart from the plan
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    And the timeline of "2026-03-04" has the sessions:
      | start | end   | type | todo | note       | origin |
      | 09:00 | 09:45 | work |      | Abstimmung | timer  |
    When I open the app at "/"
    Then I see "Freie Tätigkeiten"
    And I see "Abstimmung"
    And I see "45m"

  @ui @US-TP01
  Scenario: The tracked work is shown apart from the planned effort
    Given the todos:
      | rawText                          |
      | Bericht plan:2026-03-04 est:60   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned | 60       | 0     |
    And the timeline of "2026-03-04" has the sessions:
      | start | end   | type        | todo | note | origin |
      | 09:00 | 09:25 | work        |      |      | timer  |
      | 09:25 | 09:30 | short_break |      |      | timer  |
      | 10:00 | 10:35 | work        |      |      | manual |
    When I open the app
    Then I see "Geplant: 1h von 4h"
    And I see "Erfasste Arbeit: 1h"

  @ui @US-TP01
  Scenario: An overfull plan shows how much is too much
    Given the todos:
      | rawText                          |
      | Gross plan:2026-03-04 est:90     |
    And the day plan for 2026-03-04 has a capacity of 60 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned | 90       | 0     |
    When I open the app at "/"
    Then I see "30m zu viel"

  @ui @US-TP01
  Scenario: An empty plan explains how to plan something
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app at "/"
    Then I see "Was möchtest du heute angehen?"
    And I see "Aufgaben auswählen"
    And I see "Neue Aufgabe hinzufügen"
    And I see "Du kannst auch ohne Aufgabe eine Tätigkeit starten."

  @ui @US-TP01
  Scenario: Set the estimate of an entry
    Given the todos:
      | rawText                   |
      | Bericht plan:2026-03-04   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 0     |
    When I open the app at "/"
    And I click "Schätzung ändern: Bericht"
    And I enter "45" into the field "Schätzung (min): Bericht"
    And I leave the field "Schätzung (min): Bericht"
    Then the backend received "set_todo_estimate" with:
      """
      { "id": 1, "estimateMin": 45 }
      """

  @ui @US-TP01
  Scenario: Move a planned entry to tomorrow
    Given the todos:
      | rawText                   |
      | Bericht plan:2026-03-04   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 0     |
    When I open the app at "/"
    And I click "Aktionen: Bericht"
    And I click "Auf morgen verschieben"
    Then the backend received "plan_todo" with:
      """
      { "id": 1, "day": "2026-03-05" }
      """

  @ui @US-TP01
  Scenario: Take a planned entry off the plan
    Given the todos:
      | rawText                   |
      | Bericht plan:2026-03-04   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 0     |
    When I open the app at "/"
    And I click "Aktionen: Bericht"
    And I click "Vom Plan nehmen"
    Then the backend received "plan_todo" with:
      """
      { "id": 1, "day": null }
      """

  @ui @US-TP01
  Scenario: An overdue entry can only be moved by changing its due date
    Given the todos:
      | rawText                 |
      | Bericht due:2026-03-02  |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | overdue |          | 0     |
    When I open the app at "/"
    Then I see "Überfällig"
    And I do not see the button "Aktionen: Bericht"

  @ui @US-TP01
  Scenario: Check off an entry of the plan
    Given the todos:
      | rawText                   |
      | Bericht plan:2026-03-04   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 0     |
    When I open the app at "/"
    And I check "Erledigt: Bericht"
    Then the backend received "set_todo_completed" with:
      """
      { "id": 1, "completed": true }
      """

  @ui @US-TP01
  Scenario: Plan a todo for today from the list
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Aktionen: Bericht"
    And I click "Für heute einplanen"
    Then the backend received "plan_todo" with "id" equal to "1"

  @ui @US-TP01
  Scenario: The daily capacity is set in the settings
    When I open the app at "/settings"
    And I enter "180" into the field "Tageskapazität"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "dayCapacityMin": 180 } }
      """

  @ui @US-TP02
  Scenario: The day note is shown and can be saved
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    And the review is:
      """
      { "journal": { "achievements": [], "weekNotes": [],
        "dayNotes": [{ "id": "n", "date": "2026-03-04", "text": "Calm day" }] } }
      """
    When I open the app at "/"
    Then the field "Tagesnotiz" has the value "Calm day"
    And the button "Notiz speichern" is disabled
    When I enter "Calm day, good focus" into the field "Tagesnotiz"
    And I click "Notiz speichern"
    Then the backend received "save_journal_note" with:
      """
      { "kind": "day_note", "date": "2026-03-04", "text": "Calm day, good focus" }
      """

  @ui @US-TP02
  Scenario: Closing the day moves the planned tasks to tomorrow by default
    Given the todos:
      | rawText                           |
      | Einkaufen plan:2026-03-04         |
      | Bericht due:2026-03-04            |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 0     |
      | 2    | due     |          | 0     |
    When I open the app at "/"
    And I click "Tagesmenü"
    And I click "Tag abschließen"
    Then I see a dialog with the text "1 eingeplante Aufgaben, 1 mit Frist."
    And I see a dialog with the text "Aufgaben mit Frist bleiben auf dem Plan"
    When I click "Abschließen"
    Then the backend received "close_day" with:
      """
      { "action": "postpone", "note": null }
      """
    And I do not see a dialog

  @ui @US-TP02
  Scenario: The planned tasks can be taken off the plan and a note can be added
    Given the todos:
      | rawText                    |
      | Einkaufen plan:2026-03-04  |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 0     |
    When I open the app at "/"
    And I click "Tagesmenü"
    And I click "Tag abschließen"
    And I click "Vom Plan nehmen"
    And I enter "Too much planned" into the field "Notiz zum Tag"
    And I click "Abschließen"
    Then the backend received "close_day" with:
      """
      { "action": "unplan", "note": "Too much planned" }
      """

  @ui @US-TP02
  Scenario: Cancelling the dialog changes nothing
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app at "/"
    And I click "Tagesmenü"
    And I click "Tag abschließen"
    And I click "Abbrechen"
    Then I do not see a dialog
    And the backend did not receive "close_day"

  @ui @US-TP02
  Scenario: Without planned tasks the dialog offers only the note
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app at "/"
    And I click "Tagesmenü"
    And I click "Tag abschließen"
    Then I see a dialog with the text "0 eingeplante Aufgaben, 0 mit Frist."
    And I do not see the button "Unverändert lassen"
