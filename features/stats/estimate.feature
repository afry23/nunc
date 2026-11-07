Feature: Estimate versus actual
  Source: ADR 0035
  A task carries an estimate in minutes (est:N) or in pomodoros (estp:N). The review compares it with the effort
  actually spent on the task, in the unit of the estimate: minutes against the focus time, pomodoros against the
  completed pomodoro sessions. A deviation of up to 20 percent of the estimate is on target.

  Background:
    Given the todos:
      | id | line                                          | pomodoros | position |
      | 1  | x 2026-03-04 2026-03-02 Write report est:60   | 0         | 1        |
      | 2  | x 2026-03-05 2026-03-02 Plan trip estp:4      | 0         | 2        |
      | 3  | x 2026-03-05 2026-03-02 Quick fix estp:2      | 0         | 3        |
      | 4  | x 2026-03-06 2026-03-02 No estimate           | 0         | 4        |
      | 5  | x 2026-03-06 2026-03-02 Short call est:60     | 0         | 5        |
      | 6  | Open task est:30                              | 0         | 6        |
    And the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-02-20 09:00 | flowtime | work | completed | 20      | 1    |
      | 2026-03-02 09:00 | flowtime | work | completed | 50      | 1    |
      | 2026-03-02 10:00 | pomodoro | work | completed | 25      | 2    |
      | 2026-03-02 11:00 | pomodoro | work | completed | 25      | 2    |
      | 2026-03-02 12:00 | pomodoro | work | completed | 25      | 2    |
      | 2026-03-02 13:00 | pomodoro | work | completed | 25      | 2    |
      | 2026-03-02 14:00 | pomodoro | work | completed | 25      | 2    |
      | 2026-03-03 09:00 | pomodoro | work | completed | 25      | 3    |
      | 2026-03-03 10:00 | pomodoro | work | aborted   | 10      | 3    |
      | 2026-03-03 11:00 | flowtime | work | completed | 15      | 4    |
      | 2026-03-03 12:00 | flowtime | work | completed | 20      | 5    |

  @domain @US-RV03
  Scenario: Completed tasks show the estimate against the effort
    When I build the review from 2026-03-02 to 2026-03-08
    Then the estimate checks are:
      | text                | estimate | actual | verdict   |
      | Write report est:60 | 60 min   | 70     | on_target |
      | Plan trip estp:4    | 4 pom    | 5      | over      |
      | Quick fix estp:2    | 2 pom    | 1      | under     |
      | No estimate         | none     | none   | none      |
      | Short call est:60   | 60 min   | 20     | under     |

  @domain @US-RV03
  Scenario: The effort counts over the whole life of the task, not only the period
    When I build the review from 2026-03-04 to 2026-03-04
    Then the estimate checks are:
      | text                | estimate | actual | verdict   |
      | Write report est:60 | 60 min   | 70     | on_target |

  @domain @US-RV03
  Scenario: The summary counts the verdicts of the estimated tasks
    When I build the review from 2026-03-02 to 2026-03-08
    Then the estimate summary is 4 estimated, 1 on target, 2 under, 1 over

  @domain @US-RV03
  Scenario Outline: A deviation within 20 percent of the estimate is on target
    When I compare an estimate of "<estimate>" with <minutes> minutes and <pomodoros> pomodoros
    Then the verdict is <verdict> with a deviation of <delta>

    Examples:
      | estimate | minutes | pomodoros | verdict   | delta |
      | 60 min   | 60      | 0         | on_target | 0     |
      | 60 min   | 72      | 0         | on_target | 12    |
      | 60 min   | 73      | 0         | over      | 13    |
      | 60 min   | 48      | 0         | on_target | -12   |
      | 60 min   | 47      | 0         | under     | -13   |
      | 5 pom    | 200     | 6         | on_target | 1     |
      | 5 pom    | 200     | 7         | over      | 2     |
      | 5 pom    | 0       | 3         | under     | -2    |
      | 1 pom    | 0       | 1         | on_target | 0     |
      | 60 min   | 4000000000 | 0      | over      | 2147483647  |
      | 4000000000 min | 0  | 0         | under     | -2147483648 |

  @domain @US-RV03
  Scenario Outline: The estimate is read from the description
    Then the estimate of "<description>" is "<estimate>"

    Examples:
      | description         | estimate |
      | Report              | none     |
      | Report est:45       | 45 min   |
      | Report estp:3       | 3 pom    |
      | Report estp:3 est:45 | 45 min  |
      | Report est:0        | none     |

  @domain @US-RV03
  Scenario: The Markdown export names estimate and effort
    When I build the review from 2026-03-02 to 2026-03-08
    And I export the review as Markdown in English
    Then the Markdown contains "Plan trip estp:4 (125 min) [estimated 4 pomodoros, actual 5 pomodoros]"
    And the Markdown contains "Write report est:60 (50 min) [estimated 60 min, actual 70 min]"
