Feature: Responsive layout
  Source: PLAN 1.3, roadmap M4-01, M5-02

  @ui @REQ-responsive
  Scenario: A sidebar is shown from 768 pixels width
    Given the viewport is 768 pixels wide
    When I open the app
    Then I see a sidebar
    And I do not see a bottom tab bar

  @ui @REQ-responsive
  Scenario: A bottom tab bar is shown below 768 pixels width
    Given the viewport is 767 pixels wide
    When I open the app
    Then I see a bottom tab bar
    And I do not see a sidebar

  @ui @REQ-responsive @mobile
  Scenario: Safe areas are respected
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app
    Then the page extends below the display cutout
    And the content keeps clear of the system bars

  # Die Höhe wird aus den Tailwind-Klassen abgeleitet (jsdom hat kein Layout); echte Maße prüft der Maestro-Flow.
  @ui @REQ-responsive @mobile
  Scenario Outline: Touch targets are at least 44 pixels
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the todos:
      | rawText           |
      | (A) Bericht +work |
      | Einkaufen @street |
    When I open the app at "<route>"
    Then every control is at least 44 pixels high

    Examples:
      | route     |
      | /         |
      | /todos    |
      | /stats    |
      | /settings |

  @ui @REQ-responsive @mobile
  Scenario: The todo input stays visible above the keyboard
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the screen is 800 pixels high
    When I open the app at "/todos"
    And I focus the field "Neue Aufgabe"
    And the on-screen keyboard opens with a height of 300 pixels
    Then the field "Neue Aufgabe" is scrolled into view
    And I do not see a bottom tab bar
    When the on-screen keyboard closes
    Then I see a bottom tab bar

  @ui @REQ-responsive @mobile
  Scenario: Pulling the page down reloads the data
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app at "/todos"
    And the todo "Neu im Backend" was added in the backend
    And I pull the page down by 200 pixels
    Then I see "Loslassen zum Aktualisieren"
    When I release the page
    Then the todo list shows "Neu im Backend"

  @ui @REQ-responsive @mobile
  Scenario: A short pull does not reload the data
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app at "/todos"
    And the todo "Neu im Backend" was added in the backend
    And I pull the page down by 40 pixels
    Then I see "Zum Aktualisieren ziehen"
    When I release the page
    Then the todo list is empty

  @ui @REQ-responsive @mobile
  Scenario: Desktop-only settings are hidden on mobile
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I do not see "Symbol im Infobereich (Tray)"
    When I press the key "Space"
    Then the backend did not receive "start_timer"
