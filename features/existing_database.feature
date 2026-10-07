Feature: Existing databases
  A database created by the Go app (schema version 5) is opened, migrated and taken over once. After that nunc
  owns it: there is no parallel operation with the Go app.
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
  Scenario: Only the additive migrations 006 and 007 are applied on top
    Given the Go fixture database
    When nunc opens the database
    Then exactly the migrations 1 to 7 are recorded as applied
    And the migration timestamps did not change

  @integration @REQ-compat @desktop
  Scenario: Settings and their defaults match the Go keys
    Given the Go fixture database
    When nunc opens the database
    And I load the settings
    Then the loaded settings are:
      | setting                 | value      |
      | pomodoroMin             | 30         |
      | shortBreakMin           | 6          |
      | longBreakAfter          | 3          |
      | soundVolume             | 55         |
      | lastBackupDate          | 2026-09-29 |
      | syncProvider            | webdav     |
      | flowtimeRatio           | 0.25       |
      | flowtimeMinFlowStateMin | 20         |
      | autoStartNextPomodoro   | false      |

  @integration @REQ-compat @desktop
  Scenario: Todos are read including sync ids and soft-deleted ones
    Given the Go fixture database
    When nunc opens the database
    And I list the todos
    Then the listed todos are:
      | id | rawText                                      | priority | position | pomodoroCount |
      | 1  | (A) 2026-09-20 Write quarterly report +work @desk | A   | 0        | 5             |
      | 2  | (B) 2026-09-21 Call dentist @phone           | B        | 1        | 1             |
      | 3  | 2026-09-22 Buy groceries +home @errands      |          | 2        | 0             |
      | 4  | x 2026-09-25 2026-09-18 Plan sprint +work    |          | 3        | 3             |
      | 5  | (C) Read Rust book +learning +rust @evening | C        | 4        | 2             |
    And the todo 4 is:
      | field       | value      |
      | completed   | true       |
      | completedAt | 2026-09-25 |
      | createdAt   | 2026-09-18 |
    And the todo with sync id "t-0006" is marked as deleted
    And the todo with sync id "t-0001" has the id 1

  @integration @REQ-compat @desktop
  Scenario: Sessions are read including the sync device id
    Given the Go fixture database
    When nunc opens the database
    Then there are 11 sessions
    And session 1 is:
      | field      | value                |
      | mode       | pomodoro             |
      | sessionType| work                 |
      | status     | completed            |
      | startedAt  | 2026-09-22T09:00:00Z |
      | endedAt    | 2026-09-22T09:30:00Z |
      | plannedMin | 30                   |
      | actualMin  | 30                   |
      | todoId     | 1                    |
      | breakMin   | 6                    |
    And session 2 is:
      | field  | value |
      | todoId |       |
    And all sessions have the sync device id "fixture-device-a"

  @integration @REQ-compat @desktop
  Scenario: A backup created by the Go app can be restored
    Given no database exists yet
    When nunc opens the database
    And I add the todo "mine"
    And the Go fixture is copied to "go-backup.db"
    And I restore "go-backup.db" on 2026-10-04
    And I list the todos
    Then the listed todos are:
      | id | rawText                                           |
      | 1  | (A) 2026-09-20 Write quarterly report +work @desk |
      | 2  | (B) 2026-09-21 Call dentist @phone                |
      | 3  | 2026-09-22 Buy groceries +home @errands           |
      | 4  | x 2026-09-25 2026-09-18 Plan sprint +work         |
      | 5  | (C) Read Rust book +learning +rust @evening       |
    And exactly the migrations 1 to 7 are recorded as applied
    And there are 11 sessions

  @integration @REQ-compat @desktop
  Scenario: A database written by nunc keeps the Go tables unchanged
    Given no database exists yet
    When nunc opens the database
    And I add the todo "(A) 2026-10-01 Write report +work"
    And a pomodoro for todo 1 is started at 09:00:00 and saved
    And the pomodoro is finished at 09:25:00 and stored
    And I load the settings
    And I set "pomodoroMin" to "30"
    And I save the settings
    Then the Go tables keep the definitions of the Go fixture
    And the settings keys are all known to the Go app
    And the rows written by nunc use the Go formats

  @integration @REQ-compat @desktop
  Scenario: On first start the database of the Go app is copied from the old data path
    Given the Go app left its database in the user config folder "config"
    When nunc starts with the data folder "data"
    Then the legacy import result is "imported"
    And exactly the migrations 1 to 7 are recorded as applied
    And there are 11 sessions
    When I list the todos
    Then the todo list has 5 entries

  @integration @REQ-compat @desktop
  Scenario: A second start does not copy again
    Given the Go app left its database in the user config folder "config"
    When nunc starts with the data folder "data"
    And I add the todo "added by nunc"
    And nunc starts with the data folder "data"
    Then the legacy import result is "already_present"
    When I list the todos
    Then the todo list has 6 entries

  @integration @REQ-compat @desktop
  Scenario: Without a Go database nunc starts with an empty one
    When nunc starts with the data folder "data"
    Then the legacy import result is "no_legacy_database"
    When I list the todos
    Then the todo list is empty

  @integration @REQ-compat @desktop
  Scenario: The original Go database stays untouched after the copy
    Given the Go app left its database in the user config folder "config"
    And the state of the folder "config/nunc" is remembered
    When nunc starts with the data folder "data"
    And I add the todo "added by nunc"
    Then the folder "config/nunc" is unchanged

  @integration @REQ-compat @desktop
  Scenario: The backup folder setting is taken over with the copy
    Given the Go app left its database in the user config folder "config"
    And the Go database has the backup folder "/data/backups/nunc"
    When nunc starts with the data folder "data"
    And I load the settings
    Then the loaded settings are:
      | setting   | value              |
      | backupDir | /data/backups/nunc |
