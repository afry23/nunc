Feature: Todo filtering, search and sorting
  Source: REQUIREMENTS 4.2

  Background:
    Given the todos:
      | id | line                                          | pomodoros | position |
      | 1  | (A) 2025-01-03 Write report +work @desk       | 3         | 3        |
      | 2  | (B) 2025-01-01 Call dentist @phone            | 0         | 1        |
      | 3  | 2025-01-02 Buy milk +home @errands            | 5         | 2        |
      | 4  | (A) Plan trip +holiday +family @home          | 1         | 4        |
      | 5  | x 2025-01-05 2025-01-04 Water plants +home    | 2         | 5        |
      | 6  | Read Rust book +work +learning @desk          | 0         | 6        |

  @domain @REQ-4.2
  Scenario: Filter by project
    Given the filter projects are "+work"
    When I apply the filter
    Then the result ids are "1 6"

  @domain @REQ-4.2
  Scenario: A filter with several projects matches any of them
    Given the filter projects are "+work +holiday"
    When I apply the filter
    Then the result ids are "1 4 6"

  @domain @REQ-4.2
  Scenario: Filter by context
    Given the filter contexts are "@desk"
    When I apply the filter
    Then the result ids are "1 6"

  @domain @REQ-4.2
  Scenario: A filter with several contexts matches any of them
    Given the filter contexts are "@phone @errands"
    When I apply the filter
    Then the result ids are "2 3"

  @domain @REQ-4.2
  Scenario: Filter by priority
    Given the filter priorities are "A"
    When I apply the filter
    Then the result ids are "1 4"

  @domain @REQ-4.2
  Scenario: A filter with several priorities matches any of them
    Given the filter priorities are "A B"
    When I apply the filter
    Then the result ids are "1 2 4"

  @domain @REQ-4.2
  Scenario: Filter by free text
    Given the filter text is "milk"
    When I apply the filter
    Then the result ids are "3"

  @domain @REQ-4.2
  Scenario: Free text ignores case
    Given the filter text is "RUST BOOK"
    When I apply the filter
    Then the result ids are "6"

  @domain @REQ-4.2
  Scenario: Every word of the free text must match
    Given the filter text is "report milk"
    When I apply the filter
    Then the result ids are ""

  @domain @REQ-4.2
  Scenario: Free text also finds tags
    Given the filter text is "+home"
    When I apply the filter
    Then the result ids are "3 5"

  @domain @REQ-4.2
  Scenario: Filter by completion status
    Given the filter shows only completed todos
    When I apply the filter
    Then the result ids are "5"

  @domain @REQ-4.2
  Scenario: Open todos exclude completed ones
    Given the filter shows only open todos
    When I apply the filter
    Then the result ids are "1 2 3 4 6"

  @domain @REQ-4.2
  Scenario: Combined filters are applied together
    Given the filter projects are "+work +home"
    And the filter contexts are "@desk"
    And the filter priorities are "A"
    And the filter text is "report"
    When I apply the filter
    Then the result ids are "1"

  @domain @REQ-4.2
  Scenario: A filter without matches yields an empty list
    Given the filter projects are "+work"
    And the filter contexts are "@phone"
    When I apply the filter
    Then the result ids are ""

  @domain @REQ-4.2
  Scenario: An empty filter keeps all todos in their order
    When I apply the filter
    Then the result ids are "1 2 3 4 5 6"

  @domain @REQ-4.2
  Scenario: Sort by priority
    When I sort by priority in ascending order
    Then the result ids are "1 4 2 3 5 6"

  @domain @REQ-4.2
  Scenario: Sort by priority descending keeps todos without priority last
    When I sort by priority in descending order
    Then the result ids are "2 1 4 3 5 6"

  @domain @REQ-4.2
  Scenario: Sort by creation date
    When I sort by created in ascending order
    Then the result ids are "2 3 1 5 4 6"

  @domain @REQ-4.2
  Scenario: Sort by creation date descending keeps todos without date last
    When I sort by created in descending order
    Then the result ids are "5 1 3 2 4 6"

  @domain @REQ-4.2
  Scenario: Sort by pomodoro count
    When I sort by pomodoros in descending order
    Then the result ids are "3 1 5 4 2 6"

  @domain @REQ-4.2
  Scenario: Sort by pomodoro count ascending
    When I sort by pomodoros in ascending order
    Then the result ids are "2 6 4 5 1 3"

  @domain @REQ-4.2
  Scenario: Sort by manual position
    When I sort by manual in ascending order
    Then the result ids are "2 3 1 4 5 6"

  @domain @REQ-4.2
  Scenario: Sort by manual position descending
    When I sort by manual in descending order
    Then the result ids are "6 5 4 1 3 2"

  @domain @REQ-4.2
  Scenario: Sort alphabetically ignoring case
    When I sort by alphabetical in ascending order
    Then the result ids are "3 2 4 6 5 1"

  @domain @REQ-4.2
  Scenario: Filter and sort in one step
    Given the filter projects are "+work +home"
    When I apply the filter and sort by pomodoros in descending order
    Then the result ids are "3 1 5 6"
