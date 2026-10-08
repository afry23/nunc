Feature: Manual time entries
  Source: ADR 0017
  Forgotten work can be added afterwards. Manual entries are stored as completed flowtime work sessions.

  @domain @US-TT03 @wip
  Scenario: A valid manual entry is accepted

  @domain @US-TT03 @wip
  Scenario Outline: An invalid range is rejected

  @domain @US-TT03 @wip
  Scenario: An entry that overlaps another session is accepted with a warning

  @integration @US-TT03 @wip
  Scenario: A manual entry is stored as a completed flowtime work session with origin "manual"

  @integration @US-TT03 @wip
  Scenario: A manual entry counts towards the project of its todo
