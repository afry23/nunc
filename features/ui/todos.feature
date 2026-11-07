Feature: Todo list
  Source: REQUIREMENTS 4.2, 4.3; todo.txt syntax, filters, reordering

  @ui @REQ-4.2
  Scenario: Add a todo inline using todo.txt syntax
    When I open the app at "/todos"
    And I enter "(A) Bericht +work @desk" into the field "Neue Aufgabe"
    And I click "Hinzufügen"
    Then the backend received "add_todo" with:
      """
      { "rawText": "(A) Bericht +work @desk" }
      """
    And the todo list shows "(A) Bericht +work @desk"
    And the field "Neue Aufgabe" is empty

  @ui @REQ-4.2
  Scenario: Edit a todo inline
    Given the todos:
      | rawText           |
      | Bericht +work     |
    When I open the app at "/todos"
    And I click "Aktionen: Bericht"
    And I click "Bearbeiten"
    And I enter "Bericht abgeben +work" into the field "Aufgabe bearbeiten: Bericht"
    And I press the key "Enter" in the field "Aufgabe bearbeiten: Bericht"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht abgeben +work" }
      """
    And the todo list shows "Bericht abgeben +work"

  @ui @REQ-4.2
  Scenario: Check off a todo
    Given the todos:
      | rawText           |
      | Bericht +work     |
    When I open the app at "/todos"
    And I check "Erledigt: Bericht"
    Then the backend received "set_todo_completed" with:
      """
      { "id": 1, "completed": true }
      """
    And the todo list is empty
    When I click "Erledigte anzeigen (1)"
    Then the todo list shows "Bericht +work"

  @ui @REQ-4.2
  Scenario: Delete a todo from the list
    Given the todos:
      | rawText           |
      | Bericht +work     |
      | Einkaufen @street |
    When I open the app at "/todos"
    And I click "Aktionen: Bericht"
    And I click "Löschen"
    Then the backend received "delete_todo" with:
      """
      { "id": 1 }
      """
    And the todo list shows "Einkaufen @street"

  @ui @REQ-4.3
  Scenario: The pomodoro count is shown next to the todo
    Given the todos:
      | rawText           | pomodoroCount |
      | Bericht +work     | 3             |
      | Einkaufen @street | 0             |
    When I open the app at "/todos"
    Then the todo "Bericht" shows 3 pomodoros
    And the todo "Einkaufen" shows no pomodoros

  @ui @REQ-4.2 @desktop
  Scenario: Reorder todos by drag and drop
    Given the todos:
      | rawText |
      | Eins    |
      | Zwei    |
      | Drei    |
    When I open the app at "/todos"
    And I drag the todo "Eins" below the todo "Zwei"
    Then the backend received "reorder_todos" with:
      """
      { "ids": [2, 1, 3] }
      """
    And the todo list shows "Zwei, Eins, Drei"

  @ui @REQ-4.2
  Scenario: Filters can be set and cleared
    Given the todos:
      | rawText                |
      | (A) Bericht +work      |
      | Einkaufen @street      |
      | Steuer +privat @desk   |
    When I open the app at "/todos"
    And I click "Projekte"
    And I click "+work"
    Then the todo list shows "(A) Bericht +work"
    When I click "+work entfernen"
    And I click "Kontexte"
    And I click "@street"
    Then the todo list shows "Einkaufen @street"
    When I click "Filter zurücksetzen"
    Then the todo list shows "(A) Bericht +work, Einkaufen @street, Steuer +privat @desk"

  @ui @REQ-4.2
  Scenario: The search box filters while typing
    Given the todos:
      | rawText           |
      | Bericht +work     |
      | Einkaufen @street |
    When I open the app at "/todos"
    And I enter "bericht" into the field "Aufgaben durchsuchen"
    Then the todo list shows "Bericht +work"
    When I enter "" into the field "Aufgaben durchsuchen"
    Then the todo list shows "Bericht +work, Einkaufen @street"

  @ui @REQ-4.2 @mobile
  Scenario: Reorder todos by touch
    Given the platform is "mobile"
    And the todos:
      | rawText |
      | Eins    |
      | Zwei    |
      | Drei    |
    When I open the app at "/todos"
    Then I see the buttons "Verschieben: Eins, Aktionen: Eins"
    When I click "Aktionen: Eins"
    And I click "Nach unten"
    Then the backend received "reorder_todos" with:
      """
      { "ids": [2, 1, 3] }
      """
    And the todo list shows "Zwei, Eins, Drei"

  @ui @REQ-4.2 @mobile
  Scenario: Swiping a todo to the right completes it
    Given the platform is "mobile"
    And the todos:
      | rawText   |
      | Einkaufen |
    When I open the app at "/todos"
    And I swipe the todo "Einkaufen" to the right by 120 pixels
    Then the backend received "set_todo_completed" with:
      """
      { "id": 1, "completed": true }
      """

  @ui @REQ-4.2 @mobile
  Scenario: A short or vertical swipe does not complete a todo
    Given the platform is "mobile"
    And the todos:
      | rawText   |
      | Einkaufen |
    When I open the app at "/todos"
    And I swipe the todo "Einkaufen" to the right by 30 pixels
    Then the backend did not receive "set_todo_completed"

  @ui @REQ-4.2
  Scenario: Subtasks are shown as an indented tree
    Given the todos:
      | rawText | parent |
      | Projekt |        |
      | Teil A  | 1      |
      | Teil B  | 1      |
      | Anderes |        |
    When I open the app at "/todos"
    Then the todo "Teil A" is at depth 1
    And the todo "Projekt" is at depth 0

  @ui @REQ-4.2
  Scenario: Indenting a todo makes it a subtask of the one above
    Given the todos:
      | rawText |
      | Projekt |
      | Teil A  |
    When I open the app at "/todos"
    And I click "Aktionen: Teil A"
    And I click "Einrücken"
    Then the backend received "set_todo_parent" with:
      """
      { "id": 2, "parentId": 1 }
      """
    And the todo "Teil A" is at depth 1

  @ui @REQ-4.2
  Scenario: Adding a subtask
    Given the todos:
      | rawText |
      | Projekt |
    When I open the app at "/todos"
    And I click "Aktionen: Projekt"
    And I click "Unteraufgabe hinzufügen"
    And I enter "Teil A" into the field "Neue Unteraufgabe zu Projekt"
    And I press the key "Enter" in the field "Neue Unteraufgabe zu Projekt"
    Then the backend received "add_subtask" with:
      """
      { "parentId": 1, "rawText": "Teil A" }
      """
    And the todo "Teil A" is at depth 1

  @ui @US-UM03
  Scenario: A row shows only the main actions
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    Then I see the buttons "Starten: Bericht, Startmodus: Bericht, Aktionen: Bericht"
    And I do not see the menu item "Löschen"
    When I click "Aktionen: Bericht"
    Then I see the menu items "Für heute einplanen, Unteraufgabe hinzufügen, Duplizieren, Bearbeiten, Löschen"

  @ui @US-UM03
  Scenario: A task is started from its row
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Starten: Bericht"
    Then the backend received "start_timer" with:
      """
      { "mode": "pomodoro", "todoId": 1 }
      """

  @ui @US-UM03
  Scenario: A task is started in another mode from its row
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Startmodus: Bericht"
    And I click "Stoppuhr"
    Then the backend received "start_timer" with:
      """
      { "mode": "stopwatch", "todoId": 1 }
      """
    And I see the heading "Timer"

  @ui @US-UM03
  Scenario: A completed task cannot be started
    Given the todos:
      | rawText                | completed |
      | x 2026-03-03 Bericht   | true      |
    When I open the app at "/tasks"
    And I click "Erledigte anzeigen (1)"
    Then I do not see the button "Starten: Bericht"

  @ui @US-UM03
  Scenario: A task is duplicated as an open copy
    Given the todos:
      | rawText              |
      | Bericht +work est:30 |
    When I open the app at "/tasks"
    And I click "Aktionen: Bericht"
    And I click "Duplizieren"
    Then the backend received "add_todo" with:
      """
      { "rawText": "Bericht +work est:30" }
      """

  @ui @US-UM03
  Scenario: A row with subtasks shows their progress
    Given the todos:
      | rawText | parent | completed |
      | Projekt |        | false     |
      | Teil A  | 1      | true      |
      | Teil B  | 1      | false     |
    When I open the app at "/tasks"
    Then I see "1/2 Teilaufgaben"

  @ui @US-RV03
  Scenario Outline: The list shows the estimate of a task
    Given the todos:
      | rawText      |
      | <rawText>    |
    When I open the app at "/tasks"
    Then I see "<shown>"

    Examples:
      | rawText         | shown                |
      | Bericht estp:3  | geschätzt 3 Pomodoros |
      | Bericht est:90  | geschätzt 1h 30m     |

  @ui @US-RV03
  Scenario: A task without estimate shows no estimate
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    Then I do not see "geschätzt"

  @ui @US-RV03
  Scenario: The estimate joins today and the tracked time in the list
    Given the todos:
      | rawText                          |
      | Bericht plan:2026-03-04 estp:2   |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 25    |
    When I open the app at "/tasks"
    Then I see "Heute · geschätzt 2 Pomodoros · 25m erfasst"

  @ui @US-UM03
  Scenario: A planned task shows today and its tracked time
    Given the todos:
      | rawText                 |
      | Bericht plan:2026-03-04 |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned | 60       | 75    |
    When I open the app at "/tasks"
    Then I see "Heute · 1h 15m erfasst"
    When I click "Aktionen: Bericht"
    Then I see the menu items "Vom Plan nehmen"

  @ui @US-UM03
  Scenario: Import and export are in the overflow menu
    When I open the app at "/tasks"
    Then I do not see the button "Importieren"
    When I click "Weitere Aktionen"
    Then I see the menu items "Importieren, Exportieren"

  @ui @US-UM03
  Scenario: Clicking the title opens the edit dialog over the list
    Given the todos:
      | rawText                     |
      | (A) Bericht +work @office   |
      | Einkaufen                   |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then I see a dialog with the text "Aufgabe bearbeiten"
    And the field "Aufgabe bearbeiten (todo.txt-Zeile)" contains "(A) Bericht"
    And the todo list shows "(A) Bericht +work @office, Einkaufen"

  @ui @US-UM03
  Scenario: The dialog lists the time entries of the task with their total
    Given the todos:
      | rawText |
      | Bericht |
    And the time entries of todo 1 are:
      | start                | minutes | mode      | note       |
      | 2026-03-03T09:00:00Z | 25      | pomodoro  |            |
      | 2026-03-02T09:00:00Z | 90      | stopwatch | Kickoff    |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then I see "Zeiteinträge"
    And I see "Gesamt: 1h 55m"
    And I see "Kickoff"
    And I see "1h 30m"

  @ui @US-RV03
  Scenario: The dialog edits the task line
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I enter "Bericht schreiben +work" into the field "Aufgabe bearbeiten (todo.txt-Zeile)"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht schreiben +work" }
      """

  @ui @US-RV03
  Scenario: The time needed is planned in pomodoros
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I enter "3" into the field "Anzahl Pomodoros"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht estp:3" }
      """

  @ui @US-RV03
  Scenario: The time needed is planned in hours and minutes
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I choose the option "Stunden und Minuten"
    And I enter "1" into the field "Stunden"
    And I enter "30" into the field "Minuten"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht est:90" }
      """

  @ui @US-RV03
  Scenario: The time needed can be removed
    Given the todos:
      | rawText         |
      | Bericht estp:3  |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I enter "" into the field "Anzahl Pomodoros"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht" }
      """

  @ui @US-RV03
  Scenario: Changing the time needed keeps a single estimate in the line
    Given the todos:
      | rawText                        |
      | (A) Bericht +work plan:2026-11-06 estp:2 |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then the field "Aufgabe bearbeiten (todo.txt-Zeile)" contains "(A) Bericht plan:2026-11-06"
    And the field "Anzahl Pomodoros" contains "2"
    When I enter "3" into the field "Anzahl Pomodoros"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "(A) Bericht plan:2026-11-06 +work estp:3" }
      """

  @ui @US-RV03
  Scenario: Projects and contexts are edited as tags, the text stays in the line
    Given the todos:
      | rawText                           |
      | (A) Bericht +work @office est:30  |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then the field "Aufgabe bearbeiten (todo.txt-Zeile)" contains "(A) Bericht"
    When I click "+work entfernen"
    And I enter "+home" into the field "Projekt hinzufügen"
    And I press the key "Enter" in the field "Projekt hinzufügen"
    And I enter "@phone" into the field "Kontext hinzufügen"
    And I press the key "Enter" in the field "Kontext hinzufügen"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "(A) Bericht +home @office @phone est:30" }
      """

  @ui @US-RV03
  Scenario: A tag typed into the line is not duplicated by the tag input
    Given the todos:
      | rawText         |
      | Bericht +work   |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I enter "Bericht +work @office" into the field "Aufgabe bearbeiten (todo.txt-Zeile)"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht +work @office" }
      """

  @ui @US-RV03
  Scenario: Switching the unit replaces the estimate
    Given the todos:
      | rawText        |
      | Bericht estp:3 |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I choose the option "Stunden und Minuten"
    And I enter "45" into the field "Minuten"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht est:45" }
      """

  @ui @US-RV03
  Scenario: Nothing is saved with an invalid time needed
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I enter "0" into the field "Anzahl Pomodoros"
    Then the button "Speichern" is disabled

  @ui @US-RV03
  Scenario: The edit dialog closes after saving and Cancel leaves the task unchanged
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then I see a dialog with the text "Aufgabe bearbeiten"
    When I click "Abbrechen"
    Then I do not see a dialog
    And the backend did not receive "update_todo"

  @ui @US-RV03
  Scenario: The dialog compares the estimate with the effort so far
    Given the todos:
      | rawText        |
      | Bericht estp:4 |
    And the task time of todo 1 is:
      """
      { "estimateCheck": { "estimate": { "unit": "pomodoros", "value": 4 }, "actual": 5, "delta": 1, "verdict": "over" } }
      """
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then I see "Bisher 5 Pomodoros von 4 Pomodoros (+1 Pomodoros)"
    And I see "länger als geschätzt"

  @ui @US-UM03
  Scenario: A task without time says so
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then I see "Noch keine Zeit erfasst."

  @ui @US-UM03 @mobile
  Scenario: On a phone the edit dialog opens over the list and cancel keeps the filter
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the todos:
      | rawText   |
      | Bericht   |
      | Einkaufen |
    When I open the app at "/tasks"
    And I enter "Bericht" into the field "Aufgaben durchsuchen"
    And I click "Details: Bericht"
    Then I see a dialog with the text "Aufgabe bearbeiten"
    When I click "Abbrechen"
    Then I do not see a dialog
    And the field "Aufgaben durchsuchen" contains "Bericht"
    And the todo list shows "Bericht"

  @ui @US-UM03
  Scenario: The raw todo.txt line is edited in the dialog
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I enter "Bericht abgeben +work" into the field "Aufgabe bearbeiten (todo.txt-Zeile)"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht abgeben +work" }
      """

  @ui @US-UM03
  Scenario: A task is started from its dialog
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I click "Arbeit starten"
    Then the backend received "start_timer" with:
      """
      { "mode": "pomodoro", "todoId": 1 }
      """

  @ui @REQ-4.3
  Scenario: A running timer for a todo shows pause instead of start
    Given the todos:
      | rawText | completed |
      | Bericht | false     |
      | Mail    | false     |
    And the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total | todoId |
      | running | pomodoro | work        | 754       | 1500  | 1      |
    When I open the app at "/todos"
    Then I see the buttons "Pausieren: Bericht, Starten: Mail"
    And I do not see the button "Starten: Bericht"
    When I click "Pausieren: Bericht"
    Then the backend received "pause_timer"

  @ui @REQ-4.3
  Scenario: A paused timer for a todo shows resume
    Given the todos:
      | rawText | completed |
      | Bericht | false     |
    And the backend reports the timer as:
      | state  | mode     | sessionType | remaining | total | todoId |
      | paused | pomodoro | work        | 754       | 1500  | 1      |
    When I open the app at "/todos"
    And I click "Fortsetzen: Bericht"
    Then the backend received "resume_timer"

  @ui @REQ-4.3
  Scenario: A break of a todo does not turn its start button into pause
    Given the todos:
      | rawText | completed |
      | Bericht | false     |
    And the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total | todoId |
      | running | pomodoro | short_break | 240       | 300   | 1      |
    When I open the app at "/todos"
    Then I see the buttons "Starten: Bericht"

