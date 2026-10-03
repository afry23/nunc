Feature: Timer persistence
  Source: PLAN 1.4, REQUIREMENTS 13 (timer state survives an app restart)

  @integration @REQ-13 @wip
  Scenario: A running timer is restored after an app restart

  @integration @REQ-13 @wip
  Scenario: A paused timer is restored after an app restart

  @domain @REQ-13 @wip
  Scenario: A timer that expired while the app was closed is completed with its original end time

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
