Feature: Pick tasks for today
  Source: docs/ui-umbau-plan.md E3, ADR 0027, ADR 0031
  "Aufgaben auswählen" opens the task stock as a selection. A task is not moved out of its project; it is only
  planned for the day.

  @ui @US-UM03
  Scenario: Only open tasks that are not on the plan yet are offered
    Given the todos:
      | rawText                   | completed |
      | Bericht                   | false     |
      | Einkaufen                 | false     |
      | Steuer plan:2026-03-04    | false     |
      | x 2026-03-03 Erledigt     | true      |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 3    | planned |          | 0     |
    When I open the app at "/pick"
    Then I see the heading "Aufgaben auswählen"
    And I see the buttons "Übernehmen (0)"
    And the field "Für heute auswählen: Bericht" exists
    And the field "Für heute auswählen: Einkaufen" exists
    And the field "Für heute auswählen: Steuer" does not exist
    And the field "Für heute auswählen: Erledigt" does not exist

  @ui @US-UM03
  Scenario: The selected tasks are planned for today
    Given the todos:
      | rawText   |
      | Bericht   |
      | Einkaufen |
      | Steuer    |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app at "/pick"
    And I check "Für heute auswählen: Bericht"
    And I check "Für heute auswählen: Steuer"
    Then I see the buttons "Übernehmen (2)"
    When I click "Übernehmen (2)"
    Then the backend received "plan_todo" with "id" equal to "3"
    And I see the heading "Heute"

  @ui @US-UM03
  Scenario: Nothing is planned without a selection
    Given the todos:
      | rawText |
      | Bericht |
    When I open the app at "/pick"
    And I click "Übernehmen (0)"
    Then the backend did not receive "plan_todo"

  @ui @US-UM03
  Scenario: The offer can be searched
    Given the todos:
      | rawText         |
      | Bericht +work   |
      | Einkaufen       |
    When I open the app at "/pick"
    And I enter "work" into the field "Aufgaben suchen"
    Then the field "Für heute auswählen: Bericht" exists
    And the field "Für heute auswählen: Einkaufen" does not exist

  @ui @US-UM03
  Scenario: The empty day leads to the selection
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app
    And I click "Aufgaben auswählen"
    Then I see the heading "Aufgaben auswählen"

  @ui @US-UM03
  Scenario: The day with a plan offers to add more tasks
    Given the todos:
      | rawText                 |
      | Bericht plan:2026-03-04 |
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the day plan lists:
      | todo | reason  | estimate | spent |
      | 1    | planned |          | 0     |
    When I open the app
    And I click "Aufgaben auswählen"
    Then I see the heading "Aufgaben auswählen"
