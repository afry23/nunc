Feature: Flowtime break recommendation
  Source: STORIES US-FT-02, REQUIREMENTS 2.2; nunc-go timer_pure_test.go

  Background:
    Given it is 09:00:00

  @domain @US-FT-02
  Scenario Outline: Break is the floor of worked minutes times the ratio
    Given the flowtime ratio is <ratio>
    And the minimum flow state is 15 minutes
    When I start a flowtime session
    And <worked> minutes pass
    And I stop the timer
    Then the suggested break is <break> minutes

    Examples:
      | ratio | worked | break |
      | 0.2   | 50     | 10    |
      | 0.2   | 14     | 0     |
      | 0.5   | 15     | 7     |
      | 0.1   | 59     | 5     |
      | 0.25  | 40     | 10    |
      | 0.5   | 100    | 50    |

  @domain @US-FT-02
  Scenario: No break is suggested below the minimum flow state
    Given the flowtime ratio is 0.5
    And the minimum flow state is 20 minutes
    When I start a flowtime session
    And 19 minutes pass
    And I stop the timer
    Then the session finished as completed
    And the recorded session is completed
    And no break is suggested
    And nothing starts automatically

  @domain @US-FT-02
  Scenario: A session of exactly the minimum flow state suggests a break
    Given the flowtime ratio is 0.2
    And the minimum flow state is 15 minutes
    When I start a flowtime session
    And 15 minutes pass
    And I stop the timer
    Then the suggested break is a short_break of 3 minutes

  @domain @US-FT-02
  Scenario Outline: Ratio limits 0.1 and 0.5 are accepted and values outside are rejected
    Given the setting "flowtimeRatio" is "<ratio>"
    When I validate the settings
    Then <result>

    Examples:
      | ratio | result                                               |
      | 0.1   | the settings are valid                               |
      | 0.25  | the settings are valid                               |
      | 0.5   | the settings are valid                               |
      | 0.09  | the settings are invalid because of "flowtimeRatio"  |
      | 0.51  | the settings are invalid because of "flowtimeRatio"  |
      | 0     | the settings are invalid because of "flowtimeRatio"  |
      | -0.2  | the settings are invalid because of "flowtimeRatio"  |
      | 1     | the settings are invalid because of "flowtimeRatio"  |

  @domain @US-FT-02
  Scenario Outline: The minimum flow state must be between 1 and 60 minutes
    Given the setting "flowtimeMinFlowStateMin" is "<minutes>"
    When I validate the settings
    Then <result>

    Examples:
      | minutes | result                                                        |
      | 1       | the settings are valid                                        |
      | 15      | the settings are valid                                        |
      | 60      | the settings are valid                                        |
      | 0       | the settings are invalid because of "flowtimeMinFlowStateMin" |
      | 61      | the settings are invalid because of "flowtimeMinFlowStateMin" |

  @domain @US-FT-02
  Scenario: Flowtime counts up instead of down
    When I start a flowtime session
    Then the timer is running
    And 0 seconds have elapsed
    When 20 minutes pass
    Then 1200 seconds have elapsed
    When I pause the timer
    And 10 minutes pass
    Then 1200 seconds have elapsed
    When the timer ticks
    Then no session finished

  @domain @US-FT-02
  Scenario: Stopping a flowtime session completes it and suggests a break
    When I start a flowtime session for todo 3
    And 40 minutes pass
    And I stop the timer
    Then the session finished as completed
    And the recorded session is completed
    And the recorded session mode is flowtime
    And the recorded session has no planned minutes
    And the recorded session has 40 actual minutes
    And the recorded session belongs to todo 3
    And the suggested break is a short_break of 8 minutes
    And todo 3 gets a pomodoro
    And the timer is idle

  @domain @US-FT-02
  Scenario: The suggested break can be started as a countdown
    When I start a flowtime session
    And 40 minutes pass
    And I stop the timer
    And I start the suggested break
    Then the timer is running
    And 480 seconds remain
    And the total is 480 seconds
    When 8 minutes pass
    And the timer ticks
    Then the session finished as completed
    And the recorded session is a short_break session
    And the recorded session has 8 planned minutes

  @domain @US-FT-02
  Scenario: A suggested flowtime break starts automatically when enabled
    Given automatic start of breaks is on
    When I start a flowtime session
    And 30 minutes pass
    And I stop the timer
    Then a break starts automatically

  @domain @US-FT-02
  Scenario: A flowtime break is not started automatically without a suggestion
    Given automatic start of breaks is on
    When I start a flowtime session
    And 5 minutes pass
    And I stop the timer
    Then nothing starts automatically
