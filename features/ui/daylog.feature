Feature: Day log grouping
  Source: docs/ui-umbau-plan.md E4, ADR 0031
  The entries of a day are listed by time. They can also be grouped by project, context or task to see where the
  time went.

  Background:
    Given the day plan for 2026-03-04 has a capacity of 240 minutes
    And the todos:
      | rawText              |
      | Bericht +work @desk  |
    And the timeline of "2026-03-04" has the sessions:
      | start | end   | type        | todo         | note         | origin | projects  | contexts |
      | 09:00 | 10:00 | work        |              | Abstimmung   | timer  | +Produkt  | @meeting |
      | 10:00 | 10:30 | work        |              | Support      | manual | +Plattform | @support |
      | 10:30 | 10:35 | short_break |              |              | timer  |           |          |
      | 11:00 | 11:15 | work        |              | Mittag       | manual |           |          |
      | 13:00 | 13:45 | work        | Bericht +work @desk |       | timer  |           |          |

  @ui @US-UM04
  Scenario: The entries are listed by time without groups
    When I open the app
    Then I see "Abstimmung"
    And I see "Mittag"
    And I do not see "Nicht zugeordnet"
    And I do not see "Ohne Aufgabe"

  @ui @US-UM04
  Scenario: Grouped by project the time per project is shown
    When I open the app
    And I click "Projekt"
    Then I see the group "+Produkt" with "1h"
    And I see the group "+work" with "45m"
    And I see the group "+Plattform" with "30m"
    And I see the group "Nicht zugeordnet" with "15m"

  @ui @US-UM04
  Scenario: Grouped by context the time per context is shown
    When I open the app
    And I click "Kontext"
    Then I see the group "@meeting" with "1h"
    And I see the group "@desk" with "45m"
    And I see the group "@support" with "30m"
    And I see the group "Ohne Kontext" with "15m"

  @ui @US-UM04
  Scenario: Grouped by task the work without a task is collected
    When I open the app
    And I click "Aufgabe"
    Then I see the group "Bericht" with "45m"
    And I see the group "Ohne Aufgabe" with "1h 45m"

  @ui @US-UM04
  Scenario: Breaks are not part of a group
    When I open the app
    And I click "Projekt"
    Then I do not see "Kurze Pause"

  @ui @US-UM04
  Scenario: Going back to the time list removes the groups
    When I open the app
    And I click "Projekt"
    And I click "Zeit"
    Then I do not see the group "Nicht zugeordnet"
    And I see "Abstimmung"

  @ui @US-UM04
  Scenario: An entry of a group can still be edited
    When I open the app
    And I click "Projekt"
    And I click "11:00–11:15 Mittag"
    Then I see a dialog with the text "Eintrag bearbeiten"

  @ui @US-UM04 @mobile
  Scenario: The log of a phone groups the entries of the chosen day too
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the timeline of today has the sessions:
      | start | end   | type | todo | note       | origin |
      | 09:00 | 10:00 | work |      | Abstimmung | timer  |
    When I open the app at "/log"
    And I click "Projekt"
    Then I see the group "Nicht zugeordnet" with "1h"
