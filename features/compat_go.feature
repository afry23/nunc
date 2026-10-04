Feature: Compatibility with the database of the Go app
  Fixtures: tests/fixtures/go-v5.db (schema version 5, created by nunc-go)

  @integration @REQ-compat @desktop
  Scenario: The Go database opens and its migrations are recognized as applied
    Given the Go fixture database
    When nunc opens the database
    Then the migrations 1 to 5 are recorded as applied
    And the migration timestamps did not change
    And the table "todos" contains 6 rows
    And the table "sessions" contains 11 rows

  @integration @REQ-compat @desktop
  Scenario: Only the additive migration 006 is applied on top
    Given the Go fixture database
    When nunc opens the database
    Then exactly the migrations 1 to 6 are recorded as applied
    And the migration timestamps did not change

  @integration @REQ-compat @desktop @wip
  Scenario: Settings and their defaults match the Go keys

  @integration @REQ-compat @desktop @wip
  Scenario: Todos are read including sync ids and soft-deleted ones

  @integration @REQ-compat @desktop @wip
  Scenario: Sessions are read including the sync device id

  @integration @REQ-compat @desktop @wip
  Scenario: A backup created by the Go app can be restored

  @integration @REQ-compat @desktop @wip
  Scenario: A database written by nunc keeps the Go tables unchanged

  @integration @REQ-compat @desktop @wip
  Scenario: On first start the database of the Go app is copied from the old data path

  @integration @REQ-compat @desktop @wip
  Scenario: The original Go database stays untouched after the copy

  @integration @REQ-compat @desktop @wip
  Scenario: The backup folder setting is taken over with the copy

  @e2e @REQ-compat @manual @wip
  Scenario: A database written by nunc can be opened by the Go app
