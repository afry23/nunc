Feature: Statistics from stored sessions
  The store hands raw sessions and todos to nunc-core; nothing is aggregated in the database.

  @integration @REQ-10
  Scenario: Statistics are computed from the Go fixture sessions
    Given the Go fixture database
    When nunc opens the database
    And I calculate the statistics for 2026-09-01 to 2026-09-30 as of 2026-09-30
    Then the statistics are:
      | field          | value |
      | totalPomodoros | 5     |
      | totalWorkMin   | 280   |
    And the top todo is 1 with 3 pomodoros

  @integration @REQ-10
  Scenario: Sessions of a deleted todo still count for that todo
    Given the Go fixture database
    When nunc opens the database
    And I delete todo 4
    And I calculate the statistics for 2026-09-01 to 2026-09-30 as of 2026-09-30
    Then the statistics are:
      | field          | value |
      | totalPomodoros | 5     |
    And the top todos include 4 with 2 pomodoros

  @integration @REQ-10
  Scenario: A new database has no statistics
    Given no database exists yet
    When nunc opens the database
    And I calculate the statistics for 2026-09-01 to 2026-09-30 as of 2026-09-30
    Then the statistics are:
      | field          | value |
      | totalPomodoros | 0     |
      | totalWorkMin   | 0     |
