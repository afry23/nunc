Feature: Habits
  Source: ADR 0028
  The view "Gewohnheiten" lists every habit with its streak and lets you tick, create, change, archive and delete
  habits. The view "Heute" shows the habits that are due today.

  @ui @US-HB04
  Scenario: The list shows the streak and the week of each habit
    Given the habits are:
      | id | name  | schedule | goal  | done | streak | unit  | weekDone | weekTarget |
      | h1 | Lesen | daily    | check | no   | 3      | days  | 2        | 7          |
      | h2 | Lauf  | weekly:3 | check | no   | 1      | weeks | 1        | 3          |
    When I open the app at "/habits"
    Then I see the heading "Gewohnheiten"
    And I see "Lesen"
    And I see "Serie: 3 Tage"
    And I see "2 von 7 diese Woche"
    And I see "Lauf"
    And I see "Serie: 1 Woche"
    And I see "1 von 3 diese Woche"

  @ui @US-HB04
  Scenario: Ticking a habit sets the tick for today
    Given the habits are:
      | id | name  | schedule | goal  | done | streak | unit | weekDone | weekTarget |
      | h1 | Lesen | daily    | check | no   | 0      | days | 0        | 7          |
    When I open the app at "/habits"
    And I check "Erledigt: Lesen"
    Then the backend received "set_habit_done" with:
      """
      { "id": "h1", "done": true }
      """

  @ui @US-HB04
  Scenario: A habit with a minutes goal is calculated and cannot be ticked
    Given the habits are:
      | id | name      | schedule | goal             | done | streak | unit | weekDone | weekTarget |
      | h1 | Deep Work | daily    | minutes:60:+work | yes  | 2      | days | 2        | 7          |
    When I open the app at "/habits"
    Then I see "Wird aus den Sessions berechnet"
    And I see "Heute erreicht"
    And I do not see the button "Erledigt: Deep Work"

  @ui @US-HB04
  Scenario: A habit from a newer version is listed but not evaluated
    Given the habits are:
      | id | name  | schedule | goal  | done | streak | unit | weekDone | weekTarget |
      | h1 | Neues | moon:1   | check |      |        |      |          |            |
    When I open the app at "/habits"
    Then I see "Neues"
    And I see "Mit einer neueren Version angelegt – wird hier nicht ausgewertet."

  @ui @US-HB04
  Scenario: Without habits the view explains how to start
    When I open the app at "/habits"
    Then I see "Noch keine Gewohnheiten."

  @ui @US-HB04
  Scenario: A new habit is created with a name, a schedule and a goal
    When I open the app at "/habits"
    And I click "Neue Gewohnheit"
    And I enter "Meditieren" into the field "Name der Gewohnheit"
    And I select "Mehrmals pro Woche" in the field "Rhythmus"
    And I enter "4" into the field "Tage pro Woche"
    And I click "Speichern"
    Then the backend received "save_habit" with:
      """
      { "id": null, "name": "Meditieren",
        "schedule": { "kind": "weekly", "perWeek": 4 }, "goal": { "kind": "check" } }
      """

  @ui @US-HB04
  Scenario: A habit with a minutes goal needs a tag
    When I open the app at "/habits"
    And I click "Neue Gewohnheit"
    And I enter "Fokus" into the field "Name der Gewohnheit"
    And I select "Zeit mit einem Tag" in the field "Ziel"
    Then the button "Speichern" is disabled
    When I enter "+work" into the field "Tag (+projekt oder @kontext)"
    And I enter "45" into the field "Minuten pro Tag"
    And I click "Speichern"
    Then the backend received "save_habit" with:
      """
      { "id": null, "name": "Fokus", "schedule": { "kind": "daily" },
        "goal": { "kind": "minutes", "minutes": 45, "tag": "+work" } }
      """

  @ui @US-HB04
  Scenario: Weekdays are chosen with buttons
    When I open the app at "/habits"
    And I click "Neue Gewohnheit"
    And I enter "Sport" into the field "Name der Gewohnheit"
    And I select "An bestimmten Wochentagen" in the field "Rhythmus"
    And I click "Sa"
    And I click "Mo"
    And I click "Speichern"
    Then the backend received "save_habit" with:
      """
      { "schedule": { "kind": "days", "days": [2, 3, 4, 5, 6] } }
      """

  @ui @US-HB04
  Scenario: A habit can be changed
    Given the habits are:
      | id | name  | schedule | goal  | done | streak | unit | weekDone | weekTarget |
      | h1 | Lesen | daily    | check | no   | 0      | days | 0        | 7          |
    When I open the app at "/habits"
    And I click "Bearbeiten: Lesen"
    And I enter "Buch lesen" into the field "Name der Gewohnheit"
    And I click "Speichern"
    Then the backend received "save_habit" with:
      """
      { "id": "h1", "name": "Buch lesen" }
      """

  @ui @US-HB04
  Scenario: Archived habits are listed apart and can be restored
    Given the habits are:
      | id | name  | schedule | goal  | done | streak | unit | weekDone | weekTarget | archived |
      | h1 | Lesen | daily    | check | no   | 0      | days | 0        | 7          | no       |
      | h2 | Alt   | daily    | check | no   | 0      | days | 0        | 7          | yes      |
    When I open the app at "/habits"
    Then I see "Archiviert"
    When I click "Archivieren: Lesen"
    Then the backend received "archive_habit" with:
      """
      { "id": "h1", "archived": true }
      """
    When I click "Wiederherstellen: Alt"
    Then the backend received "archive_habit" with:
      """
      { "id": "h2", "archived": false }
      """

  @ui @US-HB04
  Scenario: Deleting a habit asks first
    Given the habits are:
      | id | name  | schedule | goal  | done | streak | unit | weekDone | weekTarget |
      | h1 | Lesen | daily    | check | no   | 0      | days | 0        | 7          |
    When I open the app at "/habits"
    And I click "Löschen: Lesen"
    Then I see a dialog with the text "„Lesen“ und alle Haken werden gelöscht."
    And the backend did not receive "delete_habit"
    When I click "Löschen"
    Then the backend received "delete_habit" with:
      """
      { "id": "h1" }
      """

  @ui @US-HB04
  Scenario: The view Today lists the habits that are due today
    Given the habits are:
      | id | name  | schedule | goal  | done | streak | unit  | weekDone | weekTarget | open |
      | h1 | Lesen | daily    | check | no   | 0      | days  | 0        | 7          | yes  |
      | h2 | Yoga  | daily    | check | yes  | 1      | days  | 1        | 7          | no   |
      | h3 | Lauf  | days:7   | check | no   | 0      | days  | 0        | 1          | no   |
    When I open the app at "/today"
    Then I see "Gewohnheiten heute"
    And I see "Lesen"
    And I see "Yoga"
    And I do not see "Lauf"
    When I check "Erledigt: Lesen"
    Then the backend received "set_habit_done" with:
      """
      { "id": "h1", "done": true }
      """
