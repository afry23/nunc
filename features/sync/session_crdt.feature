Feature: Session CRDT sync
  Source: ADR 0017
  Sessions carry a stable id and merge field by field like todos. Version 2 files are still read.

  @domain @US-TT02
  Scenario: A session from a version 2 file gets the same id on every read
    When a version 2 sync file with a session started at "2026-03-01T10:00:00Z" on device A is read twice
    Then the state has 1 session
    And both readings give the same session id

  @domain @US-TT02
  Scenario: A version 2 file is written back as version 3
    When a version 2 sync file is read and written again
    Then the written file has format 3

  @integration @sync @US-TT02 @wip
  Scenario: Two devices that both know an old session do not create a duplicate
