Feature: CRDT sync
  Source: ADR 0016
  Todos merge field by field with a hybrid logical clock. Pomodoro counts add up per device. Sessions stay append-only.

  @wip @domain @REQ-SYNC
  Scenario: Changes to different fields of one todo are both kept
    Given a todo "a" on both devices
    And device A completes the todo
    And device B changes its text to "Buy oat milk"
    When the states are merged in either order
    Then the todo is completed
    And its text is "Buy oat milk"

  @wip @domain @REQ-SYNC
  Scenario: The higher clock wins when both devices change the same field
    Given a todo "a" on both devices
    And device A sets the priority to "A" at 10:00:00
    And device B sets the priority to "B" at 10:00:05
    When the states are merged in either order
    Then the priority is "B"

  @wip @domain @REQ-SYNC
  Scenario: Equal clocks are decided by device id, not by who merges
    Given a todo "a" on both devices
    And both devices set the priority at the same instant
    When the states are merged in either order
    Then both orders give the same priority

  @wip @domain @REQ-SYNC
  Scenario: A later edit revives a deleted todo
    Given a todo "a" on both devices
    And device A deletes the todo at 10:00:00
    And device B changes its text at 10:00:05
    When the states are merged in either order
    Then the todo exists

  @wip @domain @REQ-SYNC
  Scenario: Pomodoro counts of different devices add up
    Given a todo "a" with 2 pomodoros on both devices
    And device A adds 1 pomodoro
    And device B adds 3 pomodoros
    When the states are merged in either order
    Then the todo has 6 pomodoros

  @wip @domain @property @REQ-SYNC
  Scenario: Merging is commutative, associative and idempotent
    Given random sync states
    Then merge(a, b) equals merge(b, a)
    And merge(a, merge(b, c)) equals merge(merge(a, b), c)
    And merge(a, a) equals a

  @wip @domain @REQ-SYNC
  Scenario: A remote clock far in the future is capped
    Given a remote change stamped 30 days ahead
    When the states are merged
    Then the stored clock is at most 24 hours ahead of now

  @wip @integration @sync @REQ-SYNC
  Scenario: A legacy sync file is imported once
    Given the server holds only a legacy "nunc-sync.json"
    When I sync
    Then the todos from the legacy file are present
    And the server holds "nunc-sync.v2.json"
    And a second sync imports nothing from the legacy file
