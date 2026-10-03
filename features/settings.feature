Feature: Settings
  Source: REQUIREMENTS 5; nunc-go settings_test.go

  @domain @REQ-5
  Scenario: The default settings are valid
    When I validate the settings
    Then the settings are valid

  @domain @REQ-5
  Scenario Outline: The pomodoro duration must be at least one minute
    Given the setting "pomodoroMin" is "<minutes>"
    When I validate the settings
    Then <result>

    Examples:
      | minutes | result                                         |
      | 1       | the settings are valid                         |
      | 25      | the settings are valid                         |
      | 0       | the settings are invalid because of "pomodoroMin" |

  @domain @REQ-5
  Scenario Outline: The sound volume is limited to 100 percent
    Given the setting "soundVolume" is "<volume>"
    When I validate the settings
    Then <result>

    Examples:
      | volume | result                                           |
      | 0      | the settings are valid                           |
      | 100    | the settings are valid                           |
      | 101    | the settings are invalid because of "soundVolume" |

  @domain @REQ-5
  Scenario: Break lengths and the long break interval may be zero
    Given the setting "shortBreakMin" is "0"
    And the setting "longBreakMin" is "0"
    And the setting "longBreakAfter" is "0"
    When I validate the settings
    Then the settings are valid

  @domain @REQ-5
  Scenario: All violations are reported together
    Given the setting "pomodoroMin" is "0"
    And the setting "soundVolume" is "150"
    When I validate the settings
    Then the settings are invalid because of "pomodoroMin soundVolume"

  @integration @REQ-5 @wip
  Scenario: Defaults are returned for a fresh database

  @integration @REQ-5 @wip
  Scenario: Saved settings are returned again

  @integration @REQ-5 @wip
  Scenario: Saved settings are persisted in the database

  @ui @REQ-5.1 @wip
  Scenario: The timer durations and the long break interval can be changed

  @ui @REQ-5.1 @wip
  Scenario: Auto-start for breaks and the next pomodoro can be toggled

  @ui @REQ-5.2 @wip
  Scenario: Flowtime ratio and minimum flow state can be changed

  @ui @REQ-5.3 @wip
  Scenario: Notifications and sound can be toggled and the volume set

  @ui @REQ-5.3 @wip
  Scenario: The sounds for pomodoro end and break end can be chosen

  @ui @REQ-5.4 @desktop @wip
  Scenario: The system tray setting is shown on desktop

  @ui @REQ-5.4 @mobile @wip
  Scenario: The system tray setting is hidden on mobile
