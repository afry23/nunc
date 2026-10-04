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

  @integration @US-R01
  Scenario: Backups are listed newest first
    Given the folder "backups" has the empty files "nunc-backup-2026-09-27.db nunc-backup-2026-10-04.db nunc-backup-2026-09-06.db before-restore-2026-10-04.db"
    When I list the backups of the folder "backups" on 2026-10-04
    Then the listed backups are:
      | filename                  |
      | nunc-backup-2026-10-04.db |
      | nunc-backup-2026-09-27.db |
      | nunc-backup-2026-09-06.db |

  @integration @US-R01
  Scenario: Each entry shows date, time, category and size
    Given the folder "backups" has the files:
      | name                      | size | modified             |
      | nunc-backup-2026-10-04.db | 100  | 2026-10-04T08:00:00Z |
      | nunc-backup-2026-09-27.db | 700  | 2026-09-27T08:00:00Z |
      | nunc-backup-2026-09-06.db | 900  | 2026-09-06T08:00:00Z |
    When I list the backups of the folder "backups" on 2026-10-04
    Then the listed backups are:
      | createdAt            | category | sizeBytes |
      | 2026-10-04T12:00:00Z | daily    | 100       |
      | 2026-09-27T12:00:00Z | weekly   | 700       |
      | 2026-09-06T12:00:00Z | monthly  | 900       |

  @integration @US-R02
  Scenario: An emergency backup is created before restoring
    Given no database exists yet
    When nunc opens the database
    And I add the todo "current"
    And a backup file "source.db" with the todos "from backup" exists
    And I restore "source.db" on 2026-10-04
    Then the folder "backups" contains exactly "before-restore-2026-10-04.db"
    And the database "backups/before-restore-2026-10-04.db" has the todos "current"

  @integration @US-R02
  Scenario: The emergency backup overwrites the one from the same day
    Given no database exists yet
    When nunc opens the database
    And I add the todo "current"
    And a backup file "source.db" with the todos "from backup" exists
    And I restore "source.db" on 2026-10-04
    And I restore "source.db" on 2026-10-04
    Then the folder "backups" contains exactly "before-restore-2026-10-04.db"
    And the database "backups/before-restore-2026-10-04.db" has the todos "from backup"

  @integration @US-R02
  Scenario: Restoring replaces all data with the backup
    Given no database exists yet
    When nunc opens the database
    And I add the todo "current"
    And a backup file "source.db" with the todos "from backup" exists
    And the settings table has "pomodoro_min" = "99"
    And I restore "source.db" on 2026-10-04
    And I list the todos
    And I load the settings
    Then the listed todos are:
      | rawText     |
      | from backup |
    And the loaded settings are:
      | setting     | value |
      | pomodoroMin | 25    |
    And the data directory has no leftover restore files

  @integration @US-R02
  Scenario: The chosen backup file is left untouched
    Given no database exists yet
    When nunc opens the database
    And the Go fixture is copied to "src/go-backup.db"
    And the state of the folder "src" is remembered
    And I restore "src/go-backup.db" on 2026-10-04
    Then the folder "src" is unchanged

  @integration @US-R02
  Scenario: A failed restore keeps the original data
    Given no database exists yet
    When nunc opens the database
    And I add the todo "current"
    And a backup file "source.db" with the todos "from backup" exists
    And a broken nunc backup "broken.db" exists
    And I try to restore "broken.db" on 2026-10-04
    Then the restore failed
    When I list the todos
    Then the listed todos are:
      | rawText |
      | current |
    And the data directory has no leftover restore files

  @integration @US-R03
  Scenario: The backup file is validated as a nunc database
    Given no database exists yet
    When nunc opens the database
    And a backup file "source.db" with the todos "x" exists
    Then the backup file "source.db" is valid

  @integration @US-R03
  Scenario: A file that is not SQLite is rejected
    Given no database exists yet
    When nunc opens the database
    And a file named "fake.db" exists
    Then the backup file "fake.db" is rejected with "keine SQLite-Datenbank"

  @integration @US-R03
  Scenario: A SQLite file without the nunc schema is rejected
    Given no database exists yet
    When nunc opens the database
    And a plain SQLite file "other.db" exists
    Then the backup file "other.db" is rejected with "keine nunc-Datenbank"

  @integration @US-R03
  Scenario: A database from a newer version is rejected
    Given no database exists yet
    When nunc opens the database
    And a backup file "future.db" with schema version 99 exists
    Then the backup file "future.db" is rejected with "neueren Version"

  @integration @US-R03
  Scenario: A missing file is rejected
    Given no database exists yet
    When nunc opens the database
    Then the backup file "missing.db" is rejected with "Datei nicht lesbar"

  @ui @US-R01
  Scenario: The backup list shows the newest backup first
    Given the backups:
      | date       | category | sizeBytes |
      | 2026-10-04 | daily    | 100       |
      | 2026-09-27 | weekly   | 700       |
      | 2026-09-06 | monthly  | 900       |
    When I open the app at "/settings"
    Then the backup list shows:
      | date       | category    | size  |
      | 04.10.2026 | Täglich     | 100 B |
      | 27.09.2026 | Wöchentlich | 700 B |
      | 06.09.2026 | Monatlich   | 900 B |

  @ui @US-R02
  Scenario: A confirmation dialog warns that all data is replaced
    Given the backups:
      | date       | category | sizeBytes |
      | 2026-10-03 | daily    | 1000      |
    When I open the app at "/settings"
    And I click "Backup vom 03.10.2026 wiederherstellen"
    Then I see a dialog with the text "Alle aktuellen Daten werden durch den Stand von nunc-backup-2026-10-03.db ersetzt"
    And I see a dialog with the text "Notfall-Backup"
    And the backend did not receive "restore_from_backup"
    When I click "Abbrechen"
    Then I do not see a dialog
    And the backend did not receive "restore_from_backup"

  @ui @US-R03
  Scenario: Restore from a file asks for a file and confirms first
    Given the file dialog returns "/home/me/Downloads/old.db"
    When I open the app at "/settings"
    And I click "Aus Datei wiederherstellen"
    Then the backend received "select_restore_file"
    And I see a dialog with the text "old.db"
    When I click "Jetzt wiederherstellen"
    Then the backend received "restore_from_backup" with:
      """
      { "sourcePath": "/home/me/Downloads/old.db" }
      """

  @ui @US-R03
  Scenario: Cancelling the file dialog opens no confirmation
    Given the file dialog is cancelled
    When I open the app at "/settings"
    And I click "Aus Datei wiederherstellen"
    Then I do not see a dialog
    And the backend did not receive "restore_from_backup"

  @ui @US-R02
  Scenario: A restore error is shown and no data changes
    Given the todos:
      | rawText |
      | Keep me |
    And the backups:
      | date       | category | sizeBytes |
      | 2026-10-03 | daily    | 1000      |
    And the restore fails with "invalid_backup": "Datei ist keine nunc-Datenbank"
    When I open the app at "/settings"
    And I click "Backup vom 03.10.2026 wiederherstellen"
    And I click "Jetzt wiederherstellen"
    Then I see "Datei ist keine nunc-Datenbank"
    And I do not see a dialog
    When I click "Aufgaben"
    Then I see "Keep me"

  @ui @US-R02
  Scenario: The app reloads its data after a restore
    Given the todos:
      | rawText |
      | Current |
    And the backups:
      | date       | category | sizeBytes |
      | 2026-10-03 | daily    | 1000      |
    And the backup "nunc-backup-2026-10-03.db" contains the todos:
      | rawText     |
      | From backup |
    When I open the app at "/settings"
    And I click "Backup vom 03.10.2026 wiederherstellen"
    And I click "Jetzt wiederherstellen"
    Then I see "Daten wiederhergestellt."
    When I click "Aufgaben"
    Then I see "From backup"
    And I do not see "Current"

  @ui @US-R03 @mobile
  Scenario: Import a backup file with the file picker
    Given the platform is "mobile"
    And the file dialog returns "/data/cache/import/phone.db"
    When I open the app at "/settings"
    And I click "Backup-Datei importieren"
    Then the backend received "select_restore_file"
    And I see a dialog with the text "phone.db"
    When I click "Jetzt wiederherstellen"
    Then the backend received "restore_from_backup" with:
      """
      { "sourcePath": "/data/cache/import/phone.db" }
      """
