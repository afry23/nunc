Feature: Statistics per timer
  Source: REQUIREMENTS 10; ADR 0018
  Time and count per day are split by timer: pomodoro, flowtime and stopwatch.

  Background:
    Given today is 2025-03-14
    And the statistics range is 2025-03-10 to 2025-03-16

  @domain @REQ-10
  Scenario: A day is split into minutes and counts per timer
    Given the sessions:
      | start            | mode      | type        | status    | minutes | todo |
      | 2025-03-12 09:00 | pomodoro  | work        | completed | 25      |      |
      | 2025-03-12 10:00 | pomodoro  | work        | completed | 25      |      |
      | 2025-03-12 11:00 | flowtime  | work        | completed | 50      |      |
      | 2025-03-12 13:00 | stopwatch | work        | completed | 16      |      |
      | 2025-03-12 14:00 | stopwatch | work        | completed | 4       |      |
      | 2025-03-12 15:00 | pomodoro  | short_break | completed | 5       |      |
    When I calculate the statistics
    Then the daily statistics per timer are:
      | date       | pomodoros | pomodoro min | flowtimes | flowtime min | stopwatches | stopwatch min |
      | 2025-03-12 | 2         | 50           | 1         | 50           | 2           | 20            |
    And the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-12 | 2         | 120  | 5     |

  @domain @REQ-10
  Scenario: An aborted pomodoro counts its minutes but not as a pomodoro
    Given the sessions:
      | start            | mode     | type | status  | minutes | todo |
      | 2025-03-12 09:00 | pomodoro | work | aborted | 10      |      |
    When I calculate the statistics
    Then the daily statistics per timer are:
      | date       | pomodoros | pomodoro min | flowtimes | flowtime min | stopwatches | stopwatch min |
      | 2025-03-12 | 0         | 10           | 0         | 0            | 0           | 0             |

  @domain @REQ-10
  Scenario: A stopwatch session is no pomodoro and does not count for the weekday pomodoros
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo |
      | 2025-03-12 09:00 | stopwatch | work | completed | 16      |      |
    When I calculate the statistics
    Then the total pomodoros are 0
    And the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-12 | 0         | 16   | 0     |

  @domain @REQ-10
  Scenario: Aborted flowtime and stopwatch sessions count their minutes but not as sessions
    Given the sessions:
      | start            | mode      | type | status  | minutes | todo |
      | 2025-03-12 09:00 | flowtime  | work | aborted | 7       |      |
      | 2025-03-12 10:00 | stopwatch | work | aborted | 9       |      |
    When I calculate the statistics
    Then the daily statistics per timer are:
      | date       | pomodoros | pomodoro min | flowtimes | flowtime min | stopwatches | stopwatch min |
      | 2025-03-12 | 0         | 0            | 0         | 7            | 0           | 9             |

  @domain @REQ-10
  Scenario: Break time is not counted for any timer
    Given the sessions:
      | start            | mode      | type        | status    | minutes | todo |
      | 2025-03-12 09:00 | flowtime  | short_break | completed | 5       |      |
      | 2025-03-12 10:00 | stopwatch | long_break  | completed | 9       |      |
    When I calculate the statistics
    Then the daily statistics per timer are:
      | date       | pomodoros | pomodoro min | flowtimes | flowtime min | stopwatches | stopwatch min |
      | 2025-03-12 | 0         | 0            | 0         | 0            | 0           | 0             |
