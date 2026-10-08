Feature: Edit and delete sessions
  Source: ADR 0017
  Sessions can be corrected or removed afterwards. Every field is a last-writer-wins register, a deletion is a tombstone.

  @domain @US-TT02 @wip
  Scenario: Changing the todo of a session moves its time to another project

  @domain @US-TT02 @wip
  Scenario: Changes to different fields of one session are both kept

  @domain @US-TT02 @wip
  Scenario: A later edit revives a deleted session

  @domain @US-TT02 @wip
  Scenario: A deletion after the last edit stays

  @domain @US-TT02 @wip
  Scenario: Session tombstones older than 90 days are dropped from the sync state

  @domain @property @US-TT02 @wip
  Scenario: Merging session states is commutative, associative and idempotent

  @integration @US-TT02 @wip
  Scenario: Editing a session stamps only the changed fields

  @integration @US-TT02 @wip
  Scenario: Deleting a session hides it from statistics and export

  @integration @US-TT02 @wip
  Scenario: Migration 008 keeps the sessions of an existing database

  @integration @US-TT02 @wip
  Scenario: A backup from an older version can be restored

  @integration @sync @US-TT02 @wip
  Scenario: One device edits a session while another deletes it and both converge

  @integration @sync @US-TT02 @wip
  Scenario: A v2 sync file is read and written back as v3

  @integration @sync @US-TT02 @wip
  Scenario: A client of an older version rejects the v3 file
