Feature: Habit progress
  Source: ADR 0028
  The streak counts consecutive due days with a tick. Days that are idle    neither break nor count, an open
  day of today does not break the streak yet. A weekly target counts weeks in which enough days are done.

  @domain @US-HB01
  Scenario Outline: A daily habit counts consecutive days
    Given the schedule "daily"
    And the habit was done on "<done>"
    When I evaluate the habit for 2026-03-04
    Then the current streak is <current> days
    And the best streak is <best>
    And the habit is <today> today

    Examples:
      | done                                              | current | best | today    |
      |                                                   | 0       | 0    | open     |
      | 2026-03-02,2026-03-03,2026-03-04                  | 3       | 3    | done     |
      | 2026-03-02,2026-03-03                             | 2       | 2    | open     |
      | 2026-03-04                                        | 1       | 1    | done     |
      | 2026-03-01,2026-03-02                             | 0       | 2    | open     |
      | 2026-02-26,2026-02-27,2026-02-28,2026-03-03,2026-03-04 | 2  | 3    | done     |
      | 2026-03-04,2026-03-05                             | 1       | 1    | done     |
      | 2026-03-03                                        | 1       | 1    | open     |
      | 2026-03-02,2026-03-04                             | 1       | 1    | done     |

  @domain @US-HB01
  Scenario Outline: A habit on chosen weekdays skips the other days
    Given the schedule "days:1,3,5"
    And the habit was done on "<done>"
    When I evaluate the habit for <today>
    Then the current streak is <current> days
    And the best streak is <best>
    And the habit is <state> today

    Examples:
      | done                                   | today      | current | best | state    |
      | 2026-03-02,2026-03-04                  | 2026-03-04 | 2       | 2    | done     |
      | 2026-02-27,2026-03-02                  | 2026-03-04 | 2       | 2    | open     |
      | 2026-03-03                             | 2026-03-04 | 0       | 0    | open     |
      | 2026-03-02                             | 2026-03-05 | 0       | 1    | idle     |
      | 2026-02-25,2026-02-27,2026-03-02       | 2026-03-05 | 0       | 3    | idle     |
      | 2026-02-25,2026-03-02                  | 2026-03-04 | 1       | 1    | open     |
      | 2026-02-27,2026-03-02,2026-03-03       | 2026-03-04 | 2       | 2    | open     |

  @domain @US-HB01
  Scenario Outline: A weekly target counts weeks
    Given the schedule "weekly:3"
    And the habit was done on "<done>"
    When I evaluate the habit for <today>
    Then the current streak is <current> weeks
    And the best streak is <best>
    And this week has <week_done> of 3
    And the habit is <state> today

    Examples:
      | done                                                    | today      | current | best | week_done | state   |
      | 2026-03-02,2026-03-03,2026-03-04                        | 2026-03-04 | 1       | 1    | 3         | done    |
      | 2026-02-23,2026-02-24,2026-02-25,2026-03-02             | 2026-03-04 | 1       | 1    | 1         | open    |
      | 2026-02-16,2026-02-17,2026-02-18,2026-02-23,2026-02-24,2026-02-25 | 2026-03-04 | 2 | 2 | 0       | open    |
      | 2026-02-09,2026-02-23,2026-02-24,2026-02-25             | 2026-03-04 | 1       | 1    | 0         | open    |
      | 2026-03-02,2026-03-03,2026-03-04                        | 2026-03-05 | 1       | 1    | 3         | idle    |
      |                                                         | 2026-03-04 | 0       | 0    | 0         | open    |
      | 2026-02-16,2026-02-17,2026-02-18,2026-03-02,2026-03-03,2026-03-04 | 2026-03-04 | 1 | 1 | 3     | done    |
      | 2026-02-16,2026-02-17,2026-02-18,2026-02-23,2026-03-02,2026-03-03,2026-03-04 | 2026-03-08 | 1 | 1 | 3 | idle    |
      | 2026-03-01,2026-03-02,2026-03-03                        | 2026-03-04 | 0       | 0    | 2         | open    |

  @domain @US-HB01
  Scenario Outline: The week target follows the schedule
    Given the schedule "<schedule>"
    When I evaluate the habit for 2026-03-04
    Then the week target is <target>

    Examples:
      | schedule   | target |
      | daily      | 7      |
      | days:1,3,5 | 3      |
      | days:2     | 1      |
      | weekly:4   | 4      |

  @domain @US-HB01
  Scenario: Days of an earlier week do not count for this week
    Given the schedule "daily"
    And the habit was done on "2026-03-01,2026-03-04"
    When I evaluate the habit for 2026-03-04
    Then this week has 1 of 7
