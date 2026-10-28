Feature: Subtask tree
  Source: ADR 0022

  Background:
    Given the todos:
      | id | line                                          | pomodoros | position |
      | 1  | (A) 2025-01-03 Write report +work @desk       | 3         | 3        |
      | 2  | (B) 2025-01-01 Call dentist @phone            | 0         | 1        |
      | 3  | 2025-01-02 Buy milk +home @errands            | 5         | 2        |
      | 4  | (A) Plan trip +holiday +family @home          | 1         | 4        |
      | 5  | x 2025-01-05 2025-01-04 Water plants +home    | 2         | 5        |
      | 6  | Read Rust book +work +learning @desk          | 0         | 6        |

  @domain @REQ-4.2
  Scenario: A todo with a parent keeps its parent
    Given the todo 3 is below the todo 1
    When I apply the filter and sort by manual in ascending order
    Then the result ids are "2 3 1 4 5 6"
    And the parent ids are "- 1 - - - -"

  @domain @REQ-4.2
  Scenario: A todo whose parent is missing is shown as a root
    Given the todo 2 is below the todo 99
    When I apply the filter and sort by manual in ascending order
    Then the parent ids are "- - - - - -"

  @domain @REQ-4.2
  Scenario: A parent cycle is shown as roots
    Given the todo 1 is below the todo 2
    And the todo 2 is below the todo 1
    When I apply the filter and sort by manual in ascending order
    Then the parent ids are "- - - - - -"

  @domain @REQ-4.2
  Scenario: A filter hit keeps its ancestors as context
    Given the todo 6 is below the todo 3
    And the filter text is "learning"
    When I apply the filter and sort by manual in ascending order
    Then the result ids are "3 6"
    And the parent ids are "- 3"

  @domain @REQ-4.2
  Scenario: A filter without a hit shows nothing
    Given the todo 6 is below the todo 3
    And the filter text is "nothing like this"
    When I apply the filter
    Then the result ids are ""
