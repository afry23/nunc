Feature: Sync provider selection
  Source: US-WebDAV (US-Sync-Provider)

  @integration @US-Sync-Provider
  Scenario: The default provider is none
    Given no database exists yet
    When nunc opens the database
    And I load the settings
    Then the loaded settings are:
      | setting      | value |
      | syncProvider | none  |

  @integration @US-Sync-Provider
  Scenario: The selection is persisted immediately
    Given no database exists yet
    When nunc opens the database
    And I select the sync provider "webdav"
    Then the settings table contains:
      | key           | value  |
      | sync_provider | webdav |
    When I load the settings
    Then the loaded settings are:
      | setting      | value  |
      | syncProvider | webdav |

  @integration @US-Sync-Provider
  Scenario: Choosing the provider leaves the other settings alone
    Given no database exists yet
    When nunc opens the database
    And the settings table has "pomodoro_min" = "40"
    And I select the sync provider "webdav"
    And I load the settings
    Then the loaded settings are:
      | setting     | value |
      | pomodoroMin | 40    |

  @integration @US-Sync-Provider
  Scenario: A provider that no longer exists falls back to none
    Given no database exists yet
    When nunc opens the database
    And the settings table has "sync_provider" = "google_drive"
    And I load the settings
    Then the loaded settings are:
      | setting      | value |
      | syncProvider | none  |

  @ui @US-Sync-Provider
  Scenario: The provider can be chosen between none and WebDAV
    When I open the app at "/settings"
    Then I see the radio group "Sync-Anbieter" with the options "Kein Sync, WebDAV"

  @ui @US-Sync-Provider
  Scenario: Only one provider can be active
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    Then the option "WebDAV" is selected
    And the option "Kein Sync" is not selected

  @ui @US-Sync-Provider
  Scenario: Switching a configured provider asks for confirmation
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    And I click "Kein Sync"
    Then I see a dialog with the text "Die Synchronisierung mit WebDAV wird beendet"
    And the backend did not receive "set_sync_provider"
    When I click "Abbrechen"
    Then the option "WebDAV" is selected
    When I click "Kein Sync"
    And I click "Anbieter wechseln"
    Then the backend received "set_sync_provider" with:
      """
      { "provider": "none" }
      """
    And the option "Kein Sync" is selected

  @ui @US-Sync-Provider
  Scenario: Choosing a provider while none is active needs no confirmation and is saved at once
    When I open the app at "/settings"
    And I click "WebDAV"
    Then I do not see a dialog
    And the backend received "set_sync_provider" with:
      """
      { "provider": "webdav" }
      """
    And the option "WebDAV" is selected

  @ui @US-Sync-Provider
  Scenario: A failing selection keeps the old provider and shows the error
    Given the provider selection fails with "invalid_input": "Anbieter nicht verfügbar"
    When I open the app at "/settings"
    And I click "WebDAV"
    Then I see "Anbieter nicht verfügbar"
    And the option "Kein Sync" is selected

  @ui @US-Sync-Provider
  Scenario: The active provider is highlighted
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    Then the option "WebDAV" is selected

  @ui @US-Sync-Provider
  Scenario: Provider fields are disabled when no sync is selected
    When I open the app at "/settings"
    Then the field "Server-URL" is disabled
    And the field "Benutzername" is disabled
    And the button "Verbindung testen" is disabled
    And I see "Wähle WebDAV als Anbieter, um die Verbindung zu bearbeiten."
    When I click "WebDAV"
    Then the field "Server-URL" is enabled
    And the field "Benutzername" is enabled

  @ui @US-Sync-Provider @mobile
  Scenario: Mobile offers only none and WebDAV
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I see the radio group "Sync-Anbieter" with the options "Kein Sync, WebDAV"

  @integration @sync @US-Sync-Provider
  Scenario: Each provider shows its own time of the last sync
    Given a sync coordinator whose runs take 0 ms
    When I sync now
    And I replace the last sync time with "2026-10-03T08:00:00Z"
    Then the sync status is "idle 2026-10-03T08:00:00Z -"
    When I replace the last sync time with nothing
    Then the sync status is "idle - -"

  @integration @sync @US-Sync-Provider
  Scenario: Changing the provider clears an old error
    Given a sync coordinator whose runs take 0 ms
    And the next run fails with "Verbindung abgelehnt – Server nicht erreichbar"
    When I sync now
    And I replace the last sync time with "2026-10-03T08:00:00Z"
    Then the sync status is "idle 2026-10-03T08:00:00Z -"

  @integration @sync @US-Sync-Provider
  Scenario: A running sync keeps its status when the provider is changed
    Given a sync coordinator whose runs take 400 ms
    When the app starts
    And I wait 100 ms
    And I replace the last sync time with "2026-10-03T08:00:00Z"
    Then the sync status is "syncing - -"
