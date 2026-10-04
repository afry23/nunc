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
