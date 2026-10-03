Feature: Sync merge
  Source: US-S04, US-Sync-02-WebDAV; nunc-go sync_merge_test.go, sync_webdav_merge_test.go

  @domain @US-S04 @wip
  Scenario: A todo that exists only remotely is added

  @domain @US-S04 @wip
  Scenario: A todo that exists only locally is kept

  @domain @US-S04 @wip
  Scenario: The remote todo wins when it is newer

  @domain @US-S04 @wip
  Scenario: The local todo wins when it is newer

  @domain @US-S04 @wip
  Scenario: Todos without a sync id are skipped

  @domain @US-S04 @wip
  Scenario: The deleted flag is propagated

  @domain @US-S04 @wip
  Scenario: Conflicting todo versions are counted

  @domain @US-S04 @wip
  Scenario: Sessions are appended and never overwritten

  @domain @US-S04 @wip
  Scenario: Sessions are not duplicated

  @domain @US-S04 @wip
  Scenario: Duplicate remote sessions are removed

  @domain @US-S04 @wip
  Scenario: Sessions with a different device id but identical payload are not duplicated

  @domain @US-S04 @wip
  Scenario: Merging with an empty remote state keeps the local state

  @domain @US-S04 @wip
  Scenario: Merging with an empty local state takes over the remote state

  @domain @US-S04 @property @wip
  Scenario: Merging is idempotent

  @domain @US-S04 @property @wip
  Scenario: Merging is commutative apart from conflict counts
