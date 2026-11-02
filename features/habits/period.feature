Feature: Habit quota in a period
  Source: ADR 0028
  For a review a habit is counted over a period: how many days it was due, on how many of them it was done and the
  resulting quota. Days before the habit was created and days after today do not count.

  @domain @US-HB05
  Scenario Outline: The due days follow the schedule
    Given the schedule "<schedule>"
    And the habit was done on "<done>"
    When I count the period from <from> to <to> on <today> for a habit created on <created>
    Then the habit was due on <due> days and done on <ticked> days
    And the quota is <rate>

    Examples:
      | schedule | done                                   | from       | to         | today      | created    | due | ticked | rate |
      | daily    | 2026-03-02,2026-03-03,2026-03-05       | 2026-03-02 | 2026-03-08 | 2026-03-08 | 2026-03-01 | 7   | 3      | 43   |
      | daily    | 2026-03-02                             | 2026-03-02 | 2026-03-08 | 2026-03-08 | 2026-03-01 | 7   | 1      | 14   |
      | days:1,3,5 | 2026-03-02,2026-03-04                | 2026-03-02 | 2026-03-08 | 2026-03-08 | 2026-03-01 | 3   | 2      | 67   |
      | days:1,3,5 | 2026-03-02                           | 2026-03-02 | 2026-03-08 | 2026-03-08 | 2026-03-01 | 3   | 1      | 33   |
      | weekly:3 | 2026-03-02,2026-03-03                  | 2026-03-02 | 2026-03-08 | 2026-03-08 | 2026-03-01 | 3   | 2      | 67   |
      | weekly:3 | 2026-03-02                             | 2026-03-02 | 2026-03-15 | 2026-03-15 | 2026-03-01 | 6   | 1      | 17   |
      | weekly:3 | 2026-03-02                             | 2026-03-02 | 2026-03-04 | 2026-03-04 | 2026-03-01 | 2   | 1      | 50   |
      | daily    | 2026-03-04                             | 2026-03-04 | 2026-03-04 | 2026-03-04 | 2026-03-04 | 1   | 1      | 100  |

  @domain @US-HB05
  Scenario: Days after today do not count
    Given the schedule "daily"
    And the habit was done on "2026-03-03,2026-03-06"
    When I count the period from 2026-03-02 to 2026-03-08 on 2026-03-04 for a habit created on 2026-03-01
    Then the habit was due on 3 days and done on 1 days
    And the quota is 33

  @domain @US-HB05
  Scenario: Days before the habit was created do not count
    Given the schedule "daily"
    And the habit was done on "2026-03-01,2026-03-06"
    When I count the period from 2026-03-02 to 2026-03-08 on 2026-03-08 for a habit created on 2026-03-05
    Then the habit was due on 4 days and done on 1 days
    And the quota is 25

  @domain @US-HB05
  Scenario: Done days outside the period do not count
    Given the schedule "daily"
    And the habit was done on "2026-03-01,2026-03-09,2026-03-04"
    When I count the period from 2026-03-02 to 2026-03-08 on 2026-03-10 for a habit created on 2026-03-01
    Then the habit was due on 7 days and done on 1 days
    And the quota is 14

  @domain @US-HB05
  Scenario: The quota never exceeds one hundred percent
    Given the schedule "weekly:1"
    And the habit was done on "2026-03-02,2026-03-03,2026-03-04"
    When I count the period from 2026-03-02 to 2026-03-08 on 2026-03-08 for a habit created on 2026-03-01
    Then the habit was due on 1 days and done on 3 days
    And the quota is 100

  @domain @US-HB05
  Scenario Outline: Without due days there is no quota
    Given the schedule "daily"
    And the habit was done on ""
    When I count the period from <from> to <to> on <today> for a habit created on <created>
    Then the habit was due on 0 days and done on 0 days
    And the quota is none

    Examples:
      | from       | to         | today      | created    |
      | 2026-03-02 | 2026-03-08 | 2026-03-01 | 2026-03-01 |
      | 2026-03-02 | 2026-03-08 | 2026-03-08 | 2026-03-09 |
