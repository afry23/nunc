Feature: Daily backup
  Source: US-Backup (B01, B03, B04, B05); nunc-go backup_test.go, backup_runnow_test.go, backup_status_test.go

  @integration @US-B01 @wip
  Scenario: A backup is created at start when none exists for today

  @integration @US-B01 @wip
  Scenario: The backup is skipped when one exists for today

  @integration @US-B01 @wip
  Scenario: A backup is created when the last one is outdated

  @integration @US-B01 @wip
  Scenario: The backup file is a consistent SQLite database

  @integration @US-B01 @wip
  Scenario: The last backup date is stored after a successful backup

  @integration @US-B03 @wip
  Scenario: The configured backup directory is used

  @integration @US-B03 @desktop @wip
  Scenario: The default backup directory follows the operating system

  @integration @US-B03 @desktop @wip
  Scenario: An unwritable directory keeps the previous path active

  @integration @US-B05 @wip
  Scenario: A manual backup runs immediately and returns path and size

  @integration @US-B05 @wip
  Scenario: A manual backup overwrites today's file and triggers rotation

  @integration @US-B04 @wip
  Scenario: The status reports count, total size and last backup

  @integration @US-B04 @wip
  Scenario: The status reports an empty backup directory

  @integration @US-B04 @wip
  Scenario: The status warns when the last backup is older than two days

  @ui @US-B04 @wip
  Scenario: The backup status is shown in the settings

  @ui @US-B05 @wip
  Scenario: The backup now button shows a spinner and then the result

  @ui @US-B04 @wip
  Scenario: A warning is shown when the last backup is too old

  @ui @US-B03 @desktop @wip
  Scenario: The backup folder can be chosen

  @ui @US-B05 @mobile @wip
  Scenario: Backups can be exported and shared
