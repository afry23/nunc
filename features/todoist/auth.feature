Feature: Todoist connection
  Source: US-Todoist (US-Todoist-Auth); desktop only in v1

  @ui @US-Todoist-Auth @desktop
  Scenario: The Todoist section shows the connection status
    When I open the app at "/settings"
    Then I see the heading "Todoist"
    And I see "Nicht verbunden"
    And I see the buttons "Mit Todoist verbinden"
    And I do not see the button "Todoist trennen"

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: Connecting opens the browser with the Todoist authorization page
    Given a Todoist server
    When I connect Todoist
    Then the browser was opened once with the Todoist authorization page

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: The callback exchanges the code for access and refresh tokens
    Given a Todoist server
    When I connect Todoist
    Then the Todoist token request carries the code and the verifier of the challenge
    And the Todoist status shows the account "me@example.org" with an avatar

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: The tokens are stored in the secret store
    Given a Todoist server
    When I connect Todoist
    Then the secret store holds a Todoist connection
    And the database does not contain "todoist-access-1"
    And the database does not contain "todoist-refresh-1"

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A first sync runs after connecting
    Given a Todoist server
    And no database exists yet
    And the local todos "Local task"
    And the coordinator runs the Todoist runner
    When I connect Todoist
    And the account has just been connected
    Then within 3000 ms Todoist holds the task "Local task"

  @ui @US-Todoist-Auth @desktop
  Scenario: The connected account is shown with e-mail and avatar
    Given Todoist is connected
    When I open the app at "/settings"
    Then I see "Verbunden als me@example.org"
    And I see the image "Avatar" from "https://avatars.example/me.png"
    And I do not see the button "Mit Todoist verbinden"

  @ui @US-Todoist-Auth @desktop
  Scenario: Disconnecting asks for confirmation
    Given Todoist is connected
    When I open the app at "/settings"
    And I click "Todoist trennen"
    Then I see a dialog with the text "Todoist-Verbindung trennen?"
    And I see a dialog with the text "Lokale Todos bleiben erhalten, werden aber nicht mehr mit Todoist synchronisiert."
    And the backend did not receive "disconnect_todoist"
    When I click "Abbrechen"
    Then I see "Verbunden als me@example.org"
    When I click "Todoist trennen"
    And I click "Trennen"
    Then the backend received "disconnect_todoist"
    And I see "Nicht verbunden"
    And I see the buttons "Mit Todoist verbinden"

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: Disconnecting deletes the tokens and keeps local todos
    Given a Todoist server
    And Todoist is connected
    And no database exists yet
    And the local todos "Keep me"
    When I disconnect Todoist
    Then the secret store holds no Todoist connection
    And the local todos are "Keep me"

  @ui @US-Todoist-Auth @desktop
  Scenario: A token exchange error is shown inline and can be retried
    Given connecting Todoist fails with "oauth": "Token-Austausch mit Todoist fehlgeschlagen: invalid_grant"
    When I open the app at "/settings"
    And I click "Mit Todoist verbinden"
    Then I see "Token-Austausch mit Todoist fehlgeschlagen: invalid_grant"
    And I do not see a dialog
    And the button "Mit Todoist verbinden" is enabled
    When the Todoist login works again
    And I click "Mit Todoist verbinden"
    Then I see "Verbunden als me@example.org"

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: An expired access token is refreshed automatically
    Given a Todoist server
    And Todoist is connected
    And the Todoist access token has expired
    And no database exists yet
    When I sync with Todoist
    Then the sync succeeds
    And Todoist issued 1 tokens by renewal
    And the stored Todoist refresh token is "todoist-refresh-1"

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A denied authorization is reported
    Given a Todoist server
    And the user denies the authorization
    When I connect Todoist
    Then connecting failed with "Die Anmeldung bei Todoist wurde abgebrochen: access_denied"
    And the secret store holds no Todoist connection

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A failing token exchange is reported
    Given a Todoist server
    And Todoist rejects the authorization code with "invalid_grant"
    When I connect Todoist
    Then connecting failed with "Token-Austausch mit Todoist fehlgeschlagen: invalid_grant"
    And the secret store holds no Todoist connection

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A build without Todoist credentials cannot connect
    Given a Todoist server
    And the app has no Todoist credentials
    When I connect Todoist
    Then connecting failed with "In diesem Build sind keine Todoist-Zugangsdaten hinterlegt (NUNC_TODOIST_CLIENT_ID, NUNC_TODOIST_CLIENT_SECRET)."
    And the browser was not opened

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: Tokens that never expire are never renewed
    Given a Todoist server
    And Todoist issues tokens that never expire
    And no database exists yet
    When I connect Todoist
    And I sync with Todoist
    Then the sync succeeds
    And Todoist issued 0 tokens by renewal
    And the stored Todoist token never expires

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A token that Todoist rejects is renewed and the request repeated
    Given a Todoist server
    And Todoist is connected
    And Todoist no longer accepts the access token
    And no database exists yet
    When I sync with Todoist
    Then the sync succeeds
    And Todoist issued 1 tokens by renewal

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A connection that cannot be renewed asks for a new login
    Given a Todoist server
    And Todoist is connected
    And Todoist no longer accepts the access token
    And Todoist refuses to renew tokens
    And no database exists yet
    When I sync with Todoist
    Then the sync fails with "Todoist: Anmeldung abgelaufen – bitte neu verbinden"

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A sync without a connection says so
    Given a Todoist server
    And no database exists yet
    When I sync with Todoist
    Then the sync fails with "Nicht mit Todoist verbunden"
    And Todoist was not contacted

  @ui @US-Todoist-Auth @desktop
  Scenario: Connecting from the settings shows the account
    When I open the app at "/settings"
    And I click "Mit Todoist verbinden"
    Then the backend received "connect_todoist"
    And I see "Verbunden als me@example.org"

  @ui @US-Todoist-Auth @desktop
  Scenario: A build without Todoist credentials cannot connect
    Given this build has no Todoist credentials
    When I open the app at "/settings"
    Then I see "In diesem Build sind keine Todoist-Zugangsdaten hinterlegt."
    And the button "Mit Todoist verbinden" is disabled

  @ui @US-Todoist-Auth @mobile
  Scenario: Todoist is not offered on mobile
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I do not see "Mit Todoist verbinden"
    And I do not see "Todoist"
