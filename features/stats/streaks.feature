Feature: Streaks
  Source: REQUIREMENTS 10.4; nunc-go stats_streaks_test.go
  Only completed pomodoros count; the current streak ends today or yesterday.

  Background:
    Given today is 2025-03-14

  @domain @REQ-10.4
  Scenario: No sessions give no streak
    Given pomodoros were completed on ""
    When I calculate the statistics
    Then the current streak is 0
    And the longest streak is 0

  @domain @REQ-10.4
  Scenario: A single day today gives a streak of one
    Given pomodoros were completed on "2025-03-14"
    When I calculate the statistics
    Then the current streak is 1
    And the longest streak is 1

  @domain @REQ-10.4
  Scenario: A single day yesterday still counts as the current streak
    Given pomodoros were completed on "2025-03-13"
    When I calculate the statistics
    Then the current streak is 1
    And the longest streak is 1

  @domain @REQ-10.4
  Scenario: A single day older than yesterday gives no current streak
    Given pomodoros were completed on "2025-03-12"
    When I calculate the statistics
    Then the current streak is 0
    And the longest streak is 1

  @domain @REQ-10.4
  Scenario: Consecutive days build a streak
    Given pomodoros were completed on "2025-03-12 2025-03-13 2025-03-14"
    When I calculate the statistics
    Then the current streak is 3
    And the longest streak is 3

  @domain @REQ-10.4
  Scenario: A gap breaks the current streak
    Given pomodoros were completed on "2025-03-10 2025-03-11 2025-03-13 2025-03-14"
    When I calculate the statistics
    Then the current streak is 2
    And the longest streak is 2

  @domain @REQ-10.4
  Scenario: The longest streak can lie in the middle of the history
    Given pomodoros were completed on "2025-03-01 2025-03-02 2025-03-03 2025-03-04 2025-03-08 2025-03-13 2025-03-14"
    When I calculate the statistics
    Then the current streak is 2
    And the longest streak is 4

  @domain @REQ-10.4
  Scenario: The longest streak equals the current streak at the end
    Given pomodoros were completed on "2025-03-01 2025-03-02 2025-03-12 2025-03-13 2025-03-14"
    When I calculate the statistics
    Then the current streak is 3
    And the longest streak is 3

  @domain @REQ-10.4
  Scenario: Several pomodoros on one day count once
    Given pomodoros were completed on "2025-03-13 2025-03-13 2025-03-14 2025-03-14 2025-03-14"
    When I calculate the statistics
    Then the current streak is 2
    And the longest streak is 2

  @domain @REQ-10.4
  Scenario: Unsorted days are handled
    Given pomodoros were completed on "2025-03-14 2025-03-12 2025-03-13"
    When I calculate the statistics
    Then the current streak is 3

  @domain @REQ-10.4
  Scenario: Streaks run across month and year boundaries
    Given today is 2026-01-02
    And pomodoros were completed on "2025-12-30 2025-12-31 2026-01-01 2026-01-02"
    When I calculate the statistics
    Then the current streak is 4

  @domain @REQ-10.4
  Scenario: Flowtime and aborted sessions do not extend a streak
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2025-03-12 10:00 | pomodoro | work | completed | 25      |      |
      | 2025-03-13 10:00 | flowtime | work | completed | 50      |      |
      | 2025-03-14 10:00 | pomodoro | work | aborted   | 5       |      |
    When I calculate the statistics
    Then the current streak is 0
    And the longest streak is 1

  @domain @property @REQ-10.4
  Scenario: Streaks follow from the set of days
    Then the property "streaks follow from the set of days" holds for 300 random cases

  @ui @REQ-10.4 @wip
  Scenario: The current and the longest streak are shown
