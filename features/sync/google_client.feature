Feature: Own Google OAuth client
  Source: ADR 0019; nunc ships no Google client, every user registers their own; desktop only

  @ui @US-S08 @desktop @wip
  Scenario: Without an own client the Drive section asks for one
    Given the setting "syncProvider" is "google_drive"
    And no Google client is configured
    When I open the app at "/settings"
    Then I see the fields "Client-ID, Client-Secret"
    And the field "Client-Secret" is masked
    And the button "Mit Google Drive verbinden" is disabled
    And I see "nunc bringt keinen eigenen Google-Zugang mit."

  @ui @US-S08 @desktop @wip
  Scenario: The setup guide links to the Google Cloud Console
    Given the setting "syncProvider" is "google_drive"
    And no Google client is configured
    When I open the app at "/settings"
    And I click "So richtest du einen Google-Zugang ein"
    Then I see "Veröffentlichungsstatus auf „In Produktion“ setzen"
    And I see "Typ „Desktop-App“"
    When I click "Google Cloud Console öffnen"
    Then the browser opened "https://console.cloud.google.com/apis/credentials"

  @ui @US-S08 @desktop @wip
  Scenario: Saving the client enables connecting
    Given the setting "syncProvider" is "google_drive"
    And no Google client is configured
    When I open the app at "/settings"
    And I enter "123-abc.apps.googleusercontent.com" into the field "Client-ID"
    And I enter "GOCSPX-secret" into the field "Client-Secret"
    And I click "Zugang speichern"
    Then the backend received "set_google_client"
    And the button "Mit Google Drive verbinden" is enabled
    And the field "Client-Secret" is empty

  @ui @US-S08 @desktop @wip
  Scenario: A client ID that is not a Google client ID is rejected
    Given the setting "syncProvider" is "google_drive"
    And no Google client is configured
    When I open the app at "/settings"
    And I enter "not-a-client" into the field "Client-ID"
    And I enter "GOCSPX-secret" into the field "Client-Secret"
    And I click "Zugang speichern"
    Then I see "Die Client-ID muss auf .apps.googleusercontent.com enden."
    And the button "Mit Google Drive verbinden" is disabled

  @integration @sync @US-S08 @desktop @wip
  Scenario: Connecting uses the client from the secret store
    Given a Google server for the account "me@example.org"
    And the Google client "123-abc.apps.googleusercontent.com" with the secret "GOCSPX-secret" is configured
    When I connect Google Drive
    Then the authorization request carries the client ID "123-abc.apps.googleusercontent.com"
    And the token request carries the client secret "GOCSPX-secret"
    And the Drive status shows the account "me@example.org"

  @integration @sync @US-S08 @desktop @wip
  Scenario: The client is stored only in the secret store
    Given the Google client "123-abc.apps.googleusercontent.com" with the secret "GOCSPX-secret" is configured
    Then the secret store holds a Google client
    And the database does not contain "GOCSPX-secret"
    And the database does not contain "123-abc.apps.googleusercontent.com"

  @integration @sync @US-S08 @desktop @wip
  Scenario: Without a client connecting is refused and the browser stays closed
    Given a Google server for the account "me@example.org"
    And no Google client is configured
    When I connect Google Drive
    Then connecting failed with "Kein Google-Zugang eingerichtet. Trage Client-ID und Client-Secret in den Einstellungen ein."
    And the browser was not opened

  @integration @sync @US-S08 @desktop @wip
  Scenario: Changing the client ID ends the connection
    Given a Google server for the account "me@example.org"
    And the Google client "123-abc.apps.googleusercontent.com" with the secret "GOCSPX-secret" is configured
    And Google Drive is connected as "me@example.org"
    When I configure the Google client "456-def.apps.googleusercontent.com" with the secret "GOCSPX-other"
    Then the secret store holds no Google Drive connection
    And the Drive status asks to connect again

  @integration @sync @US-S08 @desktop @wip
  Scenario: Saving the same client ID with a new secret keeps the connection
    Given a Google server for the account "me@example.org"
    And the Google client "123-abc.apps.googleusercontent.com" with the secret "GOCSPX-secret" is configured
    And Google Drive is connected as "me@example.org"
    When I configure the Google client "123-abc.apps.googleusercontent.com" with the secret "GOCSPX-rotated"
    Then the secret store holds a Google Drive connection
    And the next token renewal carries the client secret "GOCSPX-rotated"

  @integration @sync @US-S08 @desktop @wip
  Scenario: Removing the client also ends the connection
    Given a Google server for the account "me@example.org"
    And the Google client "123-abc.apps.googleusercontent.com" with the secret "GOCSPX-secret" is configured
    And Google Drive is connected as "me@example.org"
    When I remove the Google client
    Then the secret store holds no Google client
    And the secret store holds no Google Drive connection

  @integration @sync @US-S08 @desktop @wip
  Scenario: A connection made with the former built-in client asks for an own client
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no Google client is configured
    And no database exists yet
    And the sync provider is "google_drive"
    When I sync
    Then the sync fails with "Google Drive braucht jetzt einen eigenen Zugang – bitte in den Einstellungen einrichten"
    And Google was not contacted
