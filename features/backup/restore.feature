Feature: Restore
  Source: US-Restore (R01-R03); nunc-go backup_list_test.go, backup_restore_test.go

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
