Feature: Google Drive sync
  Source: US-Sync-GoogleDrive (S01-S07); desktop only in v1

  @ui @US-S01 @desktop @wip
  Scenario: The sync section shows not connected

  @integration @sync @US-S01 @desktop
  Scenario: Connecting uses OAuth2 with PKCE in the browser
    Given a Google server for the account "me@example.org"
    When I connect Google Drive
    Then the browser was opened once with the Google authorization page
    And the authorization request uses the code challenge method "S256"
    And the token request carries the code verifier of the challenge
    And the Drive status shows the account "me@example.org"

  @integration @sync @US-S01 @desktop
  Scenario: Only the drive.file scope is requested
    Given a Google server for the account "me@example.org"
    When I connect Google Drive
    Then the authorization request asks for the scope "https://www.googleapis.com/auth/drive.file" only

  @integration @sync @US-S01 @desktop
  Scenario: The token is stored in the secret store
    Given a Google server for the account "me@example.org"
    When I connect Google Drive
    Then the secret store holds a Google Drive connection
    And the database does not contain "access-1"
    And the database does not contain "refresh-1"

  @ui @US-S01 @desktop @wip
  Scenario: The status shows the connected account

  @integration @sync @US-S01 @desktop
  Scenario: Disconnecting deletes the token locally
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    When I disconnect Google Drive
    Then the secret store holds no Google Drive connection
    And the sync file on Drive still exists

  @integration @sync @US-S02 @desktop
  Scenario: A sync runs at app start
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the local todos "Task"
    And the sync provider is "google_drive"
    And the coordinator runs the Drive runner
    When the app starts
    Then within 3000 ms the sync file appears on Drive
    And the sync file on Drive has the todos "Task"

  @integration @sync @US-S02 @desktop
  Scenario: A sync runs at quit and delays closing by at most five seconds
    Given a sync coordinator whose runs take 3000 ms
    And the quit limit is 200 ms
    When the app quits
    Then closing was delayed by less than 1500 ms

  @integration @sync @US-S02 @desktop
  Scenario: A failed sync does not prevent the app from starting
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the local todos "Offline task"
    And the sync provider is "google_drive"
    And Google answers every request with 503
    And the coordinator runs the Drive runner
    When the app starts
    And I wait 300 ms
    Then the sync status is "error - Unerwartete Antwort des Servers (503) beim Suchen des Ordners"
    And the local todos are "Offline task"

  @ui @US-S02 @desktop @wip
  Scenario: The sync status is shown while syncing

  @ui @US-S03 @desktop @wip
  Scenario: The sync now button is disabled while syncing

  @ui @US-S03 @desktop @wip
  Scenario: The result shows the number of imported sessions and todos

  @ui @US-S03 @desktop @wip
  Scenario: Errors are shown in plain text

  @integration @sync @US-S04 @desktop
  Scenario: Newer todo versions from another device win
    Given the Go fixture database
    And a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    When I sync with Google Drive
    Then the sync succeeds with 1 imported todos, 2 imported sessions and 2 conflicts
    And the local todo "t-0002" says "(A) 2026-09-21 Call dentist tomorrow @phone"
    And the local todo "t-0003" says "2026-09-22 Buy groceries +home @errands"

  @ui @US-S04 @desktop @wip
  Scenario: The user is informed about taken-over newer versions

  @integration @sync @US-S04 @desktop
  Scenario: Sessions of both devices are merged
    Given the Go fixture database
    And a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    When I sync with Google Drive
    Then the sync file on Drive has 7 todos and 11 sessions
    And the sync file on Drive has 11 sessions in the local database

  @integration @sync @US-S04 @desktop
  Scenario: Deleted todos are propagated
    Given the Go fixture database
    And a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    When I sync with Google Drive
    Then the local todo "t-0005" is deleted

  @integration @sync @US-S05 @desktop
  Scenario: The sync file is stored as nunc/nunc-sync.json
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the local todos "Local task, Another task"
    When I sync with Google Drive
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And the sync file on Drive has the todos "Local task, Another task"
    And the Drive folder is remembered in the settings

  @integration @sync @US-S05 @desktop
  Scenario: The folder is created when missing
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    When I sync with Google Drive
    Then Drive holds 1 folder named "nunc"
    When I sync with Google Drive
    Then Drive holds 1 folder named "nunc"
    And Drive holds 1 sync file

  @integration @sync @US-S05 @desktop
  Scenario: A deleted remote file leads to uploading the local state
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" exists without a sync file
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the local todos "Only local"
    When I sync with Google Drive
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And the sync file on Drive has the todos "Only local"
    And the local todos are "Only local"

  @ui @US-S05 @desktop @wip
  Scenario: The drive path and a link to the folder are shown

  @integration @sync @US-S06 @desktop
  Scenario: An existing sync file is detected on a device without data
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    When I look for a sync file to import
    Then the import offer shows 3 todos and 3 sessions

  @ui @US-S06 @desktop @wip
  Scenario: A dialog offers importing or starting empty

  @integration @sync @US-S06 @desktop
  Scenario: Importing takes over all sessions and todos
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    When I sync with Google Drive
    Then the sync succeeds with 3 imported todos, 3 imported sessions and 0 conflicts
    And the local database has 3 sessions

  @ui @US-S06 @desktop @wip
  Scenario: A progress bar and a summary are shown while importing

  @integration @sync @US-S06 @desktop
  Scenario: After the import the app works offline
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    When I sync with Google Drive
    And Google answers every request with 503
    Then the local database has 3 sessions
    And the local todos include "2026-09-29 Water the plants @home"

  @ui @US-S07 @desktop @wip
  Scenario: Sync can be paused with a toggle

  @integration @sync @US-S07 @desktop
  Scenario: A paused sync makes no Drive access and keeps the token
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the sync provider is "google_drive"
    And the sync is paused
    When the Drive runner runs
    Then the runner did nothing
    And Google was not contacted
    And the secret store holds a Google Drive connection

  @integration @sync @US-S01 @desktop
  Scenario: A denied authorization is reported
    Given a Google server for the account "me@example.org"
    And the user denies the authorization
    When I connect Google Drive
    Then connecting failed with "Die Anmeldung bei Google Drive wurde abgebrochen: access_denied"
    And the secret store holds no Google Drive connection

  @integration @sync @US-S01 @desktop
  Scenario: A failing token exchange is reported
    Given a Google server for the account "me@example.org"
    And Google rejects the authorization code with "invalid_grant"
    When I connect Google Drive
    Then connecting failed with "Token-Austausch mit Google Drive fehlgeschlagen: invalid_grant"
    And the secret store holds no Google Drive connection

  @integration @sync @US-S01 @desktop
  Scenario: Connecting works without the account e-mail
    Given a Google server that does not tell the account
    When I connect Google Drive
    Then the Drive status shows no account
    And the secret store holds a Google Drive connection

  @integration @sync @US-S01 @desktop
  Scenario: Connecting gives up when the browser never answers
    Given a Google server for the account "me@example.org"
    And the user ignores the browser
    And the authorization times out after 1 second
    When I connect Google Drive
    Then connecting failed with "Zeitüberschreitung beim Verbinden mit Google Drive (nach 1 s)"
    And the secret store holds no Google Drive connection

  @integration @sync @US-S01 @desktop
  Scenario: A build without Google credentials cannot connect
    Given a Google server for the account "me@example.org"
    And the app has no Google credentials
    When I connect Google Drive
    Then connecting failed with "In diesem Build sind keine Google-Zugangsdaten hinterlegt (NUNC_GOOGLE_CLIENT_ID, NUNC_GOOGLE_CLIENT_SECRET)."
    And the browser was not opened

  @integration @sync @US-S01 @desktop
  Scenario: An expired access token is renewed before the request
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And the access token has expired
    And no database exists yet
    When I sync with Google Drive
    Then the sync succeeds
    And Google issued 1 tokens by renewal
    And the stored refresh token is "refresh-1"

  @integration @sync @US-S01 @desktop
  Scenario: A token that Google rejects is renewed and the request repeated
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And Google no longer accepts the access token
    And no database exists yet
    When I sync with Google Drive
    Then the sync succeeds
    And Google issued 1 tokens by renewal

  @integration @sync @US-S03 @desktop
  Scenario: A connection that cannot be renewed asks for a new login
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And Google no longer accepts the access token
    And Google refuses to renew tokens
    And no database exists yet
    When I sync with Google Drive
    Then the sync fails with "Google Drive: Anmeldung abgelaufen – bitte neu verbinden"

  @integration @sync @US-S03 @desktop
  Scenario: A sync without a connection says so
    Given a Google server for the account "me@example.org"
    And no database exists yet
    When I sync with Google Drive
    Then the sync fails with "Nicht mit Google Drive verbunden"
    And Google was not contacted

  @integration @sync @US-S05 @desktop
  Scenario: A broken sync file is reported and left alone
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds a broken sync file
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the local todos "Task"
    When I sync with Google Drive
    Then the sync fails with a message starting "Sync-Datei nicht lesbar"
    And the Drive sync file is unchanged

  @integration @sync @US-S04 @desktop
  Scenario: A change by another device during the upload is merged again
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And another device adds the todo "Von Gerät C" to the Drive file during the first upload
    When I sync with Google Drive
    Then the sync succeeds
    And the local todos include "Von Gerät C"
    And the sync file on Drive has 5 todos and 3 sessions

  @integration @sync @US-S02 @desktop
  Scenario: A successful run records the time and the folder
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the sync provider is "google_drive"
    When the Drive runner runs
    Then the runner reports the finish time "2026-10-04T09:00:00Z"
    And the last Drive sync time in the settings is "2026-10-04T09:00:00Z"
    And the Drive folder is remembered in the settings

  @integration @sync @US-S02 @desktop
  Scenario: Another provider is not served by the Drive runner
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the sync provider is "webdav"
    When the Drive runner runs
    Then the runner did nothing
    And Google was not contacted

  @integration @sync @US-S06 @desktop
  Scenario: No import is offered when this device has data
    Given a Google server for the account "me@example.org"
    And the Drive folder "nunc" holds the sync file "webdav/nunc-sync.json"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    And the local todos "Mine"
    When I look for a sync file to import
    Then no import is offered
    And Google was not contacted

  @integration @sync @US-S06 @desktop
  Scenario: No import is offered when Drive holds no sync file
    Given a Google server for the account "me@example.org"
    And Google Drive is connected as "me@example.org"
    And no database exists yet
    When I look for a sync file to import
    Then no import is offered
