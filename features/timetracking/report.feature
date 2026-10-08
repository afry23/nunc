Feature: Time report
  Source: ADR 0017
  Focus time is summed per project, context or todo and per day from the raw sessions. Breaks do not count.

  @domain @US-TT01 @wip
  Scenario: Work sessions are summed per project and day

  @domain @US-TT01 @wip
  Scenario: Sessions of a todo without a project land in the group "no project"

  @domain @US-TT01 @wip
  Scenario: Breaks do not count towards the time

  @domain @US-TT01 @wip
  Scenario: The report can be grouped by context or by todo

  @domain @US-TT01 @wip
  Scenario: Deleted sessions are not counted

  @domain @US-TT01 @wip
  Scenario: Sessions are assigned to the local day they started on

  @integration @sync @US-TT01 @wip
  Scenario: Project time of another device appears after the sync
