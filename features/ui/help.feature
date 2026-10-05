Feature: Help
  Source: REQUIREMENTS 9

  @ui @REQ-9
  Scenario: The help view explains the main areas
    When I open the app
    And I click "Hilfe"
    Then I see the heading "Hilfe"
    And I see the heading "Timer"
    And I see the heading "Aufgaben"
    And I see the heading "Synchronisierung"

  @ui @REQ-9 @desktop
  Scenario: The help view lists the keyboard shortcuts
    When I open the app at "/help"
    Then I see the heading "Tastenkürzel"
    And I see "Timer starten, pausieren, fortsetzen"

  @ui @REQ-9 @desktop
  Scenario: The question mark key opens the help
    When I open the app
    And I press the key "?"
    Then I see the heading "Hilfe"

  @ui @REQ-9 @desktop
  Scenario: The question mark key does nothing while typing
    When I open the app at "/todos"
    And I press the key "?" in the field "Neue Aufgabe"
    Then I see the heading "Aufgaben"

  @ui @REQ-9 @mobile
  Scenario: The help view has no keyboard shortcuts on mobile
    Given the platform is "mobile"
    When I open the app at "/help"
    Then I see the heading "Hilfe"
    And I do not see "Tastenkürzel"
