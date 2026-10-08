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
