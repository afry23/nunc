Feature: Restore
  Source: US-Restore (R01-R03); nunc-go backup_list_test.go, backup_restore_test.go

  @domain @US-R01
  Scenario: The list shows the newest backup first with date, category and size
    Given today is 2026-10-04
    And the backup files:
      | name                      | size |
      | nunc-backup-2026-09-27.db | 700  |
      | nunc-backup-2026-10-04.db | 100  |
      | nunc-backup-2026-09-06.db | 900  |
      | nunc-backup-2026-09-28.db | 600  |
      | nunc-backup-2026-09-07.db | 800  |
    When I list the backups
    Then the backup list is:
      | nunc-backup-2026-10-04.db | 2026-10-04T12:00:00+00:00 | daily   | 100 |
      | nunc-backup-2026-09-28.db | 2026-09-28T12:00:00+00:00 | daily   | 600 |
      | nunc-backup-2026-09-27.db | 2026-09-27T12:00:00+00:00 | weekly  | 700 |
      | nunc-backup-2026-09-07.db | 2026-09-07T12:00:00+00:00 | weekly  | 800 |
      | nunc-backup-2026-09-06.db | 2026-09-06T12:00:00+00:00 | monthly | 900 |

  @domain @US-R01
  Scenario: The list is empty without backups and ignores other files
    Given today is 2026-10-04
    And the backup files:
      | name                         |
      | before-restore-2026-10-04.db |
      | notes.txt                    |
    When I list the backups
    Then the backup list is:

  @domain @US-R01
  Scenario: The list keeps the path of each backup
    Given today is 2026-10-04
    And the backup files:
      | name                      |
      | nunc-backup-2026-10-03.db |
      | nunc-backup-2026-10-04.db |
    When I list the backups
    Then the first listed backup has the path "/backups/nunc-backup-2026-10-04.db"

  @domain @US-R01
  Scenario Outline: The category follows the age in days
    Given today is 2026-10-04
    And the backup files:
      | name                |
      | nunc-backup-<day>.db |
    When I list the backups
    Then the backup list is:
      | nunc-backup-<day>.db | <day>T12:00:00+00:00 | <category> | 1 |

    Examples:
      | day        | category |
      | 2026-10-04 | daily    |
      | 2026-09-28 | daily    |
      | 2026-09-27 | weekly   |
      | 2026-09-07 | weekly   |
      | 2026-09-06 | monthly  |
      | 2025-10-04 | monthly  |
      | 2026-10-05 | daily    |

  @integration @US-R01 @wip
  Scenario: Backups are listed newest first

  @integration @US-R01 @wip
  Scenario: Each entry shows date, time, category and size

  @integration @US-R02 @wip
  Scenario: An emergency backup is created before restoring

  @integration @US-R02 @wip
  Scenario: The emergency backup overwrites the one from the same day

  @integration @US-R02 @wip
  Scenario: Restoring replaces all data with the backup

  @integration @US-R02 @wip
  Scenario: A failed restore keeps the original data

  @integration @US-R03 @wip
  Scenario: The backup file is validated as a nunc database

  @integration @US-R03 @wip
  Scenario: A file that is not SQLite is rejected

  @integration @US-R03 @wip
  Scenario: A SQLite file without the nunc schema is rejected

  @integration @US-R03 @wip
  Scenario: A missing file is rejected

  @ui @US-R01 @wip
  Scenario: The backup list shows the newest backup first

  @ui @US-R02 @wip
  Scenario: A confirmation dialog warns that all data is replaced

  @ui @US-R03 @wip
  Scenario: Restore from file only offers db files

  @ui @US-R02 @wip
  Scenario: A restore error is shown and no data changes

  @ui @US-R02 @wip
  Scenario: The app reloads its data after a restore

  @ui @US-R03 @mobile @wip
  Scenario: Import a backup file with the file picker
