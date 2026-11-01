Feature: Review report
  Source: ADR 0025
  A review sums up a period from the raw sessions and todos and compares it with the period before.

  Background:
    Given the todos:
      | id | line                                  | pomodoros | position |
      | 1  | Write report +work @desk              | 0         | 1        |
      | 2  | x 2026-03-04 2026-03-02 Plan trip +home | 0       | 2        |
      | 3  | x 2026-02-25 2026-02-20 Old task +work  | 0       | 3        |
    And the sessions:
      | start            | mode     | type        | status    | minutes | todo |
      | 2026-02-23 09:00 | flowtime | work        | completed | 40      | 1    |
      | 2026-03-02 09:00 | flowtime | work        | completed | 60      | 1    |
      | 2026-03-02 10:30 | pomodoro | short_break | completed | 5       |      |
      | 2026-03-03 09:00 | pomodoro | work        | completed | 25      | 1    |
      | 2026-03-03 14:00 | flowtime | work        | aborted   | 30      | 2    |

  @domain @US-RV01
  Scenario: Focus time is summed for the period
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review focus time is 85 minutes

  @domain @US-RV01
  Scenario: Breaks and aborted sessions do not count as focus time
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review has 2 completed and 1 aborted work sessions
    And the review abort rate is 33 percent

  @domain @US-RV01
  Scenario: The period is compared with the period before
    When I build the review from 2026-03-02 to 2026-03-08
    Then the previous period is 2026-02-23 to 2026-03-01
    And the previous focus time is 40 minutes
    And the focus time changed by 45 minutes

  @domain @US-RV01
  Scenario: A calendar month is compared with the previous calendar month
    When I build the review from 2026-03-01 to 2026-03-31
    Then the previous period is 2026-02-01 to 2026-02-28
    And the previous focus time is 40 minutes

  @domain @US-RV01
  Scenario: A custom range is compared with the range of equal length before
    When I build the review from 2026-03-02 to 2026-03-04
    Then the previous period is 2026-02-27 to 2026-03-01

  @domain @US-RV01
  Scenario: Completed todos are listed by completion date in the period
    When I build the review from 2026-03-02 to 2026-03-08
    Then the completed todos are:
      | completed  | text            |
      | 2026-03-04 | Plan trip +home |

  @domain @US-RV01
  Scenario: Time is grouped by project
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review projects are:
      | group | minutes |
      | +work | 85      |

  @domain @US-RV01
  Scenario Outline: A period is derived from a day
    When I ask for the <kind> containing <day>
    Then the period is <from> to <to>

    Examples:
      | kind    | day        | from       | to         |
      | week    | 2026-03-04 | 2026-03-02 | 2026-03-08 |
      | month   | 2026-02-10 | 2026-02-01 | 2026-02-28 |
      | quarter | 2026-05-15 | 2026-04-01 | 2026-06-30 |
      | quarter | 2026-06-30 | 2026-04-01 | 2026-06-30 |
      | quarter | 2026-11-20 | 2026-10-01 | 2026-12-31 |
      | month   | 2026-12-10 | 2026-12-01 | 2026-12-31 |
      | week    | 2026-03-08 | 2026-03-02 | 2026-03-08 |

  @domain @US-RV01
  Scenario: The abort rate is rounded to whole percent
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 10:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 11:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 12:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 13:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 14:00 | flowtime | work | aborted   | 10      | 1    |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review abort rate is 17 percent

  @domain @US-RV01
  Scenario: Time without a project or context is grouped as none
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 20      |      |
      | 2026-03-02 10:00 | flowtime | work | completed | 15      |      |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review projects are:
      | group  | minutes |
      | (none) | 35      |

  @domain @US-RV01
  Scenario: An empty period gives a zero review
    When I build the review from 2026-04-01 to 2026-04-07
    Then the review focus time is 0 minutes
    And the focus time changed by 0 minutes

  @domain @US-RV01
  Scenario: The review exports as Markdown
    When I build the review from 2026-03-02 to 2026-03-08
    And I export the review as Markdown
    Then the Markdown contains "# Review 2026-03-02 – 2026-03-08"
    And the Markdown contains "+45 min"
    And the Markdown contains "+work"
    And the Markdown contains "Plan trip"
