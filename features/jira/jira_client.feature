Feature: Jira client
  Source: ADR 0038
  The client talks to Jira Cloud and Data Center (wiremock). The token never appears in errors.

  @integration @jira @wip @US-J01
  Scenario: Jira Cloud authenticates with e-mail and API token

  @integration @jira @wip @US-J01
  Scenario: Jira Data Center authenticates with a bearer token

  @integration @jira @wip @US-J01
  Scenario: A search returns the matching tickets

  @integration @jira @wip @US-J04
  Scenario Outline: Rejected credentials are reported as unauthorized

  @integration @jira @wip @US-J04
  Scenario: A server error is reported with its status

  @integration @jira @wip @US-J04
  Scenario: A response that is not valid JSON is reported as bad response

  @integration @jira @wip @US-J04
  Scenario: An unreachable server is reported as transport error

  @integration @jira @wip @US-J04
  Scenario: Without saved credentials the client is not configured

  @integration @jira @wip @US-J04
  Scenario: The connection test succeeds with valid credentials

  @integration @jira @wip @US-J04
  Scenario: The connection test fails with a wrong token

  @integration @jira @wip @US-J05
  Scenario: Saving stores the token in the vault and the rest in the settings

  @integration @jira @wip @US-J05
  Scenario: Clearing removes settings and token

  @integration @jira @wip @US-J05
  Scenario: Error messages and debug output contain no token
