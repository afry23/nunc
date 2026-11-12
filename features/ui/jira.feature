Feature: Jira import
  Source: ADR 0038
  A ticket from Jira becomes a task with a link to the original.

  @ui @US-J05
  Scenario: The settings offer type, URL, e-mail and token
    When I open the app at "/settings"
    Then I see the heading "Jira"
    And I see the radio group "Jira-Art" with the options "Cloud, Data Center"
    And I see the fields "Jira-URL, Jira-E-Mail, Jira-Token"
    And I see the buttons "Jira-Verbindung testen"
    And the option "Cloud" is selected

  @ui @US-J05
  Scenario: Data Center hides the e-mail field
    When I open the app at "/settings"
    And I choose the option "Data Center"
    Then the field "Jira-E-Mail" does not exist
    And the field "Jira-URL" exists

  @ui @US-J05
  Scenario: Saved settings are shown and the stored token stays hidden
    Given the Jira settings are:
      | baseUrl                     | kind  | email        |
      | https://firma.atlassian.net | cloud | me@firma.de  |
    And a Jira token is stored
    When I open the app at "/settings"
    Then the field "Jira-URL" has the value "https://firma.atlassian.net"
    And the field "Jira-E-Mail" has the value "me@firma.de"
    And the field "Jira-Token" has the value ""
    And the field "Jira-Token" is masked
    And I see the buttons "Jira-Zugang entfernen"

  @ui @US-J05
  Scenario: Without a stored access there is nothing to remove
    When I open the app at "/settings"
    Then I do not see the button "Jira-Zugang entfernen"

  @ui @US-J05
  Scenario: An address without http is refused
    When I open the app at "/settings"
    And I enter "firma.atlassian.net" into the field "Jira-URL"
    Then I see "Die Adresse muss mit http:// oder https:// beginnen."
    And the button "Jira-Einstellungen speichern" is disabled
    And the button "Jira-Verbindung testen" is disabled

  @ui @US-J05
  Scenario: The settings are saved and the token field is cleared
    When I open the app at "/settings"
    And I enter "https://firma.atlassian.net" into the field "Jira-URL"
    And I enter "me@firma.de" into the field "Jira-E-Mail"
    And I enter "secret-1" into the field "Jira-Token"
    And I click "Jira-Einstellungen speichern"
    Then the backend received "save_jira_config"
    And I see "Jira-Zugang gespeichert."
    And the field "Jira-Token" has the value ""

  @ui @US-J05
  Scenario: The connection test reports success
    Given the Jira connection test succeeds
    When I open the app at "/settings"
    And I enter "https://firma.atlassian.net" into the field "Jira-URL"
    And I click "Jira-Verbindung testen"
    Then I see "Verbindung erfolgreich."

  @ui @US-J05
  Scenario: The connection test reports the failure
    Given the Jira connection test fails with "Jira lehnt die Zugangsdaten ab (401/403)"
    When I open the app at "/settings"
    And I enter "https://firma.atlassian.net" into the field "Jira-URL"
    And I click "Jira-Verbindung testen"
    Then I see "Jira lehnt die Zugangsdaten ab (401/403)"

  @ui @US-J05
  Scenario: The configuration can be removed
    Given the Jira settings are:
      | baseUrl                     | kind  | email        |
      | https://firma.atlassian.net | cloud | me@firma.de  |
    And a Jira token is stored
    When I open the app at "/settings"
    And I click "Jira-Zugang entfernen"
    Then the backend received "clear_jira_config"
    And I see "Jira-Zugang entfernt."
    And the field "Jira-URL" has the value ""
    And I do not see the button "Jira-Zugang entfernen"

  @ui @wip @US-J06
  Scenario: The button "Aus Jira" is hidden without a configuration

  @ui @wip @US-J06
  Scenario: The picker first shows my open tickets

  @ui @wip @US-J06
  Scenario: Searching by text or key filters the tickets

  @ui @wip @US-J06
  Scenario: Choosing a ticket creates a task with a link and shows a toast

  @ui @wip @US-J06
  Scenario: An already imported ticket is marked but can still be chosen

  @ui @wip @US-J06
  Scenario: Rejected credentials are shown as an error message

  @ui @wip @US-J06
  Scenario: Without a configuration the picker links to the settings
