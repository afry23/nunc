Feature: Sync provider selection
  Source: US-WebDAV (US-Sync-Provider)

  @integration @US-Sync-Provider @wip
  Scenario: The default provider is none

  @ui @US-Sync-Provider @wip
  Scenario: The provider can be chosen between none, Google Drive and WebDAV

  @ui @US-Sync-Provider @wip
  Scenario: Only one provider can be active

  @ui @US-Sync-Provider @wip
  Scenario: Switching a configured provider asks for confirmation

  @ui @US-Sync-Provider @wip
  Scenario: The active provider is highlighted

  @ui @US-Sync-Provider @wip
  Scenario: Provider fields are disabled when no sync is selected

  @integration @US-Sync-Provider @wip
  Scenario: The selection is persisted immediately

  @ui @US-Sync-Provider @mobile @wip
  Scenario: Mobile offers only none and WebDAV
