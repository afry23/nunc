Feature: Todoist with a personal API token
  Source: ADR 0020; replaces the OAuth login with the built-in client; desktop only in v1

  @ui @US-Todoist-Token @desktop
  Scenario: The Todoist section asks for the API token
    When I open the app at "/settings"
    Then I see the heading "Todoist"
    And I see the fields "API-Token"
    And the field "API-Token" is masked
    And I see "Einstellungen → Integrationen → Entwickler"
    And I see "Das Token hat vollen Zugriff auf dein Todoist-Konto"

  @ui @US-Todoist-Token @desktop
  Scenario: The guide links to the Todoist developer settings
    When I open the app at "/settings"
    And I click "Todoist-Einstellungen öffnen"
    Then the browser opened "https://app.todoist.com/app/settings/integrations/developer"

  @ui @US-Todoist-Token @desktop
  Scenario: Connecting with a token shows the account
    When I open the app at "/settings"
    And I enter "0123456789abcdef" into the field "API-Token"
    And I click "Mit Todoist verbinden"
    Then the backend received "connect_todoist_token"
    And I see "Verbunden als me@example.org"
    And the field "API-Token" does not exist

  @ui @US-Todoist-Token @desktop
  Scenario: Connecting needs a token
    When I open the app at "/settings"
    Then the button "Mit Todoist verbinden" is disabled

  @ui @US-Todoist-Token @desktop
  Scenario: A rejected token is shown inline and can be corrected
    Given connecting Todoist fails with "todoist_token_invalid": "Todoist hat das Token abgelehnt. Bitte prüfe es in den Todoist-Einstellungen."
    When I open the app at "/settings"
    And I enter "wrong" into the field "API-Token"
    And I click "Mit Todoist verbinden"
    Then I see "Todoist hat das Token abgelehnt. Bitte prüfe es in den Todoist-Einstellungen."
    And I do not see a dialog

  @integration @sync @US-Todoist-Token @desktop
  Scenario: Connecting checks the token and stores it
    Given a Todoist server
    When I connect Todoist with the token "  personal-1  "
    Then Todoist received "GET /api/v1/user" with the bearer token "personal-1"
    And the Todoist status shows the account "me@example.org" with an avatar
    And the secret store holds a Todoist connection that never expires
    And the database does not contain "personal-1"

  @integration @sync @US-Todoist-Token @desktop
  Scenario: A token that Todoist rejects is not stored
    Given a Todoist server
    And Todoist rejects the token "wrong"
    When I connect Todoist with the token "wrong"
    Then connecting failed with "Todoist hat das Token abgelehnt. Bitte prüfe es in den Todoist-Einstellungen."
    And the secret store holds no Todoist connection

  @integration @sync @US-Todoist-Token @desktop
  Scenario: An empty token is refused without contacting Todoist
    Given a Todoist server
    When I connect Todoist with the token "   "
    Then connecting failed with "Todoist hat das Token abgelehnt. Bitte prüfe es in den Todoist-Einstellungen."
    And Todoist was not contacted

  @integration @sync @US-Todoist-Token @desktop
  Scenario: The sync uses the personal token and never renews it
    Given a Todoist server
    And no database exists yet
    When I connect Todoist with the token "personal-1"
    And I sync with Todoist
    Then the sync succeeds
    And every Todoist request carried the bearer token "personal-1"
    And Todoist issued 0 tokens by renewal

  @integration @sync @US-Todoist-Token @desktop
  Scenario: A token revoked in Todoist asks for a new one
    Given a Todoist server
    And no database exists yet
    When I connect Todoist with the token "personal-1"
    And Todoist rejects the token "personal-1"
    And I sync with Todoist
    Then the sync fails with "Todoist: Token ungültig – bitte neu eintragen"

  @integration @sync @US-Todoist-Token @desktop
  Scenario: An OAuth connection from an earlier version keeps working until it expires
    Given a Todoist server
    And Todoist is connected
    And no database exists yet
    When I sync with Todoist
    Then the sync succeeds

  @integration @sync @US-Todoist-Token @desktop
  Scenario: An expired OAuth connection from an earlier version asks for a token
    Given a Todoist server
    And Todoist is connected
    And the Todoist access token has expired
    And no database exists yet
    When I sync with Todoist
    Then the sync fails with "Todoist: Token ungültig – bitte neu eintragen"
    And Todoist issued 0 tokens by renewal

  @integration @sync @US-Todoist-Token @desktop
  Scenario: Replacing an OAuth connection with a token keeps the mappings
    Given a Todoist server
    And Todoist is connected
    And Todoist holds the tasks:
      | content |
      | Mapped  |
    And no database exists yet
    And the local todos "Mapped"
    And the todo "Mapped" is mapped to the Todoist task "Mapped"
    When I connect Todoist with the token "personal-1"
    And I sync with Todoist
    Then the sync succeeds
    And Todoist received 0 create, 0 update and 0 close requests
    And there are 1 Todoist mappings
