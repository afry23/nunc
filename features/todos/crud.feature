Feature: Todo management
  Source: REQUIREMENTS 4.2, 4.3; nunc-go todos_test.go

  @integration @REQ-4.2
  Scenario: The todo list is empty on a fresh database
    Given no database exists yet
    When nunc opens the database
    And I list the todos
    Then the todo list is empty

  @integration @REQ-4.2
  Scenario: Add a todo
    Given no database exists yet
    When nunc opens the database
    And I add the todo "(A) 2026-10-01 Write report +work @desk"
    Then the todo 1 is:
      | field         | value                                |
      | rawText       | (A) 2026-10-01 Write report +work @desk |
      | priority      | A                                    |
      | completed     | false                                |
      | createdAt     | 2026-10-01                           |
      | description   | Write report +work @desk             |
      | projectTags   | ["+work"]                            |
      | contextTags   | ["@desk"]                            |
      | pomodoroCount | 0                                    |
      | position      | 0                                    |

  @integration @REQ-4.2
  Scenario: New todos are appended at the next position
    Given no database exists yet
    When nunc opens the database
    And I add the todo "first"
    And I add the todo "second"
    And I add the todo "third"
    And I delete todo 3
    And I add the todo "fourth"
    Then the todo 2 is:
      | field    | value |
      | position | 1     |
    And the todo 4 is:
      | field    | value |
      | position | 3     |

  @integration @REQ-4.2
  Scenario: Todos are returned ordered by position
    Given no database exists yet
    When nunc opens the database
    And I add the todo "first"
    And I add the todo "second"
    And I add the todo "third"
    And I reorder the todos to "3,1,2"
    And I list the todos
    Then the listed todos are:
      | rawText | position |
      | third   | 0        |
      | first   | 1        |
      | second  | 2        |

  @integration @REQ-4.2
  Scenario: Update a todo
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Old +a"
    And I update todo 1 to "(B) New +b @c"
    Then the todo 1 is:
      | field       | value         |
      | rawText     | (B) New +b @c |
      | priority    | B             |
      | description | New +b @c     |
      | projectTags | ["+b"]        |
      | contextTags | ["@c"]        |

  @integration @REQ-4.2
  Scenario: Updating a todo keeps its pomodoro count
    Given no database exists yet
    When nunc opens the database
    And I add the todo "first"
    And I add the todo "second"
    And todo 2 gets a pomodoro
    And todo 2 gets a pomodoro
    And I update todo 2 to "renamed"
    Then the todo 2 is:
      | field         | value   |
      | rawText       | renamed |
      | pomodoroCount | 2       |
      | position      | 1       |

  @integration @REQ-4.2
  Scenario: Complete a todo
    Given no database exists yet
    When nunc opens the database
    And I add the todo "(A) Task to complete +project"
    And I complete todo 1
    Then the todo 1 is:
      | field       | value                                |
      | rawText     | x 2026-10-04 Task to complete +project |
      | completed   | true                                 |
      | completedAt | 2026-10-04                           |
      | description | Task to complete +project            |
      | projectTags | ["+project"]                         |

  @integration @REQ-4.2
  Scenario: Uncomplete a todo
    Given no database exists yet
    When nunc opens the database
    And I add the todo "(A) Task to complete +project"
    And I complete todo 1
    And I uncomplete todo 1
    Then the todo 1 is:
      | field       | value                         |
      | rawText     | (A) Task to complete +project |
      | completed   | false                         |
      | completedAt |                               |
      | priority    | A                             |

  @integration @REQ-4.2
  Scenario: Delete a todo
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Deletable task"
    And I delete todo 1
    And I list the todos
    Then the todo list is empty
    And the todo 1 is marked as deleted

  @integration @REQ-4.2
  Scenario: Deleting a todo keeps its sessions
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Worked on"
    And a pomodoro for todo 1 is started at 09:00:00 and saved
    And the pomodoro is finished at 09:25:00 and stored
    And I delete todo 1
    Then the sessions are:
      | status    | todoId | actualMin |
      | completed | 1      | 25        |

  @integration @REQ-4.2
  Scenario: Reorder todos
    Given no database exists yet
    When nunc opens the database
    And I add the todo "first"
    And I add the todo "second"
    And I add the todo "third"
    And I reorder the todos to "2,99"
    And I list the todos
    Then the listed todos are:
      | rawText | position |
      | first   | 0        |
      | second  | 0        |
      | third   | 2        |

  @integration @REQ-4.3
  Scenario: Increment the pomodoro count of a todo
    Given no database exists yet
    When nunc opens the database
    And I add the todo "first"
    And todo 1 gets a pomodoro
    And todo 1 gets a pomodoro
    Then the todo 1 is:
      | field         | value |
      | pomodoroCount | 2     |
