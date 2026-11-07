Feature: Recurring tasks
  Source: ADR 0036
  A todo repeats when its description carries rec:[+]<n><d|b|w|m|y>. Completing it creates a follow-up todo with
  the next due date. A leading + counts from the old due date, otherwise from the completion date.

  @domain @US-TP02
  Scenario Outline: The recurrence rule is read from the description
    When I read the recurrence of "<description>"
    Then the recurrence is <rule>

    Examples:
      | description              | rule |
      | Water plants             | none |
      | Water plants rec:1w      | 1w   |
      | Water plants rec:+2m     | +2m  |
      | Report rec:3b due:2026-03-04 | 3b |
      | Report rec:0d            | none |
      | Report rec:1000d         | none |
      | Report rec:1x            | none |
      | Report rec:+             | none |
      | Report rec:d             | none |
      | Report rec:bad rec:1y    | 1y   |

  @domain @US-TP02
  Scenario Outline: The next due date is computed
    Given the task "<description>" is completed on <done>
    Then the follow-up line is "<follow-up>"

    Examples:
      | description                         | done       | follow-up                       |
      | Water rec:1d                        | 2026-03-04 | Water rec:1d due:2026-03-05     |
      | Water rec:+1w due:2026-03-01        | 2026-03-04 | Water rec:+1w due:2026-03-08    |
      | Water rec:1w due:2026-03-01         | 2026-03-04 | Water rec:1w due:2026-03-11     |
      | Water rec:+1w                       | 2026-03-04 | Water rec:+1w due:2026-03-11    |
      | Pay rec:+1m due:2026-01-31          | 2026-02-02 | Pay rec:+1m due:2026-02-28      |
      | Pay rec:1y                          | 2024-02-29 | Pay rec:1y due:2025-02-28       |
      | Mail rec:1b                         | 2026-03-06 | Mail rec:1b due:2026-03-09      |
      | Mail rec:3b                         | 2026-03-04 | Mail rec:3b due:2026-03-09      |
      | Mail rec:1b                         | 2026-03-07 | Mail rec:1b due:2026-03-09      |
      | Water rec:1d plan:2026-03-04        | 2026-03-04 | Water rec:1d due:2026-03-05     |
      | Water                               | 2026-03-04 | none                            |
      | Water rec:0d                        | 2026-03-04 | none                            |

  @domain @US-TP02
  Scenario Outline: A rule is set, replaced and removed
    When I set the recurrence "<rule>" on "<description>"
    Then the description with recurrence is "<result>"

    Examples:
      | description                  | rule | result                       |
      | Water due:2026-03-04         | +1w  | Water due:2026-03-04 rec:+1w |
      | Water rec:1d due:2026-03-04  | +1w  | Water rec:+1w due:2026-03-04 |
      | Water rec:1d due:2026-03-04  | none | Water due:2026-03-04         |
      | Water                        | 0d   | Water                        |

  @domain @US-TP02
  Scenario: The follow-up sync id is stable and depends on the due date
    Then the follow-up sync ids of "abc" for "2026-03-05" are equal
    And the follow-up sync ids of "abc" for "2026-03-05" and "2026-03-06" differ
    And the follow-up sync ids of "abc" and "abd" for "2026-03-05" differ

  @integration @US-TP02
  Scenario: Completing a recurring todo creates the follow-up
    Given no database exists yet
    When nunc opens the database
    And I add the todo "(B) 2026-09-01 Water plants +home rec:+1w due:2026-10-01 plan:2026-10-02"
    And I complete todo 1
    And I list the todos
    Then the listed todos are:
      | rawText                                                                | completed |
      | x 2026-10-04 2026-09-01 Water plants +home rec:+1w due:2026-10-01 plan:2026-10-02 | true      |
      | (B) 2026-10-04 Water plants +home rec:+1w due:2026-10-08               | false     |

  @integration @US-TP02
  Scenario: Completing a todo without a rule creates nothing
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Water plants due:2026-10-01"
    And I complete todo 1
    And I list the todos
    Then there is 1 todo in the list

  @integration @US-TP02
  Scenario: Completing a subtask does not repeat it
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Project"
    And I add the subtask "Part rec:1d" to todo 1
    And I complete todo 2
    And I list the todos
    Then there are 2 todos in the list

  @integration @US-TP02
  Scenario: Completing a parent does not repeat its subtasks
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Project"
    And I add the subtask "Part rec:1d" to todo 1
    And I complete todo 1
    And I list the todos
    Then there are 2 todos in the list

  @integration @US-TP02
  Scenario: Reopening does not remove the follow-up and completing again does not duplicate it
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Water plants rec:1d"
    And I complete todo 1
    And I uncomplete todo 1
    And I complete todo 1
    And I list the todos
    Then there are 2 todos in the list
