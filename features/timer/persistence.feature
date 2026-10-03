Feature: Timer persistence
  Source: PLAN 1.4, REQUIREMENTS 13 (timer state survives an app restart)

  @integration @REQ-13 @wip
  Scenario: A running timer is restored after an app restart

  @integration @REQ-13 @wip
  Scenario: A paused timer is restored after an app restart

  @domain @REQ-13
  Scenario: A timer that expired while the app was closed is completed with its original end time
    Given it is 09:00:00
    When I start a pomodoro for todo 4
    And 10 minutes pass
    And the timer state is saved and restored
    And 3 hours pass
    And the timer ticks
    Then the session finished as completed
    And the recorded session ended at 09:25:00
    And the recorded session has 25 actual minutes
    And the recorded session belongs to todo 4
    And todo 4 gets a pomodoro
    And the timer is idle

  @domain @REQ-13
  Scenario: A paused timer keeps its remaining time across a restart
    Given it is 09:00:00
    When I start a pomodoro
    And 10 minutes pass
    And I pause the timer
    And the timer state is saved and restored
    And 2 hours pass
    Then the timer is paused
    And 900 seconds remain
    When I resume the timer
    And 15 minutes pass
    And the timer ticks
    Then the recorded session ended at 11:25:00

  @domain @REQ-13
  Scenario: A restored suggestion and cycle survive a restart
    Given it is 09:00:00
    When I finish 3 pomodoros in a row
    And the timer state is saved and restored
    And I finish 1 pomodoros in a row
    Then the suggested break is a long_break of 25 minutes

  @integration @REQ-13 @wip
  Scenario: The active timer is removed when it is stopped or completed

  @integration @REQ-13 @wip
  Scenario: Starting without an active timer starts idle

  @integration @REQ-13 @wip
  Scenario: The active timer table is added by an additive migration

  @domain @REQ-13 @mobile @wip
  Scenario: Resuming from the background recalculates the remaining time

  @e2e @REQ-13 @mobile @wip
  Scenario: A running timer survives killing the app
