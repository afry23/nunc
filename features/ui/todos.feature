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
    And I click "+work"
    Then the todo list shows "(A) Bericht +work"
    When I click "+work"
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
