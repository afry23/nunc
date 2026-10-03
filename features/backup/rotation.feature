Feature: Backup rotation
  Source: US-B02; nunc-go backup_rotate_test.go

  @domain @US-B02 @wip
  Scenario: The oldest backup is deleted when over the limit

  @domain @US-B02 @wip
  Scenario: Several oldest backups are deleted when far over the limit

  @domain @US-B02 @wip
  Scenario: All backups are kept at the limit

  @domain @US-B02 @wip
  Scenario: All backups are kept under the limit

  @domain @US-B02 @wip
  Scenario: An empty directory is a no-op

  @domain @US-B02 @wip
  Scenario: Only seven daily backups are kept
