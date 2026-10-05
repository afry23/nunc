Feature: Navigation
  Source: REQUIREMENTS 9, PLAN 1.3

  @ui @REQ-9
  Scenario: The routes timer, todos, statistics and settings are available
    When I open the app
    Then I see a navigation with the entries "Timer, Aufgaben, Statistik, Einstellungen, Hilfe"
    When I click "Aufgaben"
    Then I see the heading "Aufgaben"
    When I click "Statistik"
    Then I see the heading "Statistik"
    When I click "Einstellungen"
    Then I see the heading "Einstellungen"
    When I click "Timer"
    Then the ring shows "25:00"

  @ui @REQ-9
  Scenario: The active route is highlighted
    When I open the app
    Then the entry "Timer" is marked as current
    When I click "Aufgaben"
    Then the entry "Aufgaben" is marked as current
    And the entry "Timer" is not marked as current

  @ui @REQ-8 @desktop
  Scenario: The mini timer opens in its own window
    Given the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 754       | 1500  |
    When I open the app at "/mini"
    Then the ring shows "12:34"
    And I do not see a navigation
    When I click "Zur App"
    Then the backend received "show_main_window"

  @ui @REQ-8 @mobile
  Scenario: The back button navigates instead of closing the app
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Aufgaben"
    And I press the back button of the device
    Then the ring shows "25:00"
    And the back button would leave the app

  @ui @REQ-8 @mobile
  Scenario: Switching between tabs does not pile up a back stack
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Aufgaben"
    And I click "Statistik"
    And I click "Einstellungen"
    And I press the back button of the device
    Then the ring shows "25:00"
    And the back button would leave the app

  @ui @REQ-8 @mobile
  Scenario: The timer tab goes back instead of adding an entry
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Statistik"
    And I click "Timer"
    Then the ring shows "25:00"
    And the back button would leave the app

  @ui @REQ-8 @desktop
  Scenario: On the desktop every view is a history entry
    Given the platform is "desktop"
    And the viewport is 390 pixels wide
    When I open the app
    And I click "Aufgaben"
    And I click "Statistik"
    And I press the back button of the device
    Then I see the heading "Aufgaben"

  @ui @REQ-responsive
  Scenario: The sidebar can be collapsed to icons and expanded again
    When I open the app
    Then the sidebar is expanded
    When I click "Seitenleiste einklappen"
    Then the sidebar is collapsed
    And the navigation still offers "Timer, Aufgaben, Statistik, Einstellungen, Hilfe"
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
