Feature: Subtasks
  Source: ADR 0022

  Background:
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Project"
    And I add the subtask "Part" to todo 1
    And I add the subtask "Detail" to todo 2
    And I add the todo "Other"

  @integration @REQ-4.2
  Scenario: Subtasks are stored below their parent
    Then the todo 2 is below the todo 1
    And the todo 3 is below the todo 2
    And the todo 4 is at the top level

  @integration @REQ-4.2
  Scenario: Completing a parent completes its descendants
    When I complete todo 1
    Then the todo 2 is:
      | field     | value |
      | completed | true  |
    And the todo 3 is:
      | field     | value |
      | completed | true  |
    And the todo 4 is:
      | field     | value |
      | completed | false |

  @integration @REQ-4.2
  Scenario: Reopening a parent keeps its children completed
    When I complete todo 1
    And I uncomplete todo 1
    Then the todo 1 is:
      | field     | value |
      | completed | false |
    And the todo 2 is:
      | field     | value |
      | completed | true  |

  @integration @REQ-4.2
  Scenario: Deleting a parent deletes its descendants
    When I delete todo 1
    Then the todo 1 is marked as deleted
    And the todo 2 is marked as deleted
    And the todo 3 is marked as deleted
    And the todo 4 is not marked as deleted

  @integration @REQ-4.2
  Scenario: Moving a todo under another todo and back to the top
    When I move todo 4 below todo 3
    Then the todo 4 is below the todo 3
    When I move todo 4 to the top level
    Then the todo 4 is at the top level

  @integration @REQ-4.4
  Scenario: The todo.txt export leaves out subtasks
    When I export the todos from the database
    Then the exported text is:
      """
      Project
      Other
      """
