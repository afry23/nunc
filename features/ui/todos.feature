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
    And I click "Bearbeiten: Bericht"
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
    And I click "Löschen: Bericht"
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
    Then I see the buttons "Verschieben: Eins, Nach unten: Eins"
    When I click "Nach unten: Eins"
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
    And I click "Einrücken: Teil A"
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
    And I click "Unteraufgabe hinzufügen: Projekt"
    And I enter "Teil A" into the field "Neue Unteraufgabe zu Projekt"
    And I press the key "Enter" in the field "Neue Unteraufgabe zu Projekt"
    Then the backend received "add_subtask" with:
      """
      { "parentId": 1, "rawText": "Teil A" }
      """
    And the todo "Teil A" is at depth 1
