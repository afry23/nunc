Feature: Statistics aggregation
  Source: REQUIREMENTS 10; nunc-go stats_test.go

  @integration @REQ-10 @wip
  Scenario: An empty database yields zeroed statistics

  @integration @REQ-10 @wip
  Scenario: Daily aggregates

  @integration @REQ-10 @wip
  Scenario: Total completed pomodoros

  @integration @REQ-10 @wip
  Scenario: Date range filtering

  @domain @REQ-10 @wip
  Scenario: Only completed work sessions count as pomodoros

  @domain @REQ-10 @wip
  Scenario: Aborted and skipped sessions are not counted as pomodoros

  @domain @REQ-10 @wip
  Scenario: Break time is summed separately

  @domain @REQ-10 @wip
  Scenario: The weekly view groups pomodoros by weekday

  @domain @REQ-10 @wip
  Scenario: The monthly view groups pomodoros by day

  @domain @REQ-10 @wip
  Scenario: The yearly view groups pomodoros and hours by month

  @domain @REQ-10 @wip
  Scenario: Top todos by pomodoro count

  @domain @REQ-10 @wip
  Scenario: Time distribution by project tag

  @domain @REQ-10 @wip
  Scenario: Time distribution by context tag

  @domain @REQ-10 @wip
  Scenario: Average pomodoros per day over 7 and 30 days

  @domain @REQ-10 @wip
  Scenario: Total focused time over all sessions

  @ui @REQ-10 @wip
  Scenario: Switch between day, week, month and year

  @ui @REQ-10 @wip
  Scenario: Bar, line, heatmap calendar and pie charts render

  @ui @REQ-10 @wip
  Scenario: An empty period shows an empty state
