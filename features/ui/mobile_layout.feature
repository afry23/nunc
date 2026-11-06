Feature: One structure on phone and desktop
  Source: docs/ui-umbau-plan.md E4, ADR 0031
  The four areas stay the same on every screen size. A phone shows them one after another; a wide window shows
  what belongs together side by side.

  @ui @US-UM04 @mobile
  Scenario: The bottom bar offers the four main areas
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    Then I see a bottom tab bar
    And I see a navigation with the entries "Heute, Aufgaben, Verlauf, Rückblick"

  @ui @US-UM04 @mobile
  Scenario: Settings and help are in the menu on a phone
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Menü"
    Then I see the menu items "Einstellungen, Hilfe"
    When I click "Einstellungen"
    Then I see the heading "Einstellungen"

  @ui @US-UM04
  Scenario: A wide window has no menu button
    Given the viewport is 1024 pixels wide
    When I open the app
    Then I do not see the button "Menü"
    And I see a navigation with the entries "Heute, Aufgaben, Verlauf, Rückblick, Einstellungen, Hilfe"

  @ui @US-UM04 @mobile
  Scenario: Today shows one part at a time on a phone
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the timeline of "2026-03-04" has the sessions:
      | start | end   | type | todo | note | origin |
      | 09:05 | 10:10 | work |      | Abstimmung | timer |
    When I open the app
    Then I see the heading "Heute"
    And I do not see "Tagesnotiz"
    And I do not see "Abstimmung"
    When I click "Tagesverlauf"
    Then I see "Abstimmung"
    When I click "Notizen"
    Then I see "Tagesnotiz"
    When I click "Plan"
    Then I see "Erfasste Arbeit"

  @ui @US-UM04
  Scenario: Today shows plan, log and notes together in a wide window
    Given the viewport is 1024 pixels wide
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    And the timeline of "2026-03-04" has the sessions:
      | start | end   | type | todo | note | origin |
      | 09:05 | 10:10 | work |      | Abstimmung | timer |
    When I open the app
    Then I see "Abstimmung"
    And I see "Tagesnotiz"
    And I see "Erfasste Arbeit"
    And I do not see the button "Notizen"

  @ui @US-UM04
  Scenario: A time entry can be added from the day log of today
    Given the viewport is 1024 pixels wide
    And the day plan for 2026-03-04 has a capacity of 240 minutes
    When I open the app
    And I click "Zeit nachtragen"
    Then I see a dialog with the text "Eintrag"

  @ui @US-UM04 @mobile
  Scenario: The log starts with the list of one day on a phone
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the timeline of today has the sessions:
      | start | end   | type | todo | note       | origin |
      | 09:05 | 10:10 | work |      | Abstimmung | timer  |
      | 10:15 | 10:45 | work |      | Support    | manual |
    When I open the app at "/log"
    Then I see "Abstimmung"
    And I see "Support"
    And I see "Erfasst: 1h 35m"
    And I do not see "Wochenübersicht"

  @ui @US-UM04 @mobile
  Scenario: The log of a phone moves from day to day
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app at "/log"
    And I click "Vorheriger Tag"
    Then the backend received "get_day_timeline"
    And I see "Keine Einträge an diesem Tag."

  @ui @US-UM04 @mobile
  Scenario: The week matrix stays available on a phone
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app at "/log"
    And I click "Woche"
    Then I see the buttons "Vorherige Woche"
    And the backend received "get_timesheet"

  @ui @US-UM04
  Scenario: A wide window shows the week matrix and the day together
    Given the viewport is 1024 pixels wide
    When I open the app at "/log"
    Then the backend received "get_timesheet"
    And I do not see the button "Tag"

  @ui @US-UM04 @mobile
  Scenario: The review sections of a phone fold open on demand
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the review is:
      """
      { "completedTodos": [] }
      """
    When I open the app at "/review"
    Then I do not see "Keine Aufgaben erledigt."
    When I click "Erledigte Aufgaben"
    Then I see "Keine Aufgaben erledigt."

  @ui @US-UM04
  Scenario: The review sections of a wide window are open
    Given the viewport is 1024 pixels wide
    And the review is:
      """
      { "completedTodos": [] }
      """
    When I open the app at "/review"
    Then I see "Keine Aufgaben erledigt."
