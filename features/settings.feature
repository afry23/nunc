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

  @integration @REQ-5
  Scenario: Defaults are returned for a fresh database
    Given no database exists yet
    When nunc opens the database
    And I load the settings
    Then the loaded settings are:
      | setting                 | value    |
      | pomodoroMin             | 25       |
      | shortBreakMin           | 5        |
      | longBreakMin            | 25       |
      | longBreakAfter          | 4        |
      | autoStartBreaks         | false    |
      | notificationsEnabled    | true     |
      | soundVolume             | 80       |
      | soundEndPomodoro        | bell     |
      | activeMode              | pomodoro |
      | syncEnabled             | true     |
      | syncProvider            | none     |
      | flowtimeRatio           | 0.2      |
      | flowtimeMinFlowStateMin | 15       |

  @integration @REQ-5
  Scenario: Saved settings are returned again
    Given no database exists yet
    When nunc opens the database
    And I load the settings
    And I set "pomodoroMin" to "30"
    And I set "autoStartBreaks" to "true"
    And I set "flowtimeRatio" to "0.35"
    And I set "activeMode" to "flowtime"
    And I set "syncProvider" to "webdav"
    And I save the settings
    And the app restarts
    And I load the settings
    Then the loaded settings are:
      | setting         | value    |
      | pomodoroMin     | 30       |
      | autoStartBreaks | true     |
      | flowtimeRatio   | 0.35     |
      | activeMode      | flowtime |
      | syncProvider    | webdav   |

  @integration @REQ-5
  Scenario: Saved settings are persisted in the database
    Given no database exists yet
    When nunc opens the database
    And I load the settings
    And I set "pomodoroMin" to "30"
    And I set "autoStartBreaks" to "true"
    And I set "flowtimeRatio" to "0.35"
    And I save the settings
    Then the settings table contains:
      | key                      | value |
      | pomodoro_min             | 30    |
      | auto_start_breaks        | true  |
      | flowtime_ratio           | 0.35  |
      | sound_enabled            | true  |
      | sync_provider            | none  |

  @integration @REQ-5
  Scenario: A value that cannot be read falls back to its default
    Given no database exists yet
    When nunc opens the database
    And the settings table has "pomodoro_min" = "lots"
    And I load the settings
    Then the loaded settings are:
      | setting     | value |
      | pomodoroMin | 25    |

  @integration @REQ-5
  Scenario: Saving settings keeps unrelated keys
    Given the Go fixture database
    When nunc opens the database
    And I load the settings
    And I save the settings
    Then the settings table contains:
      | key       | value            |
      | device_id | fixture-device-a |

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
