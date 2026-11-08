Feature: Navigation
  Source: REQUIREMENTS 9, PLAN 1.3

  @ui @REQ-9
  Scenario: The areas today, tasks, log and review and the settings are available
    When I open the app
    Then I see the heading "Heute"
    And I see a navigation with the entries "Heute, Aufgaben, Verlauf, Rückblick, Auswertung, Einstellungen, Hilfe"
    When I click "Aufgaben"
    Then I see the heading "Aufgaben"
    When I click "Verlauf"
    Then I see the heading "Verlauf"
    When I click "Rückblick"
    Then I see the heading "Rückblick"
    When I click "Einstellungen"
    Then I see the heading "Einstellungen"
    When I click "Heute"
    Then I see the heading "Heute"

  @ui @REQ-9
  Scenario: The active route is highlighted
    When I open the app
    Then the entry "Heute" is marked as current
    When I click "Aufgaben"
    Then the entry "Aufgaben" is marked as current
    And the entry "Heute" is not marked as current

  @ui @REQ-8 @desktop
  Scenario: The mini timer opens in its own window
    Given the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 754       | 1500  |
    When I open the app at "/mini"
    Then the ring shows "12:34"
    And I do not see a navigation
    When I click "Zur App"
    Then the backend received "show_main_window" with:
      """
      { "route": "/focus" }
      """

  @ui @REQ-8 @desktop
  Scenario: The main window follows a navigation request of the backend
    When I open the app
    And the backend asks the app to open "/log"
    Then I see the heading "Verlauf"

  @ui @REQ-8 @mobile
  Scenario: The back button navigates instead of closing the app
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Aufgaben"
    And I press the back button of the device
    Then I see the heading "Heute"
    And the back button would leave the app

  @ui @REQ-8 @mobile
  Scenario: Switching between tabs does not pile up a back stack
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Aufgaben"
    And I click "Rückblick"
    And I click "Menü"
    And I click "Einstellungen"
    And I press the back button of the device
    Then I see the heading "Heute"
    And the back button would leave the app

  @ui @REQ-8 @mobile
  Scenario: The today tab goes back instead of adding an entry
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Rückblick"
    And I click "Heute"
    Then I see the heading "Heute"
    And the back button would leave the app

  @ui @REQ-8 @desktop
  Scenario: On the desktop every view is a history entry
    Given the platform is "desktop"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Aufgaben"
    And I click "Rückblick"
    And I press the back button of the device
    Then I see the heading "Aufgaben"

  @ui @REQ-responsive
  Scenario: The sidebar can be collapsed to icons and expanded again
    When I open the app
    Then the sidebar is expanded
    When I click "Seitenleiste einklappen"
    Then the sidebar is collapsed
    And the navigation still offers "Heute, Aufgaben, Verlauf, Rückblick, Auswertung, Einstellungen, Hilfe"
    When I click "Aufgaben"
    Then I see the heading "Aufgaben"
    And the sidebar is collapsed
    When I click "Seitenleiste ausklappen"
    Then the sidebar is expanded

  @ui @REQ-responsive
  Scenario: The collapsed sidebar is remembered
    Given the sidebar was collapsed earlier
    When I open the app
    Then the sidebar is collapsed

  @ui @REQ-4.2
  Scenario: A task can be created from the sidebar in any view
    When I open the app
    And I click "Neue Aufgabe"
    Then I see the heading "Neue Aufgabe anlegen"
    When I enter "Bericht +work" into the field "Neue Aufgabe"
    And I click "Hinzufügen"
    Then the backend received "add_todo" with:
      """
      { "rawText": "Bericht +work" }
      """
    And I do not see a dialog
    And I see "Aufgabe angelegt"

  @ui @REQ-4.2
  Scenario: Escape closes the new task dialog without creating a task
    When I open the app
    And I click "Neue Aufgabe"
    And I press the key "Escape" in the field "Neue Aufgabe"
    Then I do not see a dialog
    And the backend did not receive "add_todo"

  @ui @REQ-4.2
  Scenario: The collapsed sidebar still offers the new task button
    Given the sidebar was collapsed earlier
    When I open the app
    And I click "Neue Aufgabe"
    Then I see the heading "Neue Aufgabe anlegen"
