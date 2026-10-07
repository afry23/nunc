Feature: CRDT state in the database
  Source: ADR 0016
  Local changes stamp only the fields they change. Two databases that exchange their state end up identical.

  @integration @REQ-SYNC
  Scenario: Only the changed fields are stamped
    Given device A has the todo "(B) Buy milk" created at 10:00:00
    When device A changes the todo to "(B) Buy oat milk" at 11:00:00
    Then on device A the field "description" of the todo was last written at 11:00:00
    And on device A the field "priority" of the todo was last written at 10:00:00
    And on device A the field "completed" of the todo was last written at 10:00:00

  @integration @REQ-SYNC
  Scenario: A stamp is newer than the last one even if the clock runs backwards
    Given device A has the todo "Buy milk" created at 10:00:00
    When device A changes the todo to "Buy oat milk" at 11:00:00
    And device A changes the todo to "Buy soy milk" at 10:30:00
    Then on device A the field "description" of the todo was last written at 11:00:00 with counter 1

  @integration @REQ-SYNC
  Scenario: Moving one todo stamps only that todo
    Given device A has the todos "first, second, third" created at 10:00:00
    When device A reorders the todos to "3,1,2" at 11:00:00
    Then on device A the position of todo 3 was last written at 11:00:00
    And on device A the position of todo 1 was last written at 10:00:00
    And on device A the position of todo 2 was last written at 10:00:00
    And on device A the todos are listed as "third, first, second"

  @integration @REQ-SYNC
  Scenario: Changes to different fields on two devices are both kept
    Given device A and device B share the todo "Buy milk"
    When device A completes the todo at 10:00:00
    And device B changes the todo to "Buy oat milk" at 10:00:05
    And the devices exchange their state
    Then on both devices the todo line is "x 2026-10-04 Buy oat milk"
    And on both devices the todo was last changed at 10:00:05
    And the exchange added 0 todos, replaced 2 and added 0 sessions

  @integration @REQ-SYNC
  Scenario: Pomodoros counted on two devices add up
    Given device A and device B share the todo "Focus"
    When device A counts 1 pomodoro for the todo
    And device B counts 2 pomodoros for the todo
    And the devices exchange their state
    Then on both devices the todo has 3 pomodoros

  @integration @REQ-SYNC
  Scenario: A todo created on one device appears on the other
    Given device A has the todo "Only on A" created at 10:00:00
    When the devices exchange their state
    Then on both devices the todos are listed as "Only on A"
    And the exchange added 1 todos, replaced 0 and added 0 sessions

  @integration @REQ-SYNC
  Scenario: A deletion reaches the other device
    Given device A and device B share the todo "Obsolete"
    When device A deletes the todo at 10:00:00
    And the devices exchange their state
    Then on both devices the todos are listed as ""

  @integration @REQ-SYNC
  Scenario: A later edit wins over a concurrent deletion
    Given device A and device B share the todo "Contested"
    When device A deletes the todo at 10:00:00
    And device B changes the todo to "Contested, reworded" at 10:00:05
    And the devices exchange their state
    Then on both devices the todos are listed as "Contested, reworded"

  @integration @REQ-SYNC
  Scenario: Exchanging the state twice changes nothing
    Given device A and device B share the todo "Stable"
    When device A deletes the todo at 10:00:00
    And device B changes the todo to "Stable, reworded" at 10:00:05
    And the devices exchange their state
    And the devices exchange their state
    Then the second exchange changed nothing

  @integration @REQ-SYNC @desktop
  Scenario: Existing todos get a stamp for every field when the database is migrated
    Given the Go fixture database
    When nunc opens the database
    Then every todo has a stamp for each of its 7 fields
    And the position keys follow the old order

  @integration @REQ-SYNC
  Scenario: A todo that was deleted before the other device saw it is not created there
    Given device A has the todo "Short-lived" created at 09:00:00
    And device A deletes the todo at 10:00:00
    When the devices exchange their state
    Then on device B the database holds 0 todos
    And the exchange added 0 todos, replaced 0 and added 0 sessions

  @integration @REQ-SYNC
  Scenario: Sessions are exchanged once
    Given device A has a completed session started at 09:00:00
    When the devices exchange their state
    And the devices exchange their state
    Then the second exchange changed nothing
    And on device B the database holds 1 sessions

  @integration @REQ-SYNC
  Scenario: Sessions are added when the devices exchange their state
    Given device A has a completed session started at 09:00:00
    When the devices exchange their state
    Then the exchange added 0 todos, replaced 0 and added 1 sessions

  @integration @REQ-SYNC
  Scenario: A local stamp after an exchange is newer than every stamp seen
    Given device A and device B share the todo "Ahead"
    When device B changes the todo to "Ahead on B" at 15:00:00
    And the devices exchange their state
    And device A changes the todo to "Ahead on A" at 11:00:00
    Then on device A the field "description" of the todo was last written at 15:00:00 with counter 1
