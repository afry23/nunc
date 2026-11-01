Feature: Stopwatch sessions in the store
  Source: ADR 0018
  A stopwatch session is stored as a completed flowtime work session with the kind "stopwatch".

  @integration @US-B01
  Scenario: A stopwatch session is stored as flowtime with the kind "stopwatch"
    Given device A records a stopwatch session from 09:00:00 to 10:00:00
    Then on device A the session is stored as mode "flowtime" with kind "stopwatch"
    And on device A the listed session is a stopwatch session of 60 minutes

  @integration @US-B01
  Scenario: A stopwatch session keeps its kind on another device
    Given device A records a stopwatch session from 09:00:00 to 10:00:00
    When the devices exchange their state
    Then on device B the session is stored as mode "flowtime" with kind "stopwatch"
    And on device B the listed session is a stopwatch session of 60 minutes

  @integration @US-B01
  Scenario: A stopwatch session counts as work time but not as a pomodoro
    Given device A records a stopwatch session from 09:00:00 to 10:00:00
    Then on device A the day has 60 minutes of work and 0 pomodoros

  @integration @US-TT03
  Scenario: A manual entry is listed as a stopwatch session
    Given device A adds a manual session from 09:00:00 to 10:00:00
    Then on device A the listed session is a stopwatch session of 60 minutes

  @integration @US-TT03
  Scenario: A manual entry is a stopwatch session on another device too
    Given device A adds a manual session from 09:00:00 to 10:00:00
    When the devices exchange their state
    Then on device B the listed session is a stopwatch session of 60 minutes

  @integration @US-B01
  Scenario: A stopwatch session without a todo keeps its projects, contexts and note on another device
    Given device A records a stopwatch session from 09:00:00 to 09:45:00 with projects "+meetings +team" contexts "@office" and note "Weekly"
    When the devices exchange their state
    Then on device B the session has projects "+meetings +team" and contexts "@office"
    And on device B the session note is "Weekly"
    And on device B the project "+team" has 45 minutes
