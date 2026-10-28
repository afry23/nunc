Feature: Settings view
  Source: REQUIREMENTS 5; durations, auto-start, sound, tray

  @ui @REQ-5.1
  Scenario: The timer durations and the long break interval can be changed
    When I open the app at "/settings"
    And I enter "30" into the field "Arbeitszeit"
    And I enter "10" into the field "Kurze Pause"
    And I enter "20" into the field "Lange Pause"
    And I enter "3" into the field "Lange Pause nach"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "pomodoroMin": 30, "shortBreakMin": 10, "longBreakMin": 20, "longBreakAfter": 3 } }
      """
    And I see "Einstellungen gespeichert."

  @ui @REQ-5.1
  Scenario: Auto-start for breaks and the next pomodoro can be toggled
    When I open the app at "/settings"
    And I switch on "Pausen automatisch starten"
    And I switch on "Nächsten Pomodoro automatisch starten"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "autoStartBreaks": true, "autoStartNextPomodoro": true } }
      """

  @ui @REQ-5.2
  Scenario: Flowtime ratio and minimum flow state can be changed
    When I open the app at "/settings"
    And I set the slider "Arbeit-zu-Pause-Verhältnis" to "0.3"
    And I enter "20" into the field "Minimale Flow-Zeit"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "flowtimeRatio": 0.3, "flowtimeMinFlowStateMin": 20 } }
      """

  @ui @REQ-5.3
  Scenario: Notifications and sound can be toggled and the volume set
    When I open the app at "/settings"
    And I switch off "Benachrichtigungen"
    And I switch off "Signalton"
    Then the slider "Lautstärke: 80 %" is disabled
    When I switch on "Signalton"
    And I set the slider "Lautstärke: 80 %" to "40"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "notificationsEnabled": false, "soundEnabled": true, "soundVolume": 40 } }
      """

  @ui @REQ-5.3
  Scenario: The sounds for pomodoro end and break end can be chosen
    When I open the app at "/settings"
    And I select "Digital" in the field "Ton am Ende eines Pomodoros"
    And I select "Glocke" in the field "Ton am Ende einer Pause"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "soundEndPomodoro": "digital", "soundEndBreak": "bell" } }
      """

  @ui @REQ-5.4 @desktop
  Scenario: The system tray setting is shown on desktop
    Given the platform is "desktop"
    When I open the app at "/settings"
    Then I see "Symbol im Infobereich (Tray)"

  @ui @REQ-5.4 @mobile
  Scenario: The system tray setting is hidden on mobile
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I do not see "Symbol im Infobereich (Tray)"

  @ui @REQ-5.4
  Scenario: The interface language can be switched to English and back
    When I open the app at "/settings"
    And I select "English" in the field "Sprache"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "language": "en" } }
      """
    And I see "Configuration"
    And I see "Settings saved."
    When I select "Deutsch" in the field "Language"
    And I click "Save"
    Then I see "Konfiguration"

  @ui @REQ-5.4
  Scenario: The settings show the running app version
    When I open the app at "/settings"
    Then I see "Version 1.2.3"
