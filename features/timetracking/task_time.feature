Feature: Time entries of a task
  The details of a task list the completed work sessions booked on it, newest first, with their sum.

  Background:
    Given the todos:
      | id | line        | pomodoros | position |
      | 1  | Write report | 0        | 1        |
      | 2  | Plan trip    | 0        | 2        |
    And the sessions:
      | start            | mode      | type        | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime  | work        | completed | 60      | 1    |
      | 2026-03-03 09:00 | pomodoro  | work        | completed | 25      | 1    |
      | 2026-03-03 10:00 | pomodoro  | short_break | completed | 5       | 1    |
      | 2026-03-03 14:00 | flowtime  | work        | aborted   | 30      | 1    |
      | 2026-03-04 09:00 | stopwatch | work        | completed | 15      | 2    |

  @domain @US-TT01
  Scenario: The entries of a task are listed newest first with their total
    When I list the time entries of todo 1
    Then the total time is 85 minutes
    And the time entries are:
      | start            | minutes | note |
      | 2026-03-03 09:00 | 25      |      |
      | 2026-03-02 09:00 | 60      |      |

  @domain @US-TT01
  Scenario: Breaks, aborted sessions and other tasks do not count
    When I list the time entries of todo 2
    Then the total time is 15 minutes

  @domain @US-TT01
  Scenario: A task without time has no entries
    When I list the time entries of todo 3
    Then the total time is 0 minutes
    And the time entries are:
      | start | minutes | note |

  @domain @US-TT01
  Scenario: The note of a session is shown without surrounding blanks
    Given the session notes:
      | start            | note           |
      | 2026-03-02 09:00 |   Kickoff call |
    When I list the time entries of todo 1
    Then the time entries are:
      | start            | minutes | note        |
      | 2026-03-03 09:00 | 25      |             |
      | 2026-03-02 09:00 | 60      | Kickoff call |
