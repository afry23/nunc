Feature: Day plan
  Source: ADR 0027
  A todo carries its planning as todo.txt key:value words in the description: due:YYYY-MM-DD is the deadline,
  plan:YYYY-MM-DD the day it is planned for, est:N the estimate in minutes. The day plan lists the open todos that
  are overdue, due today or planned for today or earlier, and compares the estimates with the daily capacity.

  @domain @US-TP01
  Scenario Outline: Planning fields are read from the description
    When I read the planning fields of "<description>"
    Then the due date is <due>
    And the plan date is <plan>
    And the estimate is <estimate>

    Examples:
      | description                                  | due        | plan       | estimate |
      | Write report +work                           | none       | none       | none     |
      | Write report due:2026-03-04                  | 2026-03-04 | none       | none     |
      | Write report plan:2026-03-02 est:45          | none       | 2026-03-02 | 45       |
      | due:2026-03-04 plan:2026-03-03 est:90 Report | 2026-03-04 | 2026-03-03 | 90       |
      | Report due:tomorrow                          | none       | none       | none     |
      | Report due:2026-02-30                        | none       | none       | none     |
      | Report due:2026-3-4                          | none       | none       | none     |
      | Report est:0                                 | none       | none       | none     |
      | Report est:10m                               | none       | none       | none     |
      | Report est:10081                             | none       | none       | none     |
      | Report est:10080                             | none       | none       | 10080    |
      | Report due:2026-03-04 due:2026-03-09         | 2026-03-04 | none       | none     |
      | Report due:bad due:2026-03-09                | 2026-03-09 | none       | none     |
      | Report plan:2026-03-02 plan:2026-03-09 | none | 2026-03-02 | none |
      | Report est:30 est:60 | none | none | 30 |
      | Report est:+30 | none | none | none |
      | Report xdue:2026-03-04                       | none       | none       | none     |

  @domain @US-TP01
  Scenario Outline: A planning field is set, replaced or removed in the description
    When I set "<key>" to "<value>" in "<description>"
    Then the description reads "<result>"

    Examples:
      | description                     | key  | value      | result                          |
      | Write report +work              | due  | 2026-03-04 | Write report +work due:2026-03-04 |
      | Write report due:2026-03-04 +w  | due  | 2026-03-09 | Write report due:2026-03-09 +w  |
      | Write report due:2026-03-04 +w  | due  |            | Write report +w                 |
      | Write report                    | est  | 25         | Write report est:25             |
      | Write report est:25 due:2026-03-04 | est | 50       | Write report est:50 due:2026-03-04 |
      | Write report                    | plan | 2026-03-02 | Write report plan:2026-03-02    |
      | Write report                    | due  | tomorrow   | Write report                    |
      | Write report est:25             | est  | 0          | Write report est:25             |
      | Write report                    | due  |            | Write report                    |
      | due:2026-03-04 due:2026-03-05 x | due  | 2026-03-09 | due:2026-03-09 x                |
      | Report due:bad                  | due  |            | Report due:bad                  |
      |                                 | est  | 15         | est:15                          |

  @domain @US-TP01
  Scenario: The day plan lists what is overdue, due today and planned
    Given today is 2026-03-04 and the capacity is 240 minutes
    And the open todos
      | id | priority | position | description                  |
      | 1  |          | 1        | Planned today plan:2026-03-04 est:60 |
      | 2  |          | 2        | Late report due:2026-03-02 est:30   |
      | 3  |          | 3        | Due today due:2026-03-04            |
      | 4  |          | 4        | Next week due:2026-03-11            |
      | 5  |          | 5        | Backlog item                        |
      | 6  |          | 6        | Rolled over plan:2026-03-02 est:20  |
      | 7  |          | 7        | Tomorrow plan:2026-03-05            |
    When I build the day plan
    Then the plan lists the todos 2, 3, 1, 6
    And the plan reasons are overdue, due, planned, planned
    And the planned estimate is 110 minutes
    And 1 plan entries have no estimate
    And the remaining capacity is 130 minutes

  @domain @US-TP01
  Scenario: Completed todos never appear on the plan
    Given today is 2026-03-04 and the capacity is 240 minutes
    And the open todos
      | id | priority | position | description                       |
      | 1  |          | 1        | Open due:2026-03-04               |
      | 2  |          | 2        | Done due:2026-03-04               |
    And todo 2 is completed
    When I build the day plan
    Then the plan lists the todos 1

  @domain @US-TP01
  Scenario: Within a reason the priority comes first, then the manual order
    Given today is 2026-03-04 and the capacity is 240 minutes
    And the open todos
      | id | priority | position | description              |
      | 1  |          | 1        | First plan:2026-03-04    |
      | 2  | B        | 2        | Second plan:2026-03-04   |
      | 3  | A        | 3        | Third plan:2026-03-04    |
      | 4  | B        | 0        | Fourth plan:2026-03-04   |
    When I build the day plan
    Then the plan lists the todos 3, 4, 2, 1

  @domain @US-TP01
  Scenario: An overfull plan shows a negative remaining capacity
    Given today is 2026-03-04 and the capacity is 60 minutes
    And the open todos
      | id | priority | position | description                  |
      | 1  |          | 1        | Big plan:2026-03-04 est:90   |
    When I build the day plan
    Then the remaining capacity is -30 minutes

  @domain @US-TP01
  Scenario: The plan shows the time already spent on a todo
    Given today is 2026-03-04 and the capacity is 240 minutes
    And the open todos
      | id | priority | position | description                |
      | 1  |          | 1        | Report plan:2026-03-04     |
    And the work sessions
      | todo | status    | minutes |
      | 1    | completed | 25      |
      | 1    | aborted   | 10      |
      | 1    | skipped   | 5       |
      | 2    | completed | 50      |
    When I build the day plan
    Then todo 1 on the plan has spent 35 minutes
