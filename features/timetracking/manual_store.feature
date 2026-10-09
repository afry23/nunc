Feature: Manual time entries in the store
  Source: ADR 0017
  Manual entries are stored as completed flowtime work sessions with the origin "manual".

  @integration @US-TT03
  Scenario: A manual entry is stored as a completed flowtime work session with origin "manual"
    Given device A adds a manual session from 09:00:00 to 10:00:00
    Then on device A the session is a completed flowtime work session with origin "manual"
    And on device A the session has 60 minutes

  @integration @US-TT03
  Scenario: A manual entry counts towards the project of its todo
    Given device A and device B share the todo "Write report +work"
    When device A adds a manual session from 09:00:00 to 10:30:00 for the todo
    Then on device A the project "+work" has 90 minutes
