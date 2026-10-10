Feature: Edit and delete sessions
  Source: ADR 0017
  Sessions can be corrected or removed afterwards. Every field is a last-writer-wins register, a deletion is a tombstone.

  @domain @US-TT02
  Scenario: Changes to different fields of one session are both kept
    Given a session "s1" on both devices
    And device A sets the session minutes to 30 at 10:00:00
    And device B sets the session note to "review" at 10:00:05
    When the states are merged in either order
    Then the merged session has 30 minutes
    And the session note is "review"

  @domain @US-TT02
  Scenario: The later change wins when both devices change the same field
    Given a session "s1" on both devices
    And device A sets the session minutes to 30 at 10:00:00
    And device B sets the session minutes to 45 at 10:00:05
    When the states are merged in either order
    Then the merged session has 45 minutes

  @domain @US-TT02
  Scenario: Assigning a session to a todo is kept across devices
    Given a session "s1" on both devices
    And device A assigns the session to todo "t-1" at 10:00:00
    When the states are merged in either order
    Then the session belongs to todo "t-1"

  @domain @US-TT02
  Scenario: A later edit revives a deleted session
    Given a session "s1" on both devices
    And device A deletes the session at 10:00:00
    And device B sets the session note to "still needed" at 10:00:05
    When the states are merged in either order
    Then the session exists

  @domain @US-TT02
  Scenario: A deletion after the last edit stays
    Given a session "s1" on both devices
    And device B sets the session note to "typo" at 10:00:00
    And device A deletes the session at 10:00:05
    When the states are merged in either order
    Then the session is deleted

  @domain @US-TT02
  Scenario Outline: Session tombstones are dropped after 90 days
    Given an old session "s1" on both devices
    And device A deleted the session <days> days ago
    When the garbage is collected
    Then the session is <outcome> in the state

    Examples:
      | days | outcome |
      | 10   | kept    |
      | 90   | kept    |
      | 91   | dropped |

  @domain @US-TT02
  Scenario: A live session is never dropped, however old
    Given an old session "s1" on both devices
    When the garbage is collected 400 days later
    Then the session is kept in the state

  @domain @US-TT02
  Scenario: A deletion that an edit overruled is no tombstone
    Given an old session "s1" on both devices
    And device A deleted the session 200 days ago
    And device B changed the session 150 days ago
    When the garbage is collected
    Then the session is kept in the state

  @domain @property @US-TT02
  Scenario: Merging session states is commutative, associative and idempotent
    Then the property "merging session states is commutative, associative and idempotent" holds for 300 random cases

  @integration @US-TT02
  Scenario: Editing a session stamps only the changed fields
    Given device A adds a manual session from 09:00:00 to 10:00:00
    When device A changes the session note to "review" at 11:00:00
    Then on device A the field "note" of the session was last written at 11:00:00
    And on device A the field "actual_min" of the session was last written at 10:00:00
    And on device A the field "todo" of the session was last written at 10:00:00

  @integration @US-TT02
  Scenario: Deleting a session hides it from statistics and export
    Given device A and device B share the todo "Write report +work"
    And device A adds a manual session from 09:00:00 to 10:00:00 for the todo
    When device A deletes the session at 11:00:00
    Then on device A 0 sessions are listed
    And on device A the project "+work" has 0 minutes
    And on device A the database holds 1 sessions

  @integration @US-TT02
  Scenario: Migration 008 keeps the sessions of an existing database
    Given the Go fixture database
    When nunc opens the database
    Then there are 11 sessions
    And the sync state of the database holds 9 sessions with stable ids

  @integration @US-TT02
  Scenario: A backup from an older version can be restored
    Given no database exists yet
    When nunc opens the database
    And the Go fixture is copied to "src/go-backup.db"
    And I restore "src/go-backup.db" on 2026-10-04
    Then there are 11 sessions
    And the sync state of the database holds 9 sessions with stable ids

  @integration @US-TT02
  Scenario: An edited session reaches the other device
    Given device A adds a manual session from 09:00:00 to 10:00:00
    And the devices exchange their state
    When device A changes the session note to "review" at 11:00:00
    And the devices exchange their state
    Then on device B the session note is "review"
    And the exchange added 0 todos, replaced 0 and added 0 sessions

  @integration @US-TT02
  Scenario: Changes to different fields of one session on two devices are both kept
    Given device A adds a manual session from 09:00:00 to 10:00:00
    And the devices exchange their state
    When device A changes the session minutes to 45 at 11:00:00
    And device B changes the session note to "review" at 11:00:05
    And the devices exchange their state
    Then on device A the session has 45 minutes
    And on device A the session note is "review"
    And on device B the session has 45 minutes
    And on device B the session note is "review"

  @integration @US-TT02
  Scenario: A deleted session disappears on the other device
    Given device A adds a manual session from 09:00:00 to 10:00:00
    And the devices exchange their state
    When device A deletes the session at 11:00:00
    And the devices exchange their state
    Then on device B 0 sessions are listed
    And on device B the database holds 1 sessions

  @integration @US-TT02
  Scenario: A later edit wins over a concurrent deletion of a session
    Given device A adds a manual session from 09:00:00 to 10:00:00
    And the devices exchange their state
    When device A deletes the session at 11:00:00
    And device B changes the session note to "still needed" at 11:00:05
    And the devices exchange their state
    Then on device A 1 sessions are listed
    And on device B 1 sessions are listed

  @integration @sync @US-TT02
  Scenario: A version 2 sync file is read and written back as version 4
    Given a WebDAV server with the folder "/nunc/" holding a sync file of format 2
    And no database exists yet
    And the local todos "Task"
    When I sync
    Then the sync succeeds
    And the sync file on the server has format 4
