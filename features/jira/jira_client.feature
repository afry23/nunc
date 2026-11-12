Feature: Jira client
  Source: ADR 0038
  The client talks to Jira Cloud and Data Center (wiremock). The token never appears in errors.

  @integration @jira @US-J01
  Scenario: Jira Cloud authenticates with e-mail and API token
    Given a Jira Cloud server for "me@firma.de" with the token "secret-1"
    When I search Jira for ""
    Then I get 2 Jira hits
    And the Jira server received the JQL "assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC"
    And the Jira server was asked for 50 results with the field "summary"

  @integration @jira @US-J01
  Scenario: Jira Data Center authenticates with a bearer token
    Given a Jira Data Center server with the token "pat-1"
    When I search Jira for "PROJ-1"
    Then I get 2 Jira hits
    And the Jira server received the JQL "key = \"PROJ-1\""

  @integration @jira @US-J01
  Scenario: A search returns the tickets with their links
    Given a Jira Cloud server for "me@firma.de" with the token "secret-1"
    When I search Jira for "login"
    Then ticket 1 of the Jira hits is "PROJ-1" "First"
    And ticket 2 of the Jira hits is "PROJ-2" "Second"
    And the Jira server received the JQL "text ~ \"login\" AND statusCategory != Done ORDER BY updated DESC"

  @integration @jira @US-J04
  Scenario Outline: Rejected credentials are reported as unauthorized
    Given a Jira server that answers with status <status>
    When I search Jira for ""
    Then the Jira call fails with "unauthorized"

    Examples:
      | status |
      | 401    |
      | 403    |

  @integration @jira @US-J04
  Scenario: A wrong token is rejected
    Given a Jira Cloud server for "me@firma.de" with the token "secret-1"
    And the client uses the token "wrong"
    When I search Jira for ""
    Then the Jira call fails with "unauthorized"

  @integration @jira @US-J04
  Scenario: A wrong e-mail is rejected
    Given a Jira Cloud server for "me@firma.de" with the token "secret-1"
    And the client uses the e-mail "other@firma.de"
    When I search Jira for ""
    Then the Jira call fails with "unauthorized"

  @integration @jira @US-J04
  Scenario: A server error is reported with its status
    Given a Jira server that answers with status 503
    When I search Jira for ""
    Then the Jira call fails with "http 503"

  @integration @jira @US-J04
  Scenario: A response that is not valid JSON is reported as bad response
    Given a Jira server that answers with the body "<html>login</html>"
    When I search Jira for ""
    Then the Jira call fails with "bad response"

  @integration @jira @US-J04
  Scenario: An unreachable server is reported as transport error
    Given no Jira server is running
    When I search Jira for ""
    Then the Jira call fails with "transport"

  @integration @jira @US-J04
  Scenario: A trailing slash in the base URL is tolerated
    Given a Jira Data Center server with the token "pat-1"
    And the base URL has a trailing slash
    When I search Jira for ""
    Then I get 2 Jira hits

  @integration @jira @US-J04
  Scenario: The connection test succeeds with valid credentials
    Given a Jira Cloud server for "me@firma.de" with the token "secret-1"
    When I test the Jira connection
    Then the Jira connection test succeeds

  @integration @jira @US-J04
  Scenario: The connection test fails with a wrong token
    Given a Jira Data Center server with the token "pat-1"
    And the client uses the token "wrong"
    When I test the Jira connection
    Then the Jira call fails with "unauthorized"

  @integration @jira @US-J04
  Scenario: Without saved credentials the client is not configured
    When I load the Jira credentials
    Then the Jira call fails with "not configured"

  @integration @jira @US-J05
  Scenario: Saving stores the token in the vault and the rest in the settings
    When I save the Jira configuration "cloud" "https://firma.atlassian.net" "me@firma.de" with the token "secret-1"
    Then the vault holds the Jira token "secret-1"
    And the saved Jira configuration is "cloud" "https://firma.atlassian.net" "me@firma.de"
    And the database does not contain "secret-1"
    And the Jira credentials can be loaded

  @integration @jira @US-J05
  Scenario: Saving without a token keeps the stored token
    Given the Jira configuration "cloud" "https://firma.atlassian.net" "me@firma.de" with the token "secret-1" is saved
    When I save the Jira configuration "data_center" "https://jira.firma.de" "" without a token
    Then the vault holds the Jira token "secret-1"
    And the saved Jira configuration is "data_center" "https://jira.firma.de" ""

  @integration @jira @US-J05
  Scenario Outline: An incomplete configuration is not usable
    Given the Jira configuration "<kind>" "<url>" "<email>" with the token "<token>" is saved
    When I load the Jira credentials
    Then the Jira call fails with "not configured"

    Examples:
      | kind  | url                         | email    | token    |
      | cloud |                             | me@x.net | secret-1 |
      | cloud | https://firma.atlassian.net |          | secret-1 |
      | cloud | https://firma.atlassian.net | me@x.net |          |

  @integration @jira @US-J05
  Scenario: Clearing removes settings and token
    Given the Jira configuration "cloud" "https://firma.atlassian.net" "me@firma.de" with the token "secret-1" is saved
    When I clear the Jira configuration
    Then the vault holds no Jira token
    And the saved Jira configuration is "cloud" "" ""
    And the Jira call fails when loading the credentials with "not configured"

  @integration @jira @US-J05
  Scenario: Error messages and debug output contain no token
    Given a Jira Cloud server for "me@firma.de" with the token "secret-1"
    And the client uses the token "leaky-token"
    When I search Jira for ""
    Then the Jira error text does not contain "leaky-token"
    And the Jira credentials debug output does not contain "leaky-token"
