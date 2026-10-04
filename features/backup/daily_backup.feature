Feature: Daily backup
  Source: US-Backup (B01, B03, B04, B05); nunc-go backup_test.go, backup_runnow_test.go, backup_status_test.go

  @domain @US-B01
  Scenario Outline: A backup is needed unless one was already made today
    Given today is 2026-10-04
    And the last backup date is "<last>"
    When I check whether a backup is needed
    Then <result>

    Examples:
      | last       | result              |
      |            | a backup is needed  |
      | 2026-10-03 | a backup is needed  |
      | 2026-10-04 | no backup is needed |
      | 2026-10-05 | a backup is needed  |
      | yesterday  | a backup is needed  |

  @domain @US-B01
  Scenario Outline: The backup files are named after the day
    Then the daily backup of <day> is named "<daily>"
    And the emergency backup of <day> is named "<emergency>"

    Examples:
      | day        | daily                     | emergency                    |
      | 2026-10-04 | nunc-backup-2026-10-04.db | before-restore-2026-10-04.db |
      | 2025-01-09 | nunc-backup-2025-01-09.db | before-restore-2025-01-09.db |

  @domain @US-B01
  Scenario Outline: Only exact backup names carry a date
    Given the file name "<name>"
    When I read the backup date from the file name
    Then the backup date is <date>

    Examples:
      | name                         | date       |
      | nunc-backup-2026-10-04.db    | 2026-10-04 |
      | nunc-backup-2024-02-29.db    | 2024-02-29 |
      | nunc-backup-2025-02-29.db    | none       |
      | nunc-backup-2025-13-01.db    | none       |
      | nunc-backup-2025-6-1.db      | none       |
      | nunc-backup-2025-06-01.db.tmp | none      |
      | nunc-backup-2025-06-01       | none       |
      | before-restore-2025-06-01.db | none       |
      | backup-2025-06-01.db         | none       |
      | nunc-backup-.db              | none       |

  @domain @US-B04
  Scenario: The status of an empty folder is outdated
    Given the backup files:
      | name |
    When I read the backup status at 2026-10-04T12:00:00Z
    Then the status shows 0 backups with 0 bytes
    And the last backup was at never
    And the status is outdated

  @domain @US-B04
  Scenario: The status reports count, total size and last backup
    Given the backup files:
      | name                      | size | modified             |
      | nunc-backup-2026-10-02.db | 1000 | 2026-10-02T09:00:00Z |
      | nunc-backup-2026-10-04.db | 2500 | 2026-10-04T08:30:00Z |
      | nunc-backup-2026-10-03.db | 500  | 2026-10-03T09:00:00Z |
    When I read the backup status at 2026-10-04T12:00:00Z
    Then the status shows 3 backups with 4000 bytes
    And the last backup was at 2026-10-04T08:30:00Z
    And the status is up to date

  @domain @US-B04
  Scenario Outline: The status warns when the last backup is older than two days
    Given the backup files:
      | name                      | modified   |
      | nunc-backup-2026-10-01.db | <modified> |
    When I read the backup status at 2026-10-04T10:00:00Z
    Then <result>

    Examples:
      | modified             | result                   |
      | 2026-10-04T09:00:00Z | the status is up to date |
      | 2026-10-02T10:00:01Z | the status is up to date |
      | 2026-10-02T10:00:00Z | the status is up to date |
      | 2026-10-02T09:59:59Z | the status is outdated   |
      | 2026-09-01T00:00:00Z | the status is outdated   |

  @domain @US-B04
  Scenario: The status ignores files that are not backups
    Given the backup files:
      | name                         | size | modified             |
      | nunc-backup-2026-10-04.db    | 100  | 2026-10-04T08:00:00Z |
      | before-restore-2026-10-04.db | 900  | 2026-10-04T11:00:00Z |
      | notes.txt                    | 50   | 2026-10-04T11:30:00Z |
    When I read the backup status at 2026-10-04T12:00:00Z
    Then the status shows 1 backups with 100 bytes
    And the last backup was at 2026-10-04T08:00:00Z

  @integration @US-B01
  Scenario: A backup is created at start when none exists for today
    Given no database exists yet
    When nunc opens the database
    And I run the daily backup on 2026-10-04
    Then a backup was made
    And the folder "backups" contains exactly "nunc-backup-2026-10-04.db"

  @integration @US-B01
  Scenario: The backup is skipped when one exists for today
    Given no database exists yet
    When nunc opens the database
    And the settings table has "last_backup_date" = "2026-10-04"
    And I run the daily backup on 2026-10-04
    Then no backup was made
    And the folder "backups" does not exist

  @integration @US-B01
  Scenario: A backup is created when the last one is outdated
    Given no database exists yet
    When nunc opens the database
    And the settings table has "last_backup_date" = "2026-10-01"
    And I run the daily backup on 2026-10-04
    Then a backup was made
    And the folder "backups" contains exactly "nunc-backup-2026-10-04.db"

  @integration @US-B01
  Scenario: The backup file is a consistent SQLite database
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Written before the backup"
    And I run the daily backup on 2026-10-04
    Then the backup file "backups/nunc-backup-2026-10-04.db" is valid
    And the database "backups/nunc-backup-2026-10-04.db" has the todos "Written before the backup"
    And the folder "backups" has no temporary files

  @integration @US-B01
  Scenario: The last backup date is stored after a successful backup
    Given no database exists yet
    When nunc opens the database
    And I run the daily backup on 2026-10-04
    Then the settings table contains:
      | key              | value      |
      | last_backup_date | 2026-10-04 |

  @integration @US-B03
  Scenario: The configured backup directory is used
    Given no database exists yet
    When nunc opens the database
    And I choose the backup folder "custom"
    And I run the daily backup on 2026-10-04 into the configured or default folder "default"
    Then the folder "custom" contains exactly "nunc-backup-2026-10-04.db"
    And the folder "default" does not exist

  @integration @US-B03 @desktop
  Scenario: The default backup directory follows the operating system
    Given no database exists yet
    When nunc opens the database
    Then the backup folder resolves to the default "default" when none is configured

  @integration @US-B03 @desktop
  Scenario: An unwritable directory keeps the previous path active
    Given no database exists yet
    When nunc opens the database
    And I choose the backup folder "custom"
    And a file named "blocked" exists
    And I try to choose the backup folder "blocked/sub"
    Then choosing the folder failed
    And the backup folder setting is "custom"

  @integration @US-B05
  Scenario: A manual backup runs immediately and returns path and size
    Given no database exists yet
    When nunc opens the database
    And I add the todo "Manual"
    And I run a manual backup on 2026-10-04
    Then the backup result is "nunc-backup-2026-10-04.db" with the real file size

  @integration @US-B05
  Scenario: A manual backup overwrites today's file and triggers rotation
    Given no database exists yet
    When nunc opens the database
    And the folder "backups" has the empty files "nunc-backup-2026-09-27.db nunc-backup-2026-09-28.db nunc-backup-2026-09-29.db nunc-backup-2026-09-30.db nunc-backup-2026-10-01.db nunc-backup-2026-10-02.db nunc-backup-2026-10-03.db"
    And I run a manual backup on 2026-10-04
    And I add the todo "Second state"
    And I run a manual backup on 2026-10-04
    Then the folder "backups" contains exactly "nunc-backup-2026-09-28.db nunc-backup-2026-09-29.db nunc-backup-2026-09-30.db nunc-backup-2026-10-01.db nunc-backup-2026-10-02.db nunc-backup-2026-10-03.db nunc-backup-2026-10-04.db"
    And the database "backups/nunc-backup-2026-10-04.db" has the todos "Second state"

  @integration @US-B04
  Scenario: The status reports count, total size and last backup
    Given the folder "backups" has the files:
      | name                      | size | modified             |
      | nunc-backup-2026-10-02.db | 1000 | 2026-10-02T09:00:00Z |
      | nunc-backup-2026-10-04.db | 2500 | 2026-10-04T08:30:00Z |
      | nunc-backup-2026-10-03.db | 500  | 2026-10-03T09:00:00Z |
    When I read the status of the folder "backups" at 2026-10-04T12:00:00Z
    Then the backup status is:
      | field          | value                |
      | backupCount    | 3                    |
      | totalSizeBytes | 4000                 |
      | lastBackupAt   | 2026-10-04T08:30:00Z |
      | isOutdated     | false                |

  @integration @US-B04
  Scenario: The status reports an empty backup directory
    When I read the status of the folder "backups" at 2026-10-04T12:00:00Z
    Then the backup status is:
      | field          | value |
      | backupCount    | 0     |
      | totalSizeBytes | 0     |
      | lastBackupAt   |       |
      | isOutdated     | true  |

  @integration @US-B04
  Scenario Outline: The status warns when the last backup is older than two days
    Given the folder "backups" has the files:
      | name                      | size | modified   |
      | nunc-backup-2026-10-01.db | 10   | <modified> |
    When I read the status of the folder "backups" at 2026-10-04T10:00:00Z
    Then the backup status is:
      | field      | value      |
      | isOutdated | <outdated> |

    Examples:
      | modified             | outdated |
      | 2026-10-02T10:00:00Z | false    |
      | 2026-10-02T09:59:59Z | true     |

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
