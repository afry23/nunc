Feature: Periodic sync (desktop)
  Source: REQ-Sync-Periodic; a desktop app that stays open picks up changes from other devices without user action

  @integration @sync @REQ-Sync-Periodic
  Scenario: The sync runs repeatedly at the chosen interval
    Given a sync coordinator whose runs take 0 ms
    And the coordinator runs on desktop
    When the sync interval is set to 100 ms
    Then within 5000 ms at least 3 sync runs have happened

  @integration @sync @REQ-Sync-Periodic
  Scenario: The first periodic run comes after a full interval
    Given a sync coordinator whose runs take 0 ms
    And the coordinator runs on desktop
    When the sync interval is set to 400 ms
    And I wait 150 ms
    Then no sync run has happened yet

  @integration @sync @REQ-Sync-Periodic
  Scenario: Switching the interval off stops the runs
    Given a sync coordinator whose runs take 0 ms
    And the coordinator runs on desktop
    When the sync interval is set to 100 ms
    And the sync interval is switched off
    And I wait 400 ms
    Then no sync run has happened yet

  @integration @sync @REQ-Sync-Periodic
  Scenario: A new interval replaces the old one
    Given a sync coordinator whose runs take 0 ms
    And the coordinator runs on desktop
    When the sync interval is set to 5000 ms
    And the sync interval is set to 100 ms
    Then within 5000 ms at least 2 sync runs have happened

  @integration @sync @REQ-Sync-Periodic
  Scenario: A periodic run never overlaps a running sync
    Given a sync coordinator whose runs take 250 ms
    And the coordinator runs on desktop
    When the sync interval is set to 50 ms
    And I wait 1000 ms
    Then at most 1 sync run was active at a time
    And within 2000 ms at least 2 sync runs have happened

  @integration @sync @REQ-Sync-Periodic
  Scenario: A failing run does not stop the schedule
    Given a sync coordinator whose runs take 0 ms
    And the coordinator runs on desktop
    And the next run fails with "Verbindung abgelehnt – Server nicht erreichbar"
    When the sync interval is set to 100 ms
    Then within 5000 ms at least 3 sync runs have happened

  @integration @sync @REQ-Sync-Periodic
  Scenario: Mobile devices ignore the interval
    Given a sync coordinator whose runs take 0 ms
    And the coordinator runs on mobile
    When the sync interval is set to 50 ms
    And I wait 400 ms
    Then no sync run has happened yet

  @ui @REQ-Sync-Periodic
  Scenario: The desktop offers the interval once a provider is chosen
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    Then I see the radio group "Automatisch synchronisieren" with the options "Aus, 5 min, 15 min, 30 min"
    And the option "15 min" is selected

  @ui @REQ-Sync-Periodic
  Scenario: Without a provider there is nothing to schedule
    When I open the app at "/settings"
    Then I do not see the radio group "Automatisch synchronisieren"

  @ui @REQ-Sync-Periodic
  Scenario: Choosing an interval is saved at once
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    And I click "30 min"
    Then the backend received "save_settings" with:
      """
      { "settings": { "syncIntervalMin": 30 } }
      """
    And the option "30 min" is selected

  @ui @REQ-Sync-Periodic
  Scenario: The sync can be switched off
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    And I click "Aus"
    Then the backend received "save_settings" with:
      """
      { "settings": { "syncIntervalMin": 0 } }
      """

  @ui @REQ-Sync-Periodic @mobile
  Scenario: Mobile devices have no interval setting
    Given the platform is "mobile"
    And the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    Then I do not see the radio group "Automatisch synchronisieren"
