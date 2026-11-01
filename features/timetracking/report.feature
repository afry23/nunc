Feature: Time report
  Source: ADR 0017
  Focus time is summed per project, context or todo and per day from the raw sessions. Breaks do not count.

  Background:
    Given the todos:
      | id | line                            | pomodoros | position |
      | 1  | Write report +work @desk        | 0         | 1        |
      | 2  | Plan trip +home +holiday @phone | 0         | 2        |
      | 3  | Call dentist                    | 0         | 3        |
    And the sessions:
      | start            | mode     | type        | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work        | completed | 60      | 1    |
      | 2026-03-02 10:30 | pomodoro | short_break | completed | 5       |      |
      | 2026-03-03 09:00 | pomodoro | work        | completed | 25      | 1    |
      | 2026-03-03 14:00 | flowtime | work        | aborted   | 30      | 2    |
      | 2026-03-04 08:00 | flowtime | work        | completed | 45      | 3    |
      | 2026-03-04 09:00 | flowtime | work        | completed | 20      |      |

  @domain @US-TT01
  Scenario: Work sessions are summed per project and day
    When I calculate the timesheet from 2026-03-02 to 2026-03-04 grouped by project
    Then the timesheet is:
      | group    | 2026-03-02 | 2026-03-03 | 2026-03-04 | total |
      | +work    | 60         | 25         | 0          | 85    |
      | +holiday | 0          | 30         | 0          | 30    |
      | +home    | 0          | 30         | 0          | 30    |
      | (none)   | 0          | 0          | 65         | 65    |
    And the day totals are "60, 55, 65" and the total is 180

  @domain @US-TT01
  Scenario: Sessions without a project land in the group "(none)"
    When I calculate the timesheet from 2026-03-04 to 2026-03-04 grouped by project
    Then the timesheet is:
      | group  | 2026-03-04 | total |
      | (none) | 65         | 65    |

  @domain @US-TT01
  Scenario: Breaks do not count towards the time
    When I calculate the timesheet from 2026-03-02 to 2026-03-02 grouped by project
    Then the timesheet is:
      | group | 2026-03-02 | total |
      | +work | 60         | 60    |
    And the day totals are "60" and the total is 60

  @domain @US-TT01
  Scenario Outline: The report can be grouped by project, context or todo
    When I calculate the timesheet from 2026-03-02 to 2026-03-04 grouped by <grouping>
    Then the group "<group>" totals <minutes> minutes

    Examples:
      | grouping | group                           | minutes |
      | project  | +home                           | 30      |
      | context  | @desk                           | 85      |
      | context  | @phone                          | 30      |
      | context  | (none)                          | 65      |
      | todo     | Write report +work @desk        | 85      |
      | todo     | Plan trip +home +holiday @phone | 30      |
      | todo     | Call dentist                    | 45      |

  @domain @US-TT01
  Scenario: Sessions are assigned to the local day they started on
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 23:30 | flowtime | work | completed | 30      | 1    |
    And the local time zone is UTC+2
    When I calculate the timesheet from 2026-03-02 to 2026-03-03 grouped by project
    Then the timesheet is:
      | group | 2026-03-02 | 2026-03-03 | total |
      | +work | 0          | 30         | 30    |

  @domain @US-TT01
  Scenario: Sessions outside the range are ignored
    When I calculate the timesheet from 2026-03-03 to 2026-03-03 grouped by project
    Then the day totals are "55" and the total is 55

  @domain @US-TT02
  Scenario: Changing the todo of a session moves its time to another project
    When I assign the session starting "2026-03-02 09:00" to todo 2
    And I calculate the timesheet from 2026-03-02 to 2026-03-02 grouped by project
    Then the timesheet is:
      | group    | 2026-03-02 | total |
      | +holiday | 60         | 60    |
      | +home    | 60         | 60    |

  @domain @US-TT01
  Scenario: Sessions without a todo are grouped by their own projects and contexts
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects        | contexts |
      | 2026-03-02 09:00 | stopwatch | work | completed | 45      |      | +meetings +team | @office  |
      | 2026-03-02 10:00 | stopwatch | work | completed | 30      | 1    | +ignored        | @ignored |
    When I calculate the timesheet from 2026-03-02 to 2026-03-02 grouped by project
    Then the timesheet is:
      | group     | 2026-03-02 | total |
      | +meetings | 45         | 45    |
      | +team     | 45         | 45    |
      | +work     | 30         | 30    |
    And the day totals are "75" and the total is 75

  @domain @US-TT01
  Scenario: Own contexts group the report by context
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02 09:00 | stopwatch | work | completed | 45      |      | +meetings | @office |
    When I calculate the timesheet from 2026-03-02 to 2026-03-02 grouped by context
    Then the timesheet is:
      | group   | 2026-03-02 | total |
      | @office | 45         | 45    |
