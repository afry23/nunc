Feature: todo.txt import and export
  Source: nunc extension (not in the Go app); the file format is plain todo.txt, one todo per line

  Background:
    Given the todos:
      | id | line                         | pomodoros | position |
      | 1  | (A) Write report +work @desk | 3         | 2        |
      | 2  | Call dentist @phone          | 0         | 1        |
      | 3  | x 2025-03-10 Plan trip +home | 1         | 3        |

  @domain @REQ-4.4
  Scenario: The export lists the todo.txt lines in the manual order
    When I export the todos
    Then the todo.txt file is:
      """
      Call dentist @phone
      (A) Write report +work @desk
      x 2025-03-10 Plan trip +home
      """

  @domain @REQ-4.4
  Scenario: The export can leave out completed todos
    When I export the todos without completed ones
    Then the todo.txt file is:
      """
      Call dentist @phone
      (A) Write report +work @desk
      """

  @domain @REQ-4.4
  Scenario: Nothing to export yields an empty file
    Given the todos:
      | id | line | pomodoros | position |
    When I export the todos
    Then the todo.txt file is empty

  @domain @REQ-4.4
  Scenario: Every line of the file becomes a todo, in file order
    When I import the todo.txt file:
      """
      Buy milk @shop
      (B) 2025-03-01 Call mum
      x 2025-03-02 Pay rent +home
      """
    Then the lines to import are:
      """
      Buy milk @shop
      (B) 2025-03-01 Call mum
      x 2025-03-02 Pay rent +home
      """
    And 0 duplicates are skipped

  @domain @REQ-4.4
  Scenario: Blank lines and surrounding whitespace are ignored
    When I import the todo.txt file:
      """
      Buy milk

         
        Call mum   
      """
    Then the lines to import are:
      """
      Buy milk
      Call mum
      """

  @domain @REQ-4.4
  Scenario: Windows line endings are accepted
    When I import the todo.txt file with Windows line endings:
      """
      Buy milk
      Call mum
      """
    Then the lines to import are:
      """
      Buy milk
      Call mum
      """

  @domain @REQ-4.4
  Scenario: A byte order mark at the start of the file is ignored
    When I import the todo.txt file with a byte order mark:
      """
      (A) Buy milk
      """
    Then the lines to import are:
      """
      (A) Buy milk
      """

  @domain @REQ-4.4
  Scenario: Lines that already exist as todos are skipped
    When I import the todo.txt file:
      """
      Call dentist @phone
      Buy milk
      x 2025-03-10 Plan trip +home
      """
    Then the lines to import are:
      """
      Buy milk
      """
    And 2 duplicates are skipped

  @domain @REQ-4.4
  Scenario: A line that appears twice in the file is imported once
    When I import the todo.txt file:
      """
      Buy milk
      Call mum
      Buy milk
      """
    Then the lines to import are:
      """
      Buy milk
      Call mum
      """
    And 1 duplicates are skipped

  @domain @REQ-4.4
  Scenario: Importing the export again adds nothing
    When I import the exported todos
    Then there are no lines to import
    And 3 duplicates are skipped

  @domain @REQ-4.4
  Scenario: An empty file adds nothing
    When I import the todo.txt file with a byte order mark:
      """
      """
    Then there are no lines to import
    And 0 duplicates are skipped
