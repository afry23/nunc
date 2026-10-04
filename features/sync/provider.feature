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

  @domain @US-Sync-Provider
  Scenario Outline: Google Drive is only available on the desktop
    Then the sync provider "<provider>" is <result> on <platform>

    Examples:
      | provider     | platform | result      |
      | none         | desktop  | available   |
      | none         | mobile   | available   |
      | webdav       | desktop  | available   |
      | webdav       | mobile   | available   |
      | google_drive | desktop  | available   |
      | google_drive | mobile   | unavailable |

  @ui @US-Sync-Provider
  Scenario: The provider can be chosen between none and WebDAV and Google Drive follows later
    When I open the app at "/settings"
    Then I see the radio group "Sync-Anbieter" with the options "Kein Sync, Google Drive, WebDAV"
    And the option "Google Drive" is disabled
    And I see "Google Drive folgt in einer späteren Version."

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

  @ui @US-Sync-Provider @wip
  Scenario: Provider fields are disabled when no sync is selected

  @ui @US-Sync-Provider @mobile
  Scenario: Mobile offers only none and WebDAV
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I see the radio group "Sync-Anbieter" with the options "Kein Sync, WebDAV"
