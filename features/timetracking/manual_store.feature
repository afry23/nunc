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

  @integration @US-TT03
  Scenario: A manual entry without a todo counts towards its own project
    Given device A adds a manual session from 09:00:00 to 10:00:00 with projects "+meetings" and contexts "@office"
    Then on device A the project "+meetings" has 60 minutes
    And on device A the session has projects "+meetings" and contexts "@office"

  @integration @US-TT03
  Scenario: Own projects and contexts of a session are kept across devices
    Given device A adds a manual session from 09:00:00 to 10:00:00 with projects "+meetings" and contexts "@office"
    When the devices exchange their state
    Then on device B the session has projects "+meetings" and contexts "@office"
    And on device B the project "+meetings" has 60 minutes

  @integration @US-TT03
  Scenario: The later change of the projects wins across devices
    Given device A adds a manual session from 09:00:00 to 10:00:00 with projects "+meetings" and contexts ""
    And the devices exchange their state
    When device A changes the session projects to "+team" at 12:00:00
    And device B changes the session projects to "+review" at 12:00:05
    And the devices exchange their state
    Then on device A the session has projects "+review" and contexts ""
    And on device B the session has projects "+review" and contexts ""
