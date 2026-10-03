Feature: Pomodoro timer
  Source: REQUIREMENTS 2.1, 3; nunc-go timer_test.go, timer_pure_test.go

  @domain @REQ-2.1 @wip
  Scenario: Start a pomodoro with the configured duration

  @domain @REQ-2.1 @wip
  Scenario: Starting a timer while one is running is rejected

  @domain @REQ-2.1 @wip
  Scenario: Remaining time is calculated from timestamps

  @domain @REQ-2.1 @wip
  Scenario: Pausing freezes the remaining time

  @domain @REQ-2.1 @wip
  Scenario: Resuming continues from the paused remaining time

  @domain @REQ-2.1 @wip
  Scenario: Paused time does not count towards the worked time

  @domain @REQ-2.1 @wip
  Scenario: Stopping while the timer is idle is rejected

  @domain @REQ-2.1 @wip
  Scenario: Stopping before 50 percent records an aborted session

  @domain @REQ-2.1 @wip
  Scenario: Stopping at or after 50 percent records a completed session

  @domain @REQ-2.1 @wip
  Scenario: A pomodoro completes automatically when the time is up

  @domain @REQ-2.1 @wip
  Scenario: A completed pomodoro increments the pomodoro count of the selected todo

  @domain @REQ-2.1 @wip
  Scenario: A pomodoro without a selected todo is recorded without a todo

  @domain @REQ-2.1 @wip
  Scenario: A short break is suggested after a completed pomodoro

  @domain @REQ-2.1 @wip
  Scenario: A long break is suggested after the configured number of pomodoros

  @domain @REQ-2.1 @wip
  Scenario: Starting a break without a preceding work session is rejected

  @domain @REQ-2.1 @wip
  Scenario: Ending a break early records it as skipped

  @domain @REQ-2.1 @wip
  Scenario: Auto-start settings start the break and the next pomodoro automatically

  @ui @REQ-3 @wip
  Scenario: The ring shows the remaining time as MM:SS

  @ui @REQ-3 @wip
  Scenario: The ring color depends on the session type

  @ui @REQ-3 @wip
  Scenario: A running timer pulses and a paused timer does not

  @ui @REQ-2.1 @wip
  Scenario: Start, pause, resume and stop buttons reflect the timer state

  @ui @REQ-4.3 @wip
  Scenario: A todo can be selected before starting the timer

  @ui @REQ-2.1 @wip
  Scenario: A break suggestion dialog appears after a completed pomodoro

  @ui @REQ-13 @wip
  Scenario: The remaining time is announced to screen readers

  @e2e @REQ-2.1 @desktop @wip
  Scenario: Run a short pomodoro and see it in the statistics
