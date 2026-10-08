Feature: Time report across devices and storage
  Source: ADR 0017
  The report is computed from the stored sessions: deleted sessions are gone from it, and sessions of other devices
  count towards the projects of their todos.

  @integration @US-TT01 @wip
  Scenario: Deleted sessions are not counted

  @integration @sync @US-TT01 @wip
  Scenario: Project time of another device appears after the sync
