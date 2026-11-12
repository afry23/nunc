Feature: Jira import – core logic
  Source: ADR 0038, docs/jira-integration.md
  Pure functions that build the search query, the todo line and the ticket link, and that parse Jira responses.

  @domain @US-J01
  Scenario: An empty search lists my open tickets
    When I build the Jira query for ""
    Then the Jira query is 'assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC'

  @domain @US-J01
  Scenario: Spaces around a ticket key are ignored
    When I build the Jira query for "  proj-7  "
    Then the Jira query is 'key = "PROJ-7"'

  @domain @US-J01
  Scenario: A blank search is treated like an empty one
    When I build the Jira query for "   "
    Then the Jira query is 'assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC'

  @domain @US-J01
  Scenario Outline: A ticket key searches for exactly that ticket
    When I build the Jira query for "<input>"
    Then the Jira query is '<query>'

    Examples:
      | input     | query                  |
      | PROJ-123  | key = "PROJ-123"     |
      | proj-123  | key = "PROJ-123"     |
      | A1_B-42   | key = "A1_B-42"      |
      | AB-1      | key = "AB-1"         |

  @domain @US-J01
  Scenario Outline: Text that is not a ticket key is searched as free text
    When I build the Jira query for "<input>"
    Then the Jira query is '<query>'

    Examples:
      | input        | query                                                                 |
      | login        | text ~ "login" AND statusCategory != Done ORDER BY updated DESC     |
      | PROJ-        | text ~ "PROJ-" AND statusCategory != Done ORDER BY updated DESC     |
      | -123         | text ~ "-123" AND statusCategory != Done ORDER BY updated DESC      |
      | P-1          | text ~ "P-1" AND statusCategory != Done ORDER BY updated DESC       |
      | 1A-5         | text ~ "1A-5" AND statusCategory != Done ORDER BY updated DESC      |
      | PROJ-12a     | text ~ "PROJ-12a" AND statusCategory != Done ORDER BY updated DESC  |
      | PROJ 123     | text ~ "PROJ 123" AND statusCategory != Done ORDER BY updated DESC  |

  @domain @US-J01
  Scenario Outline: Quotes and backslashes in the search text are escaped
    When I escape "<input>" for JQL
    Then the escaped text is "<escaped>"

    Examples:
      | input   | escaped     |
      | plain   | plain       |
      | a{Q}b   | a{B}{Q}b    |
      | a{B}b   | a{B}{B}b    |
      | {Q}{B}  | {B}{Q}{B}{B} |

  @domain @property @US-J01
  Scenario: Escaped search text never contains an unescaped quote
    Then the property "escaped JQL text has no unescaped quote" holds for 200 random cases

  @domain @US-J02
  Scenario Outline: The ticket URL is built from the base URL and the key
    When I build the URL of "<key>" on "<base>"
    Then the Jira URL is "<url>"

    Examples:
      | base                           | key      | url                                         |
      | https://firma.atlassian.net    | PROJ-123 | https://firma.atlassian.net/browse/PROJ-123 |
      | https://firma.atlassian.net/   | PROJ-123 | https://firma.atlassian.net/browse/PROJ-123 |
      | https://jira.firma.de/jira//   | AB-1     | https://jira.firma.de/jira/browse/AB-1      |
      |  https://jira.firma.de         | AB-1     | https://jira.firma.de/browse/AB-1           |

  @domain @US-J02
  Scenario: The todo line links the key and keeps the summary
    When I build the todo line for the ticket "PROJ-123" "Login-Fehler beheben" on "https://firma.atlassian.net"
    Then the Jira todo line is "[PROJ-123](https://firma.atlassian.net/browse/PROJ-123) Login-Fehler beheben"

  @domain @US-J02
  Scenario: Line breaks and tabs in the summary become spaces
    When I build the todo line for a ticket with the summary "  Fix\tthe\nbug\r\nnow  "
    Then the Jira todo line is "[PROJ-1](https://x.net/browse/PROJ-1) Fix the bug  now"

  @domain @US-J02
  Scenario: A ticket without a summary becomes only the link
    When I build the todo line for a ticket with the summary " \n "
    Then the Jira todo line is "[PROJ-1](https://x.net/browse/PROJ-1)"

  @domain @US-J01
  Scenario Outline: A search response is parsed for Jira Cloud and Data Center
    When I parse the Jira response for "<base>":
      """
      {"issues":[{"key":"PROJ-1","fields":{"summary":"First"}},{"key":"PROJ-2","fields":{"summary":"Second"}}],"total":2}
      """
    Then I get 2 Jira tickets
    And ticket 1 is "PROJ-1" "First" at "<url1>"
    And ticket 2 is "PROJ-2" "Second" at "<url2>"

    Examples:
      | base                        | url1                                     | url2                                     |
      | https://firma.atlassian.net | https://firma.atlassian.net/browse/PROJ-1 | https://firma.atlassian.net/browse/PROJ-2 |
      | https://jira.firma.de/      | https://jira.firma.de/browse/PROJ-1      | https://jira.firma.de/browse/PROJ-2      |

  @domain @US-J01
  Scenario: A ticket without a summary field gets an empty summary
    When I parse the Jira response for "https://x.net":
      """
      {"issues":[{"key":"PROJ-1","fields":{}},{"key":"PROJ-2"}]}
      """
    Then I get 2 Jira tickets
    And ticket 1 is "PROJ-1" "" at "https://x.net/browse/PROJ-1"
    And ticket 2 is "PROJ-2" "" at "https://x.net/browse/PROJ-2"

  @domain @US-J01
  Scenario: An empty result is not an error
    When I parse the Jira response for "https://x.net":
      """
      {"issues":[]}
      """
    Then I get 0 Jira tickets

  @domain @US-J01
  Scenario: At most 50 hits are returned
    When I parse a Jira response with 60 tickets
    Then I get 50 Jira tickets
    And ticket 50 is "PROJ-50" "Ticket 50" at "https://x.net/browse/PROJ-50"

  @domain @US-J01
  Scenario Outline: A malformed response is rejected
    When I parse the Jira response for "https://x.net":
      """
      <body>
      """
    Then the Jira response is rejected

    Examples:
      | body                    |
      | not json                |
      | {"issues":"nope"}       |
      | {"total":1}             |
      | {"issues":[{"id":"1"}]} |

  @domain @US-J03
  Scenario: An open todo with the ticket link counts as imported
    Given the Jira todos:
      | rawText                                                        | completed |
      | [PROJ-123](https://firma.atlassian.net/browse/PROJ-123) Login  | false     |
    Then the ticket "PROJ-123" is imported

  @domain @US-J03
  Scenario: A ticket that appears at the end of the text counts as imported
    Given the Jira todos:
      | rawText                                | completed |
      | Siehe https://x.net/browse/PROJ-123    | false     |
    Then the ticket "PROJ-123" is imported

  @domain @US-J03
  Scenario: A completed todo does not count as imported
    Given the Jira todos:
      | rawText                                                                 | completed |
      | x 2026-11-01 [PROJ-123](https://firma.atlassian.net/browse/PROJ-123) Login | true   |
    Then the ticket "PROJ-123" is not imported

  @domain @US-J03
  Scenario Outline: A ticket whose key only starts like another key is not imported
    Given the Jira todos:
      | rawText                                    | completed |
      | [<other>](https://x.net/browse/<other>) A  | false     |
    Then the ticket "PROJ-12" is not imported

    Examples:
      | other     |
      | PROJ-123  |
      | PROJ-12a  |
      | PROJ-12_1 |

  @domain @US-J03
  Scenario: Another occurrence of the key later in the text still counts
    Given the Jira todos:
      | rawText                                                           | completed |
      | https://x.net/browse/PROJ-123 und https://x.net/browse/PROJ-12 x  | false     |
    Then the ticket "PROJ-12" is imported

  @domain @US-J03
  Scenario: A mention without a browse link is not an import
    Given the Jira todos:
      | rawText              | completed |
      | Fix PROJ-123 sofort  | false     |
    Then the ticket "PROJ-123" is not imported

  @domain @US-J01
  Scenario Outline: The API path depends on the Jira kind
    Then the "<endpoint>" path for "<kind>" is "<path>"

    Examples:
      | kind        | endpoint | path                  |
      | cloud       | search   | /rest/api/3/search/jql |
      | cloud       | myself   | /rest/api/3/myself    |
      | data_center | search   | /rest/api/2/search    |
      | data_center | myself   | /rest/api/2/myself    |

  @domain @US-J05
  Scenario Outline: A configuration is complete when it can be used
    Then the Jira configuration of kind "<kind>" with URL "<url>" and e-mail "<email>" is <state>

    Examples:
      | kind        | url                 | email    | state      |
      | cloud       | https://x.net       | me@x.net | complete   |
      | cloud       | https://x.net       |          | incomplete |
      | cloud       |                     | me@x.net | incomplete |
      | data_center | https://jira.x.net  |          | complete   |
      | data_center |                     |          | incomplete |
