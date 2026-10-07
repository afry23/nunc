Feature: CRDT sync
  Source: ADR 0016
  Todos merge field by field with a hybrid logical clock. Pomodoro counts add up per device. Sessions stay append-only.

  @domain @REQ-SYNC
  Scenario: Changes to different fields of one todo are both kept
    Given a todo "a" on both devices
    And device A completes the todo
    And device B changes its text to "Buy oat milk"
    When the states are merged in either order
    Then the merged todo is completed
    And its text is "Buy oat milk"

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

  @wip @integration @sync @REQ-SYNC
  Scenario: A legacy sync file is imported once
    Given the server holds only a legacy "nunc-sync.json"
    When I sync
    Then the todos from the legacy file are present
    And the server holds "nunc-sync.v2.json"
    And a second sync imports nothing from the legacy file
