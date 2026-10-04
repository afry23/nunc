Feature: Pomodoro timer
  Source: REQUIREMENTS 2.1, 3; nunc-go timer_test.go, timer_pure_test.go
  Time is calculated from timestamps (ADR 0004): "N minutes pass" moves the clock, the timer never counts ticks.

  Background:
    Given it is 09:00:00

  @domain @REQ-2.1
  Scenario: Start a pomodoro with the configured duration
    Given the pomodoro lasts 30 minutes
    When I start a pomodoro
    Then the timer is running
    And 1800 seconds remain
    And the total is 1800 seconds

  @domain @REQ-2.1
  Scenario: Starting a timer while one is running is rejected
    When I start a pomodoro
    And I start a pomodoro
    Then the command fails with already_active
    And 1500 seconds remain

  @domain @REQ-2.1
  Scenario: Starting a timer while one is paused is rejected
    When I start a pomodoro
    And I pause the timer
    And I start a flowtime session
    Then the command fails with already_active
    And the timer is paused

  @domain @REQ-2.1
  Scenario: Remaining time is calculated from timestamps
    When I start a pomodoro
    And 7 minutes pass
    Then the timer is running
    And 1080 seconds remain

  @domain @REQ-2.1
  Scenario: Whole seconds are counted, not fractions
    When I start a pomodoro
    And 90 seconds pass
    Then 1410 seconds remain

  @domain @REQ-2.1
  Scenario: A clock that jumps backwards does not add time
    When I start a pomodoro
    And the clock jumps back 5 minutes
    Then 1500 seconds remain

  @domain @REQ-2.1
  Scenario: Pausing freezes the remaining time
    When I start a pomodoro
    And 10 minutes pass
    And I pause the timer
    And 5 minutes pass
    Then the timer is paused
    And 900 seconds remain

  @domain @REQ-2.1
  Scenario: Resuming continues from the paused remaining time
    When I start a pomodoro
    And 10 minutes pass
    And I pause the timer
    And 5 minutes pass
    And I resume the timer
    Then the timer is running
    And 900 seconds remain
    When 5 minutes pass
    Then 600 seconds remain

  @domain @REQ-2.1
  Scenario: Paused time does not count towards the worked time
    When I start a pomodoro
    And 10 minutes pass
    And I pause the timer
    And 60 minutes pass
    And I resume the timer
    And 5 minutes pass
    And I stop the timer
    Then the recorded session has 15 actual minutes
    And the recorded session is completed

  @domain @REQ-2.1
  Scenario: Pausing and resuming in the wrong state is rejected
    When I pause the timer
    Then the command fails with not_active
    When I resume the timer
    Then the command fails with not_active
    When I start a pomodoro
    And I resume the timer
    Then the command fails with not_paused
    When I pause the timer
    And I pause the timer
    Then the command fails with not_running

  @domain @REQ-2.1
  Scenario: Stopping while the timer is idle is rejected
    When I stop the timer
    Then the command fails with not_active

  @domain @REQ-2.1
  Scenario: Stopping before 50 percent records an aborted session
    When I start a pomodoro
    And 749 seconds pass
    And I stop the timer
    Then the session finished as stopped
    And the recorded session is aborted
    And the recorded session is a work session
    And the recorded session has 12 actual minutes
    And the recorded session has 25 planned minutes
    And the recorded session started at 09:00:00
    And the recorded session ended at 09:12:29
    And the timer is idle

  @domain @REQ-2.1
  Scenario: Stopping at or after 50 percent records a completed session
    When I start a pomodoro
    And 750 seconds pass
    And I stop the timer
    Then the session finished as stopped
    And the recorded session is completed
    And the recorded session has 12 actual minutes

  @domain @REQ-2.1
  Scenario: Stopping a pomodoro early neither counts for the todo nor suggests a break
    When I start a pomodoro for todo 7
    And 20 minutes pass
    And I stop the timer
    Then the recorded session is completed
    And the recorded session belongs to todo 7
    And no todo gets a pomodoro
    And no break is suggested
    And nothing starts automatically

  @domain @REQ-2.1
  Scenario: A pomodoro completes automatically when the time is up
    When I start a pomodoro
    And 25 minutes pass
    And the timer ticks
    Then the session finished as completed
    And the recorded session is completed
    And the recorded session has 25 actual minutes
    And the recorded session has 25 planned minutes
    And the recorded session ended at 09:25:00
    And the timer is idle

  @domain @REQ-2.1
  Scenario: A running timer does not complete before the time is up
    When I start a pomodoro
    And 1499 seconds pass
    And the timer ticks
    Then no session finished
    And the timer is running
    And 1 seconds remain

  @domain @REQ-2.1
  Scenario: A paused timer does not complete
    When I start a pomodoro
    And 10 minutes pass
    And I pause the timer
    And 120 minutes pass
    And the timer ticks
    Then no session finished
    And the timer is paused

  @domain @REQ-2.1
  Scenario: A completed pomodoro increments the pomodoro count of the selected todo
    When I start a pomodoro for todo 7
    And 25 minutes pass
    And the timer ticks
    Then todo 7 gets a pomodoro
    And the recorded session belongs to todo 7

  @domain @REQ-2.1
  Scenario: A pomodoro without a selected todo is recorded without a todo
    When I start a pomodoro
    And 25 minutes pass
    And the timer ticks
    Then the recorded session belongs to no todo
    And no todo gets a pomodoro

  @domain @REQ-2.1
  Scenario: A short break is suggested after a completed pomodoro
    Given the short break lasts 6 minutes
    When I start a pomodoro
    And 25 minutes pass
    And the timer ticks
    Then the suggested break is a short_break of 6 minutes

  @domain @REQ-2.1
  Scenario Outline: A long break is suggested after the configured number of pomodoros
    Given the short break lasts 5 minutes
    And the long break lasts 20 minutes and comes after every 3 pomodoros
    When I finish <count> pomodoros in a row
    Then the suggested break is a <type> of <minutes> minutes

    Examples:
      | count | type        | minutes |
      | 1     | short_break | 5       |
      | 2     | short_break | 5       |
      | 3     | long_break  | 20      |
      | 4     | short_break | 5       |
      | 6     | long_break  | 20      |

  @domain @REQ-2.1
  Scenario: Without a long break interval only short breaks are suggested
    Given the long break lasts 20 minutes and comes after every 0 pomodoros
    When I finish 4 pomodoros in a row
    Then the suggested break is a short_break of 5 minutes

  @domain @REQ-2.1
  Scenario: Stopping a pomodoro early does not advance the long break cycle
    Given the long break lasts 20 minutes and comes after every 4 pomodoros
    When I start a pomodoro
    And 20 minutes pass
    And I stop the timer
    And I finish 3 pomodoros in a row
    Then the suggested break is a short_break of 5 minutes

  @domain @REQ-2.1
  Scenario: A break length of zero minutes suggests no break
    Given the short break lasts 0 minutes
    When I finish 1 pomodoros in a row
    Then no break is suggested

  @domain @REQ-2.1
  Scenario: Starting a break without a preceding work session is rejected
    When I start the suggested break
    Then the command fails with no_break_suggested
    And the timer is idle

  @domain @REQ-2.1
  Scenario: The suggested break runs as a countdown and completes
    Given the short break lasts 5 minutes
    When I finish 1 pomodoros in a row
    And I start the suggested break
    Then the timer is running
    And 300 seconds remain
    When 5 minutes pass
    And the timer ticks
    Then the session finished as completed
    And the recorded session is completed
    And the recorded session is a short_break session
    And the recorded session belongs to no todo
    And no todo gets a pomodoro
    And no break is suggested
    And the timer is idle

  @domain @REQ-2.1
  Scenario: Ending a break early records it as skipped
    When I finish 1 pomodoros in a row
    And I start the suggested break
    And 2 minutes pass
    And I stop the timer
    Then the session finished as stopped
    And the recorded session is skipped
    And the recorded session is a short_break session
    And the recorded session has 2 actual minutes
    When I start the suggested break
    Then the command fails with no_break_suggested

  @domain @REQ-2.1
  Scenario: Starting a new work session discards the break suggestion
    When I finish 1 pomodoros in a row
    And I start a pomodoro
    And I stop the timer
    And I start the suggested break
    Then the command fails with no_break_suggested

  @domain @REQ-2.1
  Scenario: The break follows the mode of the last work session
    When I start a flowtime session
    And 20 minutes pass
    And I stop the timer
    And I start the suggested break
    And 4 minutes pass
    And the timer ticks
    Then the recorded session mode is flowtime

  @domain @REQ-2.1
  Scenario: Auto-start settings start the break and the next pomodoro automatically
    Given automatic start of breaks is on
    And automatic start of the next pomodoro is on
    When I start a pomodoro for todo 7
    And 25 minutes pass
    And the timer ticks
    Then a break starts automatically
    When I start the suggested break
    And 5 minutes pass
    And the timer ticks
    Then a pomodoro session starts automatically for todo 7

  @domain @REQ-2.1
  Scenario: Without the auto-start settings nothing starts by itself
    When I start a pomodoro
    And 25 minutes pass
    And the timer ticks
    Then nothing starts automatically
    When I start the suggested break
    And 5 minutes pass
    And the timer ticks
    Then nothing starts automatically

  @domain @REQ-2.1
  Scenario: Auto-start of breaks needs a suggested break
    Given automatic start of breaks is on
    And the short break lasts 0 minutes
    When I start a pomodoro
    And 25 minutes pass
    And the timer ticks
    Then nothing starts automatically

  @domain @property @REQ-2.1
  Scenario: Random commands keep the timer consistent
    Then the property "random commands keep the timer consistent" holds for 150 random cases

  @e2e @REQ-2.1 @desktop @wip
  Scenario: Run a short pomodoro and see it in the statistics
