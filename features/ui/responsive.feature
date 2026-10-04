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

  @ui @REQ-responsive @mobile @wip
  Scenario: Safe areas are respected

  @ui @REQ-responsive @mobile @wip
  Scenario: Touch targets are at least 44 pixels

  @ui @REQ-responsive @mobile @wip
  Scenario: The todo input stays visible above the keyboard

  @ui @REQ-responsive @mobile
  Scenario: Desktop-only settings are hidden on mobile
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I do not see "Symbol im Infobereich (Tray)"
    When I press the key "Space"
    Then the backend did not receive "start_timer"
