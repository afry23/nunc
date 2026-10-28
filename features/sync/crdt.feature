Feature: CRDT sync
  Source: ADR 0016
  Todos merge field by field with a hybrid logical clock. Pomodoro counts add up per device. Sessions are covered by ADR 0017 (`session_crdt.feature`).

  @domain @REQ-SYNC
  Scenario: Changes to different fields of one todo are both kept
    Given a todo "a" on both devices
    And device A completes the todo
    And device B changes its text to "Buy oat milk"
    When the states are merged in either order
    Then the merged todo is completed
    And its text is "Buy oat milk"
    And the merged line is "x 2026-03-01 Buy oat milk"

  @domain @REQ-SYNC
  Scenario: The higher clock wins when both devices change the same field
    Given a todo "a" on both devices
    And device A sets the priority to "A" at 10:00:00
    And device B sets the priority to "B" at 10:00:05
    When the states are merged in either order
    Then the merged priority is "B"

  @domain @REQ-SYNC
  Scenario: Equal clocks are decided by device id, not by who merges
    Given a todo "a" on both devices
    And both devices set the priority at the same instant
    When the states are merged in either order
    Then both orders give the same priority

  @domain @REQ-SYNC
  Scenario: A later edit revives a deleted todo
    Given a todo "a" on both devices
    And device A deletes the todo at 10:00:00
    And device B changes its text at 10:00:05
    When the states are merged in either order
    Then the todo exists

  @domain @REQ-SYNC
  Scenario: A deletion after the last edit stays
    Given a todo "a" on both devices
    And device B changes its text at 10:00:00
    And device A deletes the todo at 10:00:05
    When the states are merged in either order
    Then the todo is deleted

  @domain @REQ-SYNC
  Scenario: Pomodoro counts of different devices add up
    Given a todo "a" with 2 pomodoros on both devices
    And device A adds 1 pomodoro
    And device B adds 3 pomodoros
    When the states are merged in either order
    Then the todo has 6 pomodoros

  @domain @REQ-SYNC
  Scenario: Sessions of both devices are kept once
    Given a session started at "2026-03-01T10:00:00Z" on device A
    And a session started at "2026-03-01T11:00:00Z" on device B
    And the same session of device A is also known to device B
    When the states are merged in either order
    Then there are 2 sessions

  @domain @REQ-SYNC
  Scenario: A remote clock far in the future is capped
    Given a todo "a" on both devices
    And device B changes its text with a clock 30 days ahead
    When device A receives the state of device B
    Then the stored clock is at most 24 hours ahead of now

  @domain @REQ-SYNC
  Scenario Outline: A new local stamp is newer than the last one
    Given the last known stamp is <last>
    When the clock reads <now>
    Then the next stamp is <next>

    Examples:
      | last  | now  | next  |
      | none  | 1000 | 1000+0 |
      | 1000+0 | 1000 | 1000+1 |
      | 1000+4 | 900  | 1000+5 |
      | 1000+4 | 1001 | 1001+0 |

  @domain @REQ-SYNC
  Scenario Outline: A position between two others
    When I place a todo between "<before>" and "<after>"
    Then the new position sorts strictly between them

    Examples:
      | before | after |
      |        |       |
      | i      |       |
      |        | i     |
      | a      | b     |
      | a      | a1    |
      | ab     | ac    |

  @domain @property @REQ-SYNC
  Scenario: Merging sync states does not depend on order or repetition
    Then the property "merging sync states is commutative" holds for 300 random cases
    And the property "merging sync states is associative" holds for 300 random cases
    And the property "merging sync states is idempotent" holds for 300 random cases

  @domain @property @REQ-SYNC
  Scenario: Positions stay ordered under repeated insertion
    Then the property "a position between two others sorts between them" holds for 300 random cases

  @domain @REQ-SYNC
  Scenario: A legacy todo becomes registers stamped by its update time
    Given a legacy todo "t-1" with the line "(A) 2026-09-20 Report +work @desk" and the description "Report" at position 3 updated at "2026-09-28T09:00:00Z"
    And a legacy todo "t-2" with the line "Water plants" and the description "Water plants" at position 1 updated at "2026-09-29T08:00:00Z"
    And the legacy todo "t-1" has 5 pomodoros
    When the legacy file is converted
    Then the todo "t-1" has the line "(A) 2026-09-20 Report +work @desk"
    And the todo "t-1" counts 5 pomodoros for the legacy device
    And the todo "t-2" has no pomodoro counters
    And every field of the todo "t-1" is stamped by the legacy device at "2026-09-28T09:00:00Z"
    And the todo "t-2" sorts before the todo "t-1"
    And the position key of the todo "t-2" is "0000015"

  @domain @REQ-SYNC
  Scenario: A deleted legacy todo stays deleted
    Given a deleted legacy todo "t-9" updated at "2026-09-28T09:00:00Z"
    When the legacy file is converted
    Then the todo "t-9" is deleted

  @domain @REQ-SYNC
  Scenario: The sync file survives writing and reading
    Given a todo "a" on both devices
    And device A completes the todo
    When the state of device A is written to a sync file and read back
    Then the state is unchanged

  @domain @REQ-SYNC
  Scenario Outline: A sync file of another format is refused
    When a sync file of format <format> is read
    Then reading fails with "Die Sync-Datei hat das Format <format>, diese Version versteht 2 bis 5. Bitte nunc aktualisieren."

    Examples:
      | format |
      | 1      |
      | 6      |

  @domain @REQ-SYNC
  Scenario: A session is the same however its start is written
    Given a session started at "2026-03-01T10:00:00Z" on device A
    And a session started at "2026-03-01T11:00:00+01:00" on device A
    And the same session of device A is also known to device B
    When the states are merged in either order
    Then there are 1 sessions

  @domain @REQ-SYNC
  Scenario Outline: Tombstones are dropped after 90 days
    Given an old todo "a" on both devices
    And device A deleted the todo <days> days ago
    When the garbage is collected
    Then the todo is <outcome> in the state

    Examples:
      | days | outcome |
      | 10   | kept    |
      | 89   | kept    |
      | 90   | kept    |
      | 91   | dropped |
      | 400  | dropped |

  @domain @REQ-SYNC
  Scenario: A live todo is never dropped, however old
    Given an old todo "a" on both devices
    When the garbage is collected 400 days later
    Then the todo is kept in the state

  @domain @REQ-SYNC
  Scenario: A deletion that an edit overruled is no tombstone
    Given an old todo "a" on both devices
    And device A deleted the todo 200 days ago
    And device B changed the todo 150 days ago
    When the garbage is collected
    Then the todo is kept in the state

  @domain @property @REQ-SYNC
  Scenario: Collecting garbage twice changes nothing more
    Then the property "garbage collection is idempotent and only drops deleted todos" holds for 300 random cases


  @domain @REQ-SYNC
  Scenario: The parent of a todo merges last writer wins
    Given a todo "a" on both devices
    And device A moves the todo below "p1" at 10:00:00
    And device B moves the todo below "p2" at 10:00:05
    When the states are merged in either order
    Then the merged todo is below "p2"

  @domain @REQ-SYNC
  Scenario: A sync file without parent fields still loads
    Given a todo "a" on both devices
    When a todo is read from a sync file that has no parent field
    Then the todo has no parent
