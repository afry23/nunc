Feature: Backup rotation
  Source: US-B02; nunc-go backup_rotate_test.go
  Only files named nunc-backup-YYYY-MM-DD.db are backups; the oldest by date are deleted first.

  @domain @US-B02
  Scenario: The oldest backup is deleted when over the limit
    Given the backup files:
      | name |
      | nunc-backup-2025-06-01.db |
      | nunc-backup-2025-06-02.db |
      | nunc-backup-2025-06-03.db |
      | nunc-backup-2025-06-04.db |
      | nunc-backup-2025-06-05.db |
      | nunc-backup-2025-06-06.db |
      | nunc-backup-2025-06-07.db |
      | nunc-backup-2025-06-08.db |
    When I rotate the backups
    Then the files to delete are "nunc-backup-2025-06-01.db"

  @domain @US-B02
  Scenario: Several oldest backups are deleted when far over the limit
    Given the backup files:
      | name |
      | nunc-backup-2025-06-01.db |
      | nunc-backup-2025-06-02.db |
      | nunc-backup-2025-06-03.db |
      | nunc-backup-2025-06-04.db |
      | nunc-backup-2025-06-05.db |
      | nunc-backup-2025-06-06.db |
      | nunc-backup-2025-06-07.db |
      | nunc-backup-2025-06-08.db |
      | nunc-backup-2025-06-09.db |
      | nunc-backup-2025-06-10.db |
    When I rotate the backups
    Then the files to delete are "nunc-backup-2025-06-01.db nunc-backup-2025-06-02.db nunc-backup-2025-06-03.db"

  @domain @US-B02
  Scenario: All backups are kept at the limit
    Given the backup files:
      | name |
      | nunc-backup-2025-06-01.db |
      | nunc-backup-2025-06-02.db |
      | nunc-backup-2025-06-03.db |
      | nunc-backup-2025-06-04.db |
      | nunc-backup-2025-06-05.db |
      | nunc-backup-2025-06-06.db |
      | nunc-backup-2025-06-07.db |
    When I rotate the backups
    Then the files to delete are ""

  @domain @US-B02
  Scenario: All backups are kept under the limit
    Given the backup files:
      | name |
      | nunc-backup-2025-06-01.db |
      | nunc-backup-2025-06-02.db |
      | nunc-backup-2025-06-03.db |
    When I rotate the backups
    Then the files to delete are ""

  @domain @US-B02
  Scenario: An empty directory is a no-op
    Given the backup files:
      | name |
    When I rotate the backups
    Then the files to delete are ""

  @domain @US-B02
  Scenario: Only seven daily backups are kept
    Given the backup files:
      | name                    |
      | nunc-backup-2025-06-09.db |
      | nunc-backup-2025-06-03.db |
      | nunc-backup-2025-06-07.db |
      | nunc-backup-2025-06-01.db |
      | nunc-backup-2025-06-05.db |
      | nunc-backup-2025-06-08.db |
      | nunc-backup-2025-06-02.db |
      | nunc-backup-2025-06-06.db |
      | nunc-backup-2025-06-04.db |
    When I rotate the backups
    Then the files to delete are "nunc-backup-2025-06-01.db nunc-backup-2025-06-02.db"

  @domain @US-B02
  Scenario: Backups are ordered by date, not by modification time
    Given the backup files:
      | name                      | modified             |
      | nunc-backup-2025-12-30.db | 2026-03-01T00:00:00Z |
      | nunc-backup-2026-01-02.db | 2025-01-01T00:00:00Z |
      | nunc-backup-2025-12-31.db | 2026-02-01T00:00:00Z |
    When I rotate the backups keeping at most 2
    Then the files to delete are "nunc-backup-2025-12-30.db"

  @domain @US-B02
  Scenario: Other files in the folder are never rotated
    Given the backup files:
      | name                       |
      | before-restore-2025-06-01.db |
      | nunc-backup-manual.db      |
      | nunc-backup-2025-13-40.db  |
      | nunc-backup-2025-6-1.db    |
      | nunc-backup-2025-06-01.db.tmp |
      | notes.txt                  |
      | nunc-backup-2025-06-02.db  |
      | nunc-backup-2025-06-03.db  |
    When I rotate the backups keeping at most 1
    Then the files to delete are "nunc-backup-2025-06-02.db"

  @domain @US-B02
  Scenario Outline: The limit can be lowered to nothing
    Given the backup files:
      | name                      |
      | nunc-backup-2025-06-01.db |
      | nunc-backup-2025-06-02.db |
    When I rotate the backups keeping at most <max>
    Then the files to delete are "<deleted>"

    Examples:
      | max | deleted                                         |
      | 0   | nunc-backup-2025-06-01.db nunc-backup-2025-06-02.db |
      | 1   | nunc-backup-2025-06-01.db                       |
      | 2   |                                                 |
      | 3   |                                                 |

  @domain @property @US-B02
  Scenario: Rotation keeps exactly the newest backups
    Then the property "rotation keeps exactly the newest backups" holds for 300 random cases
