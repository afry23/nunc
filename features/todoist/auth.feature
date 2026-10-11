Feature: Todoist connection
  Source: US-Todoist (US-Todoist-Auth); desktop only in v1. Signing in uses the personal API token (ADR 0020, token.feature)

  @ui @US-Todoist-Auth @desktop
  Scenario: The Todoist section shows the connection status
    When I open the app at "/settings"
    Then I see the heading "Todoist"
    And I see "Nicht verbunden"
    And I see the buttons "Mit Todoist verbinden"
    And I do not see the button "Todoist trennen"

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A first sync runs after connecting
    Given a Todoist server
    And no database exists yet
    And the local todos "Local task"
    And the coordinator runs the Todoist runner
    When I connect Todoist with the token "personal-1"
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

  @integration @sync @US-Todoist-Auth @desktop
  Scenario: A sync without a connection says so
    Given a Todoist server
    And no database exists yet
    When I sync with Todoist
    Then the sync fails with "Nicht mit Todoist verbunden"
    And Todoist was not contacted

  @ui @US-Todoist-Auth @mobile
  Scenario: Todoist is not offered on mobile
    Given the platform is "mobile"
    When I open the app at "/settings"
    Then I do not see "Mit Todoist verbinden"
    And I do not see "Todoist"
