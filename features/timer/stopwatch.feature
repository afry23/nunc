Feature: Stopwatch
  Source: ADR 0018, M11-06 (free time tracking without goal, break or pomodoro count)

  Background:
    Given it is 09:00:00

  @domain @US-B01
  Scenario: The stopwatch counts up without a goal
    When I start a stopwatch session
    And 90 minutes pass
    And the timer ticks
    Then the timer is running
    And 5400 seconds have elapsed

  @domain @US-B01
  Scenario: Stopping records a completed work session in stopwatch mode
    When I start a stopwatch session for todo 4
    And 7 minutes pass
    And I stop the timer
    Then the session finished as stopped
    And the recorded session is completed
    And the recorded session mode is stopwatch
    And the recorded session belongs to todo 4

  @domain @US-B01
  Scenario: No break is suggested however long the stopwatch ran
    When I start a stopwatch session
    And 3 hours pass
    And I stop the timer
    Then no break is suggested
    And nothing starts automatically

  @domain @US-B01
  Scenario: A stopwatch session does not count as a pomodoro of its todo
    When I start a stopwatch session for todo 4
    And 50 minutes pass
    And I stop the timer
    Then no todo gets a pomodoro

  @domain @US-B01
  Scenario: A paused stopwatch does not count the paused time
    When I start a stopwatch session
    And 10 minutes pass
    And I pause the timer
    And 30 minutes pass
    And I resume the timer
    And 5 minutes pass
    And I stop the timer
    Then the recorded session lasted 15 minutes

  @domain @US-B01
  Scenario: A running stopwatch can be saved and restored
    When I start a stopwatch session
    And 5 minutes pass
    And the timer state is saved and restored
    And 5 minutes pass
    And I stop the timer
    Then the recorded session lasted 10 minutes

  @domain @US-B01
  Scenario: A stopwatch session without a todo carries projects, contexts and a note
    When I start a stopwatch session with projects "+meetings, +team" contexts "@office" and note "Weekly sync"
    And 45 minutes pass
    And I stop the timer
    Then the recorded session belongs to no todo
    And the recorded session has projects "+meetings, +team"
    And the recorded session has contexts "@office"
    And the recorded session has the note "Weekly sync"

  @domain @US-B01
  Scenario: Labels are normalised when a session starts
    When I start a stopwatch session with projects "meetings, +meetings, +, team" contexts " office " and note ""
    And 5 minutes pass
    And I stop the timer
    Then the recorded session has projects "+meetings, +team"
    And the recorded session has contexts "@office"

  @domain @US-B01
  Scenario: A session with a todo ignores the labels given at the start
    When I start a stopwatch session for todo 4 with projects "+meetings" contexts "@office" and note ""
    And 5 minutes pass
    And I stop the timer
    Then the recorded session belongs to todo 4
    And the recorded session has projects ""
    And the recorded session has contexts ""
