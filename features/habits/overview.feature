Feature: Habit overview
  Source: ADR 0028
  The overview lists every habit with its progress and the days it was done recently. Ticked habits take their days
  from the ticks, habits with a minutes goal from the sessions, and a habit written by a newer app is listed but
  not evaluated.

  Background:
    Given the habit day is 2026-03-04

  @domain @US-HB03
  Scenario: A ticked habit shows its progress and recent days
    Given the habits:
      | id | name | schedule | goal  | archived |
      | h1 | Read | daily    | check | no       |
    And the ticks:
      | habit | day        | done |
      | h1    | 2026-03-02 | yes  |
      | h1    | 2026-03-03 | yes  |
      | h1    | 2026-03-04 | yes  |
    When I take the habit overview
    Then the overview lists "Read"
    And the overview of "h1" is done today with a streak of 3
    And the overview of "h1" has the recent days "2026-03-02,2026-03-03,2026-03-04"

  @domain @US-HB03
  Scenario: Removed ticks, ticks of other habits and ticks in the future do not count
    Given the habits:
      | id | name | schedule | goal  | archived |
      | h1 | Read | daily    | check | no       |
      | h2 | Run  | daily    | check | no       |
    And the ticks:
      | habit | day        | done |
      | h1    | 2026-03-03 | no   |
      | h2    | 2026-03-04 | yes  |
      | h1    | 2026-03-06 | yes  |
    When I take the habit overview
    Then the overview of "h1" is open today with a streak of 0
    And the overview of "h1" has the recent days ""
    And the overview of "h2" is done today with a streak of 1

  @domain @US-HB03
  Scenario: A habit with a minutes goal is measured from the sessions
    Given the habits:
      | id | name      | schedule | goal             | archived |
      | h1 | Deep work | daily    | minutes:60:+work | no       |
    And the ticks:
      | habit | day        | done |
      | h1    | 2026-03-04 | yes  |
    And the work sessions:
      | start            | type | status    | minutes | todo | projects | contexts |
      | 2026-03-03T08:00 | work | completed | 60      |      | +work    |          |
      | 2026-03-04T08:00 | work | completed | 59      |      | +work    |          |
    When I take the habit overview
    Then the overview of "h1" is open today with a streak of 1
    And the overview of "h1" has the recent days "2026-03-03"

  @domain @US-HB03
  Scenario: Only the last 35 days are listed as recent days
    Given the habits:
      | id | name | schedule | goal  | archived |
      | h1 | Read | daily    | check | no       |
    And the ticks:
      | habit | day        | done |
      | h1    | 2026-01-28 | yes  |
      | h1    | 2026-01-29 | yes  |
      | h1    | 2026-03-04 | yes  |
    When I take the habit overview
    Then the overview of "h1" has the recent days "2026-01-29,2026-03-04"

  @domain @US-HB03
  Scenario: A habit written by a newer app is listed but not evaluated
    Given the habits:
      | id | name  | schedule | goal  | archived |
      | h1 | Read  | daily    | check | no       |
      | h2 | Newer | moon:1   | check | no       |
      | h3 | Odd   | daily    | orbit | no       |
    When I take the habit overview
    Then the overview lists "Read | Newer | Odd"
    And the overview of "h1" can be evaluated
    And the overview of "h2" cannot be evaluated
    And the overview of "h3" cannot be evaluated

  @domain @US-HB03
  Scenario: Archived habits are listed and marked
    Given the habits:
      | id | name | schedule | goal  | archived |
      | h1 | Read | daily    | check | no       |
      | h2 | Old  | daily    | check | yes      |
    When I take the habit overview
    Then the overview lists "Read | Old (archived)"

  @domain @US-HB03
  Scenario: A habit with a weekly target counts weeks
    Given the habits:
      | id | name | schedule | goal  | archived |
      | h1 | Run  | weekly:2 | check | no       |
      | h2 | Swim | weekly:3 | check | no       |
    And the ticks:
      | habit | day        | done |
      | h1    | 2026-03-02 | yes  |
      | h1    | 2026-03-03 | yes  |
      | h2    | 2026-03-02 | yes  |
      | h2    | 2026-03-03 | yes  |
    When I take the habit overview
    Then the overview of "h1" is idle today with a streak of 1
    And the overview of "h2" is open today with a streak of 0
