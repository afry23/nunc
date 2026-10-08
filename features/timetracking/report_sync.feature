Feature: Time report across devices and storage
  Source: ADR 0017
  The report is computed from the stored sessions: deleted sessions are gone from it, and sessions of other devices
  count towards the projects of their todos.

  @integration @US-TT01
  Scenario: Deleted sessions are not counted
    Given device A and device B share the todo "Write report +work"
    And device A adds a manual session from 09:00:00 to 10:00:00 for the todo
    And device A adds a manual session from 11:00:00 to 11:30:00 for the todo
    When device A deletes the session at 12:00:00
    Then on device A the project "+work" has 30 minutes

  @integration @US-TT01
  Scenario: Project time of another device appears after the sync
    Given device A and device B share the todo "Write report +work"
    And device A adds a manual session from 09:00:00 to 10:00:00 for the todo
    When the devices exchange their state
    Then on device B the project "+work" has 60 minutes
