Feature: Todoist connection
  Source: US-Todoist (US-Todoist-Auth); desktop only in v1

  @ui @US-Todoist-Auth @desktop @wip
  Scenario: The Todoist section shows the connection status

  @integration @US-Todoist-Auth @desktop @wip
  Scenario: Connecting opens the browser with the Todoist authorization page

  @integration @US-Todoist-Auth @desktop @wip
  Scenario: The callback exchanges the code for access and refresh tokens

  @integration @US-Todoist-Auth @desktop @wip
  Scenario: The tokens are stored in the secret store

  @integration @US-Todoist-Auth @desktop @wip
  Scenario: A first sync runs after connecting

  @ui @US-Todoist-Auth @desktop @wip
  Scenario: The connected account is shown with e-mail and avatar

  @ui @US-Todoist-Auth @desktop @wip
  Scenario: Disconnecting asks for confirmation

  @integration @US-Todoist-Auth @desktop @wip
  Scenario: Disconnecting deletes the tokens and keeps local todos

  @ui @US-Todoist-Auth @desktop @wip
  Scenario: A token exchange error is shown inline and can be retried

  @integration @US-Todoist-Auth @desktop @wip
  Scenario: An expired access token is refreshed automatically
