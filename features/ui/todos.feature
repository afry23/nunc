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
  Scenario: Clicking the title opens the details next to the list
    Given the todos:
      | rawText                     |
      | (A) Bericht +work @office   |
      | Einkaufen                   |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then I see the heading "Aufgabendetails"
    And I see "Priorität"
    And I see "+work"
    And the todo list shows "(A) Bericht +work @office, Einkaufen"

  @ui @US-UM03
  Scenario: The details list the time entries of the task with their total
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

  @ui @US-UM03
  Scenario: A task without time says so
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    Then I see "Noch keine Zeit erfasst."

  @ui @US-UM03 @mobile
  Scenario: On a phone the details replace the list and back returns to it with the filter
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the todos:
      | rawText   |
      | Bericht   |
      | Einkaufen |
    When I open the app at "/tasks"
    And I enter "Bericht" into the field "Aufgaben durchsuchen"
    And I click "Details: Bericht"
    Then I see the heading "Aufgabendetails"
    And the field "Aufgaben durchsuchen" does not exist
    When I click "Zurück zur Liste"
    Then the todo list shows "Bericht"

  @ui @US-UM03
  Scenario: The raw todo.txt line is edited in the details
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/tasks"
    And I click "Details: Bericht"
    And I enter "Bericht abgeben +work" into the field "todo.txt-Zeile"
    And I click "Speichern"
    Then the backend received "update_todo" with:
      """
      { "id": 1, "rawText": "Bericht abgeben +work" }
      """

  @ui @US-UM03
  Scenario: A task is started from its details
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

  @ui @US-UM03
  Scenario: The details of a task that is gone say so
    When I open the app at "/tasks?task=99"
    Then I see "Diese Aufgabe gibt es nicht mehr."

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

