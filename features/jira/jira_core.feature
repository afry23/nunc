Feature: Jira import – core logic
  Source: ADR 0038, docs/jira-integration.md
  Pure functions that build the search query, the todo line and the ticket link, and that parse Jira responses.

  @domain @wip @US-J01
  Scenario: An empty search lists my open tickets

  @domain @wip @US-J01
  Scenario Outline: A ticket key searches for exactly that ticket

  @domain @wip @US-J01
  Scenario: Free text searches the text of open tickets

  @domain @wip @US-J01
  Scenario Outline: Quotes and backslashes in the search text are escaped

  @domain @wip @property @US-J01
  Scenario: Escaped search text never contains an unescaped quote

  @domain @wip @US-J02
  Scenario Outline: The ticket URL is built from the base URL and the key

  @domain @wip @US-J02
  Scenario Outline: The todo line links the key and keeps the summary

  @domain @wip @US-J02
  Scenario: A ticket without a summary becomes only the link

  @domain @wip @US-J01
  Scenario Outline: A search response is parsed for Jira Cloud and Data Center

  @domain @wip @US-J01
  Scenario: At most 50 hits are returned

  @domain @wip @US-J01
  Scenario: A malformed response is rejected

  @domain @wip @US-J03
  Scenario: An open todo with the ticket link counts as imported

  @domain @wip @US-J03
  Scenario: A completed todo does not count as imported

  @domain @wip @US-J03
  Scenario: A ticket whose key is only a prefix of another key is not imported

  @domain @wip @US-J01
  Scenario Outline: The API path depends on the Jira kind
