Feature: Connection and migrations
  The store opens SQLite like the Go app (WAL, foreign keys) and applies the
  embedded migrations through the same schema_migrations table.

  @integration @REQ-compat @desktop
  Scenario: A new database gets all migrations
    Given no database exists yet
    When nunc opens the database
    Then the migrations 1 to 5 are recorded as applied
    And the tables "settings, flowtime_rules, todos, sessions" exist

  @integration @REQ-compat @desktop
  Scenario: Opening twice applies nothing again
    Given no database exists yet
    When nunc opens the database
    And nunc opens the database
    Then the migrations 1 to 5 are recorded as applied
    And the migration timestamps did not change

  @integration @REQ-compat @desktop
  Scenario: Foreign keys are enforced
    Given no database exists yet
    When nunc opens the database
    Then inserting a session for the todo 999 is rejected

  @integration @REQ-compat @desktop
  Scenario: A file database uses WAL and creates its folder
    Given no database exists yet
    When nunc opens the database in a not yet existing folder
    Then the journal mode is "wal"
