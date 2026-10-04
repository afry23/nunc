Feature: Timer persistence
  Source: PLAN 1.4, REQUIREMENTS 13 (timer state survives an app restart)

  @integration @REQ-13
  Scenario: A running timer is restored after an app restart
    Given no database exists yet
    When nunc opens the database
    And a pomodoro for todo 4 is started at 09:00:00 and saved
    And the app restarts
    Then the restored timer is running for todo 4
    And 900 seconds remain at 09:10:00

  @integration @REQ-13
  Scenario: A paused timer is restored after an app restart
    Given no database exists yet
    When nunc opens the database
    And a pomodoro for todo 4 is started at 09:00:00 and saved
    And the timer is paused at 09:10:00 and saved
    And the app restarts
    Then the restored timer is paused
    And 900 seconds remain at 11:00:00

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

  @integration @REQ-13
  Scenario Outline: The active timer is removed when it is stopped or completed
    Given no database exists yet
    When nunc opens the database
    And a pomodoro for todo 4 is started at 09:00:00 and saved
    And the timer is <end> at <time> and saved
    And the app restarts
    Then the restored timer is idle

    Examples:
      | end     | time     |
      | stopped | 09:10:00 |
      | expired | 09:30:00 |

  @integration @REQ-13
  Scenario: Finishing a pomodoro stores the session, the count and the idle timer together
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Focus"
    And a pomodoro for todo 1 is started at 09:00:00 and saved
    And the pomodoro is finished at 09:25:00 and stored
    And the app restarts
    Then the restored timer is idle
    And the sessions are:
      | status    | todoId | actualMin | plannedMin |
      | completed | 1      | 25        | 25         |
    And the todo 1 is:
      | field         | value |
      | pomodoroCount | 1     |

  @integration @REQ-13
  Scenario: Starting without an active timer starts idle
    Given no database exists yet
    When nunc opens the database
    And the app restarts
    Then the restored timer is idle

  @integration @REQ-13
  Scenario: The active timer table is added by an additive migration
    Given the Go fixture database
    When nunc opens the database
    Then the table "active_timer" exists
    And the Go tables keep their definition

  @domain @REQ-13 @mobile @wip
  Scenario: Resuming from the background recalculates the remaining time

  @e2e @REQ-13 @mobile @wip
  Scenario: A running timer survives killing the app
