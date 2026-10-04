Feature: todo.txt import and export in the todo list
  Source: nunc extension (not in the Go app); the file format is plain todo.txt, one todo per line

  @ui @REQ-4.4
  Scenario: Importing a todo.txt file reports what was added
    Given the todo.txt import adds 3 todos and skips 1 duplicate
    When I open the app at "/todos"
    And I click "Importieren"
    Then the backend received "import_todotxt_file"
    And I see "3 Aufgaben importiert, 1 Duplikat übersprungen."

  @ui @REQ-4.4
  Scenario: A file without new lines says so
    Given the todo.txt import adds 0 todos and skips 0 duplicate
    When I open the app at "/todos"
    And I click "Importieren"
    Then I see "Die Datei enthält keine neuen Aufgaben."

  @ui @REQ-4.4
  Scenario: A cancelled import dialog changes nothing
    Given the todo.txt import dialog is cancelled
    When I open the app at "/todos"
    And I click "Importieren"
    Then I do not see "Aufgaben importiert"

  @ui @REQ-4.4
  Scenario: Exporting the todos saves a todo.txt file
    Given the todo.txt export is saved to "/tmp/todo.txt"
    When I open the app at "/todos"
    And I click "Exportieren"
    Then the backend received "save_todotxt_file"
    And I see "Aufgaben exportiert: /tmp/todo.txt"
