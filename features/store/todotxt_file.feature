Feature: todo.txt import and export in the database
  Source: nunc extension (not in the Go app); the file format is plain todo.txt, one todo per line

  @integration @REQ-4.4
  Scenario: Imported todos are appended after the existing ones
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Existing"
    And I import the todo.txt lines:
      """
      (A) 2026-10-01 Write report +work @desk
      x 2026-10-02 2026-10-01 Done already +home
      """
    Then the todo 2 is:
      | field       | value                                   |
      | rawText     | (A) 2026-10-01 Write report +work @desk |
      | priority    | A                                       |
      | completed   | false                                   |
      | description | Write report +work @desk                |
      | projectTags | ["+work"]                               |
      | position    | 1                                       |
    And the todo 3 is:
      | field       | value                                   |
      | rawText     | x 2026-10-02 2026-10-01 Done already +home |
      | completed   | true                                    |
      | completedAt | 2026-10-02                              |
      | position    | 2                                       |

  @integration @REQ-4.4
  Scenario: A deleted todo does not count as a duplicate
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Buy milk"
    And I delete todo 1
    And I import the todo.txt lines:
      """
      Buy milk
      """
    And I list the todos
    Then the listed todos are:
      | rawText  |
      | Buy milk |

  @integration @REQ-4.4
  Scenario: The export from the database lists the active todo.txt lines
    Given no database exists yet
    When nunc opens the database
    And I add the todo "(A) Write report +work"
    And I add the todo "Call dentist @phone"
    And I add the todo "Gone"
    And I delete todo 3
    And I complete todo 2
    And I export the todos from the database
    Then the exported text is:
      """
      (A) Write report +work
      x 2026-10-04 Call dentist @phone
      """

  @integration @REQ-4.4
  Scenario: Importing the export again adds nothing
    Given no database exists yet
    When nunc opens the database
    And I add the todo "(A) Write report +work"
    And I add the todo "Call dentist @phone"
    And I export the todos from the database
    And I import the exported text
    Then 0 todos were added and 2 duplicates were skipped
