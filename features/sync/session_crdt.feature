Feature: Session CRDT sync
  Source: ADR 0017
  Sessions carry a stable id and merge field by field like todos.

  @domain @US-TT02 @wip
  Scenario: Old sessions without an id get the same id on every device

  @integration @sync @US-TT02 @wip
  Scenario: Two devices that both know an old session do not create a duplicate
