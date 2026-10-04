Feature: Statistics aggregation
  Source: REQUIREMENTS 10; nunc-go stats_test.go

  Background:
    Given today is 2025-03-14
    And the statistics range is 2025-03-10 to 2025-03-16
    And the todos:
      | id | line                              | pomodoros | position |
      | 1  | (A) Write report +work @desk      | 0         | 1        |
      | 2  | Call dentist @phone               | 0         | 2        |
      | 3  | Plan trip +home +holiday @phone   | 0         | 3        |
    And the sessions:
      | start            | mode     | type        | status    | minutes | todo |
      | 2025-02-20 09:00 | pomodoro | work        | completed | 25      | 1    |
      | 2025-02-21 09:00 | pomodoro | work        | completed | 25      |      |
      | 2025-03-10 09:00 | pomodoro | work        | completed | 25      | 1    |
      | 2025-03-10 09:30 | pomodoro | short_break | completed | 5       |      |
      | 2025-03-10 10:00 | pomodoro | work        | completed | 25      | 1    |
      | 2025-03-11 14:00 | pomodoro | work        | aborted   | 10      | 2    |
      | 2025-03-11 15:00 | pomodoro | work        | completed | 25      | 3    |
      | 2025-03-12 09:00 | flowtime | work        | completed | 50      | 1    |
      | 2025-03-12 10:00 | pomodoro | long_break  | skipped   | 3       |      |
      | 2025-03-14 08:00 | pomodoro | work        | completed | 25      | 2    |
      | 2025-03-14 08:30 | pomodoro | short_break | completed | 5       |      |

  @domain @REQ-10
  Scenario: Only completed work sessions count as pomodoros
    When I calculate the statistics
    Then the total pomodoros are 6
    And the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-10 | 2         | 50   | 5     |
      | 2025-03-11 | 1         | 35   | 0     |
      | 2025-03-12 | 0         | 50   | 3     |
      | 2025-03-14 | 1         | 25   | 5     |

  @domain @REQ-10
  Scenario: Aborted and skipped sessions are not counted as pomodoros
    Given the statistics range is 2025-03-11 to 2025-03-12
    When I calculate the statistics
    Then the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-11 | 1         | 35   | 0     |
      | 2025-03-12 | 0         | 50   | 3     |
    And the total pomodoros are 6

  @domain @REQ-10
  Scenario: Break time is summed separately
    Given the statistics range is 2025-03-10 to 2025-03-10
    When I calculate the statistics
    Then the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-10 | 2         | 50   | 5     |

  @domain @REQ-10
  Scenario: Only sessions inside the range are listed, the totals cover all sessions
    Given the statistics range is 2025-03-14 to 2025-03-14
    When I calculate the statistics
    Then the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-14 | 1         | 25   | 5     |
    And the total pomodoros are 6
    And the total work minutes are 210

  @domain @REQ-10
  Scenario: The weekly view groups pomodoros by weekday
    When I calculate the statistics
    Then Mon has 2 pomodoros and 50 work minutes
    And Tue has 1 pomodoros and 35 work minutes
    And Wed has 0 pomodoros and 50 work minutes
    And Thu has 0 pomodoros and 0 work minutes
    And Fri has 1 pomodoros and 25 work minutes
    And Sat has 0 pomodoros and 0 work minutes
    And Sun has 0 pomodoros and 0 work minutes

  @domain @REQ-10
  Scenario: The monthly view groups pomodoros by day
    Given the statistics range is 2025-03-01 to 2025-03-31
    When I calculate the statistics
    Then the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-10 | 2         | 50   | 5     |
      | 2025-03-11 | 1         | 35   | 0     |
      | 2025-03-12 | 0         | 50   | 3     |
      | 2025-03-14 | 1         | 25   | 5     |

  @domain @REQ-10
  Scenario: The yearly view groups pomodoros and hours by month
    Given the statistics range is 2025-01-01 to 2025-12-31
    When I calculate the statistics
    Then the months are:
      | month   | pomodoros | work |
      | 2025-02 | 2         | 50   |
      | 2025-03 | 4         | 160  |

  @domain @REQ-10
  Scenario: Top todos by pomodoro count
    When I calculate the statistics
    Then the top todos are:
      | todo | pomodoros | minutes |
      | 1    | 2         | 100     |
      | 2    | 1         | 35      |
      | 3    | 1         | 25      |

  @domain @REQ-10
  Scenario: Time distribution by project tag
    When I calculate the statistics
    Then the projects are:
      | project  | minutes |
      | +work    | 100     |
      | +holiday | 25      |
      | +home    | 25      |

  @domain @REQ-10
  Scenario: Time distribution by context tag
    When I calculate the statistics
    Then the contexts are:
      | context | minutes |
      | @desk   | 100     |
      | @phone  | 60      |

  @domain @REQ-10
  Scenario: Sessions without a todo have no project or context
    Given the statistics range is 2025-02-21 to 2025-02-21
    When I calculate the statistics
    Then the projects are:
      | project | minutes |
    And the contexts are:
      | context | minutes |
    And the top todos are:
      | todo | pomodoros | minutes |

  @domain @REQ-10
  Scenario: Average pomodoros per day over 7 and 30 days
    When I calculate the statistics
    Then the average over 7 days is about 0.571
    And the average over 30 days is about 0.2

  @domain @REQ-10
  Scenario: The averages cover exactly the last 7 and 30 days including today
    Given pomodoros were completed on "2025-03-14 2025-03-08 2025-03-07 2025-02-13 2025-02-12"
    When I calculate the statistics
    Then the average over 7 days is about 0.286
    And the average over 30 days is about 0.133

  @domain @REQ-10
  Scenario: A todo without a completed pomodoro is not a top todo
    Given the sessions:
      | start            | mode     | type | status  | minutes | todo |
      | 2025-03-10 09:00 | flowtime | work | completed | 50    | 1    |
      | 2025-03-10 11:00 | pomodoro | work | aborted   | 10    | 2    |
    When I calculate the statistics
    Then the top todos are:
      | todo | pomodoros | minutes |
    And the projects are:
      | project | minutes |
      | +work   | 50      |
    And the contexts are:
      | context | minutes |
      | @desk   | 50      |
      | @phone  | 10      |

  @domain @REQ-10
  Scenario: Total focused time over all sessions
    When I calculate the statistics
    Then the total work minutes are 210

  @domain @REQ-10
  Scenario: Days follow the local time zone
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2025-03-10 23:30 | pomodoro | work | completed | 25      |      |
    And the local time zone is UTC+2
    When I calculate the statistics
    Then the daily statistics are:
      | date       | pomodoros | work | break |
      | 2025-03-11 | 1         | 25   | 0     |
    And Tue has 1 pomodoros and 25 work minutes
    And Mon has 0 pomodoros and 0 work minutes

  @domain @REQ-10
  Scenario: Nothing recorded yields zeroed statistics
    Given the sessions:
      | start | mode | type | status | minutes | todo |
    When I calculate the statistics
    Then there are no statistics

  @domain @property @REQ-10
  Scenario: The statistics do not depend on the order of the sessions
    Then the property "the statistics do not depend on the order of the sessions" holds for 150 random cases

  @integration @REQ-10 @wip
  Scenario: An empty database yields zeroed statistics

  @integration @REQ-10 @wip
  Scenario: Daily aggregates

  @integration @REQ-10 @wip
  Scenario: Total completed pomodoros

  @integration @REQ-10 @wip
  Scenario: Date range filtering

  @domain @REQ-10 @wip
  Scenario: Only completed work sessions count as pomodoros

  @domain @REQ-10 @wip
  Scenario: Aborted and skipped sessions are not counted as pomodoros

  @domain @REQ-10 @wip
  Scenario: Break time is summed separately

  @domain @REQ-10 @wip
  Scenario: The weekly view groups pomodoros by weekday

  @domain @REQ-10 @wip
  Scenario: The monthly view groups pomodoros by day

  @domain @REQ-10 @wip
  Scenario: The yearly view groups pomodoros and hours by month

  @domain @REQ-10 @wip
  Scenario: Top todos by pomodoro count

  @domain @REQ-10 @wip
  Scenario: Time distribution by project tag

  @domain @REQ-10 @wip
  Scenario: Time distribution by context tag

  @domain @REQ-10 @wip
  Scenario: Average pomodoros per day over 7 and 30 days

  @domain @REQ-10 @wip
  Scenario: Total focused time over all sessions

  @ui @REQ-10 @wip
  Scenario: Switch between day, week, month and year

  @ui @REQ-10 @wip
  Scenario: Bar, line, heatmap calendar and pie charts render

  @ui @REQ-10 @wip
  Scenario: An empty period shows an empty state
