Feature: Local interface settings
  Source: nunc extension; other programs on the same desktop read and write tasks over a token-protected HTTP interface

  @ui @REQ-API @desktop
  Scenario: The interface is off by default
    Given the platform is "desktop"
    When I open the app at "/settings"
    Then I see "Lokale Schnittstelle"
    And I see "Die Schnittstelle ist aus."
    And I do not see "Token anzeigen"

  @ui @REQ-API @desktop
  Scenario: Switching the interface on is saved with the settings
    Given the platform is "desktop"
    When I open the app at "/settings"
    And I switch on "Lokale Schnittstelle aktivieren"
    Then I see "Mit „Speichern“ übernehmen."
    When I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "apiEnabled": true, "apiPort": 42820 } }
      """
    And I see "Läuft auf http://127.0.0.1:42820"

  @ui @REQ-API @desktop
  Scenario: The port can be changed
    Given the platform is "desktop"
    And the setting "apiEnabled" is "true"
    When I open the app at "/settings"
    And I enter "43000" into the field "Port"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "apiEnabled": true, "apiPort": 43000 } }
      """
    And I see "Läuft auf http://127.0.0.1:43000"

  @ui @REQ-API @desktop
  Scenario: The token is only shown on request
    Given the platform is "desktop"
    And the setting "apiEnabled" is "true"
    When I open the app at "/settings"
    Then the field "Zugriffstoken" does not exist
    When I click "Token anzeigen"
    Then the field "Zugriffstoken" has the value "test-token"
    And I see "Authorization: Bearer"
    When I click "Token verbergen"
    Then the field "Zugriffstoken" does not exist

  @ui @REQ-API @desktop
  Scenario: A new token needs a confirmation
    Given the platform is "desktop"
    And the setting "apiEnabled" is "true"
    When I open the app at "/settings"
    And I click "Token neu erzeugen"
    Then I see "Programme mit dem alten Token verlieren sofort den Zugriff."
    And the backend did not receive "regenerate_api_token"
    When I click "Neu erzeugen"
    Then the backend received "regenerate_api_token"
    And I see "Neues Token erzeugt."
    And the field "Zugriffstoken" has the value "new-token"

  @ui @REQ-API @desktop
  Scenario: Cancelling the confirmation keeps the token
    Given the platform is "desktop"
    And the setting "apiEnabled" is "true"
    When I open the app at "/settings"
    And I click "Token neu erzeugen"
    And I click "Abbrechen"
    Then the backend did not receive "regenerate_api_token"

  @ui @REQ-API @desktop
  Scenario: A server that cannot start is explained
    Given the platform is "desktop"
    And the setting "apiEnabled" is "true"
    And the local interface cannot start because "Port 42820 nicht nutzbar"
    When I open the app at "/settings"
    Then I see "Die Schnittstelle läuft nicht: Port 42820 nicht nutzbar"
    And I do not see "Token anzeigen"

  @ui @REQ-API @mobile
  Scenario: The interface does not exist on mobile
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I do not see "Lokale Schnittstelle"
    And the backend did not receive "get_api_status"

  @ui @REQ-API @desktop
  Scenario: The interface can be used in English
    Given the platform is "desktop"
    And the setting "apiEnabled" is "true"
    And the setting "language" is "en"
    When I open the app at "/settings"
    Then I see "Local interface"
    And I see "Running on http://127.0.0.1:42820"
