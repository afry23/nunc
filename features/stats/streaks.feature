Feature: Streaks
  Source: REQUIREMENTS 10.4; nunc-go stats_streaks_test.go

  @domain @REQ-10.4 @wip
  Scenario: No sessions give no streak

  @domain @REQ-10.4 @wip
  Scenario: A single day today gives a streak of one

  @domain @REQ-10.4 @wip
  Scenario: A single day yesterday still counts as the current streak

  @domain @REQ-10.4 @wip
  Scenario: A single day older than yesterday gives no current streak

  @domain @REQ-10.4 @wip
  Scenario: Consecutive days build a streak

  @domain @REQ-10.4 @wip
  Scenario: A gap breaks the current streak

  @domain @REQ-10.4 @wip
  Scenario: The longest streak can lie in the middle of the history

  @domain @REQ-10.4 @wip
  Scenario: The longest streak equals the current streak at the end

  @ui @REQ-10.4 @wip
  Scenario: The current and the longest streak are shown
