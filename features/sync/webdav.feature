Feature: WebDAV sync
  Source: US-WebDAV (US-Sync-01-WebDAV-Config, US-Sync-02-WebDAV); nunc-go sync_webdav*.go

  @ui @US-Sync-01-WebDAV-Config
  Scenario: The configuration offers URL, authentication method, user, password and remote path
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    Then I see the fields "Server-URL, Authentifizierung, Benutzername, Passwort, Remote-Pfad"
    And the field "Remote-Pfad" has the value "/nunc/"
    And I see the buttons "Verbindung testen"

  @ui @US-Sync-01-WebDAV-Config
  Scenario: Saved settings are shown and the stored password stays hidden
    Given the setting "syncProvider" is "webdav"
    And the WebDAV settings are:
      | url                     | authMethod | username | remotePath  |
      | https://dav.example.org | digest     | me       | /apps/nunc/ |
    And a WebDAV password is stored
    When I open the app at "/settings"
    Then the field "Server-URL" has the value "https://dav.example.org"
    And the field "Benutzername" has the value "me"
    And the field "Remote-Pfad" has the value "/apps/nunc/"
    And the field "Passwort" has the value ""
    And I do not see "WebDAV ist ausgewählt, aber noch nicht eingerichtet"
  @ui @US-Sync-01-WebDAV-Config
  Scenario: The password is masked and can be revealed
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    And I enter "geheim" into the field "Passwort"
    Then the field "Passwort" is masked
    When I click "Passwort anzeigen"
    Then the field "Passwort" is readable
    When I click "Passwort verbergen"
    Then the field "Passwort" is masked
  @ui @US-Sync-01-WebDAV-Config
  Scenario: An invalid URL disables saving and testing
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    And I enter "cloud.example.org" into the field "Server-URL"
    Then I see "Die Adresse muss mit http:// oder https:// beginnen."
    And the button "Verbindung testen" is disabled
    And the button "WebDAV-Einstellungen speichern" is disabled
    When I enter "https://cloud.example.org" into the field "Server-URL"
    Then the button "Verbindung testen" is enabled
    And the button "WebDAV-Einstellungen speichern" is enabled
  @ui @US-Sync-01-WebDAV-Config
  Scenario: Changes are only applied on explicit save
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    And I enter "https://dav.example.org" into the field "Server-URL"
    And I enter "me" into the field "Benutzername"
    And I enter "pw" into the field "Passwort"
    Then the backend did not receive "save_webdav_config"
    When I click "WebDAV-Einstellungen speichern"
    Then the backend received "save_webdav_config" with:
      """
      { "config": { "url": "https://dav.example.org", "username": "me", "authMethod": "basic", "remotePath": "/nunc/" }, "password": "pw" }
      """
    And I see "WebDAV-Einstellungen gespeichert."
    And the field "Passwort" has the value ""

  @ui @US-Sync-01-WebDAV-Config
  Scenario: Saving without a new password keeps the stored one
    Given the setting "syncProvider" is "webdav"
    And the WebDAV settings are:
      | url                     | authMethod | username | remotePath |
      | https://dav.example.org | basic      | me       | /nunc/     |
    And a WebDAV password is stored
    When I open the app at "/settings"
    And I enter "https://other.example.org" into the field "Server-URL"
    And I click "WebDAV-Einstellungen speichern"
    Then the backend received "save_webdav_config" with:
      """
      { "config": { "url": "https://other.example.org" }, "password": null }
      """
  @ui @US-Sync-01-WebDAV-Config
  Scenario: A banner shows that WebDAV is selected but not configured
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    Then I see "WebDAV ist ausgewählt, aber noch nicht eingerichtet."

  @ui @US-Sync-01-WebDAV-Config
  Scenario: No banner is shown while no sync is selected
    When I open the app at "/settings"
    Then I do not see "noch nicht eingerichtet"
  @ui @US-Sync-01-WebDAV-Config
  Scenario: Testing the connection shows success with the server name
    Given the setting "syncProvider" is "webdav"
    And the connection test succeeds for "dav.example.org"
    When I open the app at "/settings"
    And I enter "https://dav.example.org" into the field "Server-URL"
    And I enter "pw" into the field "Passwort"
    And I click "Verbindung testen"
    Then the backend received "test_webdav_connection" with:
      """
      { "config": { "url": "https://dav.example.org" }, "password": "pw" }
      """
    And I see "Verbindung erfolgreich: dav.example.org"
  @ui @US-Sync-01-WebDAV-Config
  Scenario: Testing the connection shows a readable error
    Given the setting "syncProvider" is "webdav"
    And the connection test fails with "401 Unauthorized – Benutzername oder Passwort falsch"
    When I open the app at "/settings"
    And I enter "https://dav.example.org" into the field "Server-URL"
    And I click "Verbindung testen"
    Then I see "401 Unauthorized – Benutzername oder Passwort falsch"
    And I do not see "Verbindung erfolgreich"
  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: The connection test sends a PROPFIND request
    Given a WebDAV server with the folder "/nunc/"
    When I test the connection as "me" with password "secret"
    Then the server received a "PROPFIND" request for "/nunc/" with depth "0"
    And the connection test succeeds with the message "127.0.0.1"

  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: A missing folder does not fail the connection test
    Given a WebDAV server without folders
    When I test the connection as "me" with password "secret"
    Then the connection test succeeds with the message "127.0.0.1"

  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario Outline: The connection test explains what went wrong
    Given a WebDAV server that answers every request with <status>
    When I test the connection as "me" with password "secret"
    Then the connection test fails with the message "<message>"

    Examples:
      | status | message                                                       |
      | 401    | 401 Unauthorized – Benutzername oder Passwort falsch          |
      | 403    | 403 Forbidden – Zugriff verweigert                            |
      | 500    | Unerwartete Antwort des Servers (500) beim Prüfen des Ordners |

  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: A server that is not running is reported
    Given a WebDAV server that is not running
    When I test the connection as "me" with password "secret"
    Then the connection test fails with the message "Verbindung abgelehnt – Server nicht erreichbar"

  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: An address without a host is refused before any request
    When I test the connection to "cloud.example.org" as "me" with password "secret"
    Then the connection test fails with the message "Ungültige Server-Adresse: cloud.example.org"
  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: The password is stored in the secret store and not in the database
    Given no database exists yet
    When I save the WebDAV settings with the password "s3cret-pass":
      | url        | https://dav.example.org/remote.php/dav |
      | authMethod | digest                                 |
      | username   | me                                     |
      | remotePath | /apps/nunc/                            |
    Then the secret store holds the password "s3cret-pass"
    And the database does not contain "s3cret-pass"

  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: Saving without a password keeps the stored password
    Given no database exists yet
    When I save the WebDAV settings with the password "first":
      | url        | https://dav.example.org |
      | authMethod | basic                   |
      | username   | me                      |
      | remotePath | /nunc/                  |
    And I save the WebDAV settings without a password:
      | url        | https://other.example.org |
      | authMethod | basic                     |
      | username   | me                        |
      | remotePath | /nunc/                    |
    Then the secret store holds the password "first"
    And the loaded WebDAV settings have the url "https://other.example.org"
  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: The other settings are stored in the database
    Given no database exists yet
    When I save the WebDAV settings with the password "pw":
      | url        | https://dav.example.org/remote.php/dav |
      | authMethod | digest                                 |
      | username   | me                                     |
      | remotePath | /apps/nunc/                            |
    Then the settings table contains:
      | key                      | value                                  |
      | sync.webdav.url          | https://dav.example.org/remote.php/dav |
      | sync.webdav.auth_method  | digest                                 |
      | sync.webdav.username     | me                                     |
      | sync.webdav.remote_path  | /apps/nunc/                            |

  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario Outline: A sync needs a complete address and a user
    Given no database exists yet
    When I save the WebDAV settings with the password "pw":
      | url        | <url>  |
      | authMethod | basic  |
      | username   | <user> |
      | remotePath | /nunc/ |
    Then the settings are usable for a sync: <usable>

    Examples:
      | url                     | user | usable |
      | https://dav.example.org | me   | yes    |
      |                         | me   | no     |
      | https://dav.example.org |      | no     |
      | dav.example.org         | me   | no     |

  @integration @sync @US-Sync-01-WebDAV-Config
  Scenario: A sync needs a stored password
    Given no database exists yet
    When I save the WebDAV settings without a password:
      | url        | https://dav.example.org |
      | authMethod | basic                   |
      | username   | me                      |
      | remotePath | /nunc/                  |
    Then the settings are usable for a sync: no

  @integration @sync @US-Sync-02-WebDAV
  Scenario: Basic authentication is used
    Given a WebDAV server with the folder "/nunc/" that requires basic authentication for "me" with password "secret"
    When I sync as "me" with password "secret" using "basic"
    Then the sync succeeds
    And every request to the server was authenticated

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A wrong password is reported
    Given a WebDAV server with the folder "/nunc/" that requires basic authentication for "me" with password "secret"
    When I sync as "me" with password "wrong" using "basic"
    Then the sync fails with "401 Unauthorized – Benutzername oder Passwort falsch"
  @integration @sync @US-Sync-02-WebDAV
  Scenario: Digest authentication is used
    Given a WebDAV server with the folder "/nunc/" that requires digest authentication for "me" with password "secret"
    When I sync as "me" with password "secret" using "digest"
    Then the sync succeeds
    And the server accepted a digest authorization
    And the server did not receive a basic authorization

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A wrong digest password is reported
    Given a WebDAV server with the folder "/nunc/" that requires digest authentication for "me" with password "secret"
    When I sync as "me" with password "wrong" using "digest"
    Then the sync fails with "401 Unauthorized – Benutzername oder Passwort falsch"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A server that wants digest is reported when basic is configured
    Given a WebDAV server with the folder "/nunc/" that requires digest authentication for "me" with password "secret"
    When I sync as "me" with password "secret" using "basic"
    Then the sync fails with "Der Server verlangt Digest-Authentifizierung, eingestellt ist Basic"
    And the server received 1 requests
  @integration @sync @US-Sync-02-WebDAV
  Scenario: The remote folder is created with MKCOL when missing
    Given a WebDAV server without folders
    When I sync with the remote path "/files/nunc/"
    Then the sync succeeds
    And the server created the folders "/files/ /files/nunc/" in this order
    And the server holds the sync file "/files/nunc/nunc-sync.v2.json"
  @integration @sync @US-Sync-02-WebDAV
  Scenario: A folder that another device creates at the same moment is accepted
    Given a WebDAV server with the folder "/nunc/"
    And the server hides the existing folder from lookups
    And no database exists yet
    And the local todos "Task"
    When I sync
    Then the sync succeeds
    And the sync file on the server has the todos "Task"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: The sync file is created on the first sync
    Given a WebDAV server with the folder "/nunc/"
    And no database exists yet
    And the local todos "Local task, Another task"
    When I sync
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And the sync file on the server has the todos "Local task, Another task"
    And the upload required that the file does not exist yet
  @integration @sync @US-Sync-02-WebDAV
  Scenario: A missing remote file is treated as an empty remote state
    Given a WebDAV server with the folder "/nunc/"
    And no database exists yet
    And the local todos "Only local"
    When I sync
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And the local todos are "Only local"
  @integration @sync @US-Sync-02-WebDAV
  Scenario: Sync downloads, merges and uploads the sync file
    Given the Go fixture database
    And a WebDAV server with the folder "/nunc/" holding the sync file "webdav/nunc-sync.json"
    When I sync
    Then the sync succeeds with 1 imported todos, 2 imported sessions and 2 conflicts
    And the local todo "t-0002" says "(A) 2026-09-21 Call dentist tomorrow @phone"
    And the local todo "t-0003" says "2026-09-22 Buy groceries +home @errands"
    And the local todo "t-0005" is deleted
    And the local todo "t-0100" says "2026-09-29 Water the plants @home"
    And the sync file on the server has 7 todos and 11 sessions
    And the upload carried the ETag of the downloaded file

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A second sync changes nothing
    Given the Go fixture database
    And a WebDAV server with the folder "/nunc/" holding the sync file "webdav/nunc-sync.json"
    When I sync
    And I sync
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And the sync file on the server has 7 todos and 11 sessions

  @integration @sync @US-Sync-02-WebDAV
  Scenario: Sessions of this device are not exported again as foreign ones
    Given the Go fixture database
    And a WebDAV server with the folder "/nunc/" holding the sync file "nunc-sync.json"
    When I sync
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A server without ETag support is written without a condition
    Given a WebDAV server with the folder "/nunc/" holding the sync file "webdav/nunc-sync.json"
    And the server does not support ETags
    And the Go fixture database
    When I sync
    Then the sync succeeds
    And no upload carried a condition

  @integration @sync @US-Sync-02-WebDAV
  Scenario: An unreadable sync file is reported and left alone
    Given a WebDAV server with the folder "/nunc/" holding a broken sync file
    And no database exists yet
    And the local todos "Task"
    When I sync
    Then the sync fails with a message starting "Sync-Datei nicht lesbar"
    And the sync file on the server is unchanged
    And the local todos are "Task"
  @integration @sync @REQ-SYNC
  Scenario: A legacy sync file is imported once
    Given a WebDAV server with the folder "/nunc/" holding the legacy sync file "webdav/nunc-sync.json"
    And no database exists yet
    When I sync
    Then the sync succeeds with 3 imported todos, 3 imported sessions and 0 conflicts
    And the server holds the sync file "/nunc/nunc-sync.v2.json"
    And the upload required that the file does not exist yet
    When I sync
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And the legacy file was requested 1 time

  @integration @sync @REQ-SYNC
  Scenario: A sync file of an unknown format is reported and left alone
    Given a WebDAV server with the folder "/nunc/" holding a sync file of format 6
    And no database exists yet
    And the local todos "Task"
    When I sync
    Then the sync fails with a message starting "Sync-Datei nicht lesbar: Die Sync-Datei hat das Format 6"
    And the sync file on the server is unchanged
    And the local todos are "Task"

  @integration @sync @REQ-SYNC
  Scenario Outline: A tombstone is removed from the sync file after 90 days
    Given a WebDAV server with the folder "/nunc/" holding the todos "x-1" deleted on "<day>"
    And no database exists yet
    And the local todos "Task"
    When I sync
    Then the sync succeeds
    And the sync file on the server has <count> todos and 0 sessions

    Examples:
      | day        | count |
      | 2026-09-20 | 2     |
      | 2026-05-01 | 1     |

  @integration @sync @REQ-SYNC
  Scenario: Changes to different fields on two devices are both kept
    Given a WebDAV server with the folder "/nunc/"
    And no database exists yet
    And the local todos "Buy milk"
    And another device has completed the todo "Buy milk" on the server
    When I rename the local todo "Buy milk" to "Buy oat milk"
    And I sync
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 1 conflicts
    And the local todos are "x 2026-10-04 Buy oat milk"
    And the sync file on the server has the todos "x 2026-10-04 Buy oat milk"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A concurrent change on the server is detected with the ETag
    Given a WebDAV server with the folder "/nunc/" holding the sync file "webdav/nunc-sync.json"
    And the Go fixture database
    And another device adds the todo "Concurrent task" to the server during the first upload
    When I sync
    Then the sync succeeds
    And the uploads carried the ETags '"v1" "v2"'
    And the sync file on the server has the todo "Concurrent task"
    And the local todos include "Concurrent task"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: Repeated concurrent changes end in a readable error
    Given a WebDAV server with the folder "/nunc/" holding the sync file "webdav/nunc-sync.json"
    And the Go fixture database
    And another device changes the file on the server during every upload
    When I sync
    Then the sync fails with "Die Sync-Datei wurde auf dem Server gleichzeitig geändert"
    And the server received 3 uploads
  @integration @sync @US-Sync-02-WebDAV
  Scenario: Concurrent sync calls are serialized
    Given a sync coordinator whose runs take 80 ms
    When 3 syncs are requested at the same time
    Then 3 sync runs have happened one after the other
  @integration @sync @US-Sync-02-WebDAV
  Scenario: A sync runs at app start
    Given a sync coordinator whose runs take 0 ms
    When the app starts
    Then within 500 ms 1 sync runs have happened
  @integration @sync @US-Sync-02-WebDAV
  Scenario: A sync runs after a finished or aborted session
    Given a sync coordinator whose runs take 0 ms
    When a session finishes
    And a session is aborted
    Then within 500 ms 2 sync runs have happened
  @integration @sync @US-Sync-02-WebDAV
  Scenario: A sync runs after a todo change, debounced
    Given a sync coordinator whose runs take 0 ms
    And the debounce time is 1000 ms
    When 4 todos change within 80 ms
    Then no sync run has happened yet
    And within 3000 ms 1 sync runs have happened

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A todo change after a quiet period starts the next sync
    Given a sync coordinator whose runs take 0 ms
    And the debounce time is 50 ms
    When a todo changes
    And I wait 300 ms
    And a todo changes
    Then within 1000 ms 2 sync runs have happened
  @integration @sync @US-Sync-02-WebDAV @desktop
  Scenario: A sync runs at quit and delays closing by at most five seconds
    Given a sync coordinator whose runs take 100 ms
    And the quit limit is 1000 ms
    When the app quits
    Then closing waited for 1 finished sync run

  @integration @sync @US-Sync-02-WebDAV @desktop
  Scenario: A slow sync cannot hold up quitting beyond the limit
    Given a sync coordinator whose runs take 3000 ms
    And the quit limit is 200 ms
    When the app quits
    Then closing was delayed by less than 1500 ms

  @integration @sync @US-Sync-02-WebDAV @desktop
  Scenario: A todo change still waiting for its quiet period is covered by the quit sync
    Given a sync coordinator whose runs take 0 ms
    And the debounce time is 300 ms
    When a todo changes
    And the app quits
    Then within 800 ms 1 sync runs have happened

  @integration @sync @US-Sync-02-WebDAV @desktop
  Scenario: Resume and background are no reasons to sync on the desktop
    Given a sync coordinator whose runs take 0 ms
    When the app resumes
    And the app goes to the background
    Then within 300 ms 0 sync runs have happened
  @integration @sync @US-Sync-02-WebDAV @mobile
  Scenario: A sync runs on resume and when going to the background
    Given a sync coordinator whose runs take 0 ms
    And the coordinator runs on mobile
    When the app resumes
    And the app goes to the background
    Then within 500 ms 2 sync runs have happened
  @integration @sync @US-Sync-02-WebDAV
  Scenario: A sync reports its progress
    Given a sync coordinator whose runs take 0 ms
    And the next run imports 2 todos and 1 sessions with 0 conflicts
    When I sync now
    Then the observer saw: "start, status syncing, status idle, complete 2 1 0"
    And the sync status is "idle 2026-10-04T09:00:00Z -"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A failing sync shows an error and the next sync recovers
    Given a sync coordinator whose runs take 0 ms
    And the next run fails with "Verbindung abgelehnt – Server nicht erreichbar"
    When I sync now
    Then the sync call failed with "Verbindung abgelehnt – Server nicht erreichbar"
    And the sync status is "error - Verbindung abgelehnt – Server nicht erreichbar"
    And the observer saw: "start, status syncing, status error"
    When I sync now
    Then the sync status is "idle 2026-10-04T09:00:00Z -"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: The time of the last sync survives an error
    Given a sync coordinator whose runs take 0 ms
    When I sync now
    And the next run fails with "Zeitüberschreitung – Server antwortet nicht"
    And I sync now
    Then the sync status is "error 2026-10-04T09:00:00Z Zeitüberschreitung – Server antwortet nicht"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: Nothing is synchronized while no provider is selected
    Given a WebDAV server with the folder "/nunc/"
    And no database exists yet
    And the sync provider is "none"
    And the WebDAV settings point to the server for "me" with password "secret"
    When the WebDAV runner runs
    Then the runner did nothing
    And the server did not receive any request

  @integration @sync @US-Sync-02-WebDAV
  Scenario: Nothing is synchronized while the sync is paused
    Given a WebDAV server with the folder "/nunc/"
    And no database exists yet
    And the sync provider is "webdav"
    And the sync is paused
    And the WebDAV settings point to the server for "me" with password "secret"
    When the WebDAV runner runs
    Then the runner did nothing
    And the server did not receive any request

  @integration @sync @US-Sync-02-WebDAV
  Scenario: An incomplete setup is reported
    Given a WebDAV server with the folder "/nunc/"
    And no database exists yet
    And the sync provider is "webdav"
    And the WebDAV settings are incomplete
    When the WebDAV runner runs
    Then the runner failed with "WebDAV ist nicht vollständig eingerichtet"
    And the server did not receive any request

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A successful run records the time of the last sync
    Given a WebDAV server with the folder "/nunc/"
    And no database exists yet
    And the local todos "Task"
    And the sync provider is "webdav"
    And the WebDAV settings point to the server for "me" with password "secret"
    When the WebDAV runner runs
    Then the runner reports the finish time "2026-10-04T09:00:00Z"
    And the last sync time in the settings is "2026-10-04T09:00:00Z"
    And the sync file on the server has the todos "Task"

  @integration @sync @US-Sync-02-WebDAV
  Scenario: A failed run does not record a time
    Given a WebDAV server with the folder "/nunc/" that requires basic authentication for "me" with password "secret"
    And no database exists yet
    And the sync provider is "webdav"
    And the WebDAV settings point to the server for "me" with password "wrong"
    When the WebDAV runner runs
    Then the runner failed with "401 Unauthorized – Benutzername oder Passwort falsch"
    And the settings have no last sync time

  @ui @US-Sync-02-WebDAV
  Scenario: The status bar shows idle, syncing and error
    Given the setting "syncProvider" is "webdav"
    And the sync status is "idle" with the last sync at "2026-10-04T12:00:00Z"
    When I open the app
    Then I see "Zuletzt synchronisiert: 04.10.2026"
    When the backend reports the sync status "syncing"
    Then I see "Synchronisiere …"
    When the backend reports the sync status "error" with "401 Unauthorized – Benutzername oder Passwort falsch"
    Then I see "Sync-Fehler: 401 Unauthorized – Benutzername oder Passwort falsch"
    And I see the buttons "Erneut versuchen"
    When the backend reports the sync status "idle"
    Then I do not see "Sync-Fehler"

  @ui @US-Sync-02-WebDAV
  Scenario: The status bar says so when nothing was synced yet
    Given the setting "syncProvider" is "webdav"
    When I open the app
    Then I see "Noch nicht synchronisiert"

  @ui @US-Sync-02-WebDAV
  Scenario: The status bar is hidden while no sync provider is selected
    When I open the app
    Then I do not see "synchronisiert"
    And I do not see "Synchronisiere"
  @ui @US-Sync-02-WebDAV
  Scenario: A sync error is shown as a non-blocking notification
    Given the setting "syncProvider" is "webdav"
    When I open the app
    And the backend reports the sync status "error" with "Verbindung abgelehnt – Server nicht erreichbar"
    Then I see "Verbindung abgelehnt – Server nicht erreichbar"
    And I do not see a dialog
    And I see the buttons "Meldung schließen"
  @ui @US-Sync-02-WebDAV
  Scenario: The sync now button starts a sync
    Given the setting "syncProvider" is "webdav"
    And the next sync imports 2 sessions, 3 todos and 0 conflicts
    When I open the app at "/settings"
    And I click "Jetzt synchronisieren"
    Then the backend received "sync_now"
    And I see "2 Sessions und 3 Aufgaben importiert."
    And I see "Zuletzt synchronisiert: 04.10.2026"

  @ui @US-Sync-02-WebDAV
  Scenario: A sync without news says so
    Given the setting "syncProvider" is "webdav"
    When I open the app at "/settings"
    And I click "Jetzt synchronisieren"
    Then I see "Bereits auf dem neuesten Stand."

  @ui @US-Sync-02-WebDAV
  Scenario: Replaced todos are reported as a conflict hint
    Given the setting "syncProvider" is "webdav"
    And the next sync imports 0 sessions, 0 todos and 2 conflicts
    When I open the app at "/settings"
    And I click "Jetzt synchronisieren"
    Then I see "2 Aufgaben wurden durch neuere Versionen vom Server ersetzt."

  @ui @US-Sync-02-WebDAV
  Scenario: The sync now button is only offered while a provider is selected
    When I open the app at "/settings"
    Then I do not see the button "Jetzt synchronisieren"

  @ui @US-Sync-02-WebDAV
  Scenario: A failed manual sync shows the error and can be retried
    Given the setting "syncProvider" is "webdav"
    And the next sync fails with "Zeitüberschreitung – Server antwortet nicht"
    When I open the app at "/settings"
    And I click "Jetzt synchronisieren"
    Then I see "Sync-Fehler: Zeitüberschreitung – Server antwortet nicht"
    And I see the buttons "Erneut versuchen"

  @ui @US-Sync-02-WebDAV
  Scenario: Imported todos appear without reloading the app
    Given the setting "syncProvider" is "webdav"
    When I open the app
    And the backend finishes a sync that imported the todos:
      | rawText         |
      | From the server |
    And I click "Aufgaben"
    Then I see "From the server"
    And I see "1 Aufgabe importiert."

  @ui @US-Sync-02-WebDAV @mobile
  Scenario: Going to the background asks the backend for a last sync on mobile
    Given the platform is "mobile"
    And the setting "syncProvider" is "webdav"
    When I open the app
    And the app goes to the background
    Then the backend received "sync_in_background"

  @ui @US-Sync-02-WebDAV @desktop
  Scenario: The desktop does not sync when the window loses visibility
    Given the platform is "desktop"
    And the setting "syncProvider" is "webdav"
    When I open the app
    And the app goes to the background
    Then the backend did not receive "sync_in_background"

  @ui @US-Sync-02-WebDAV @mobile
  Scenario: Pulling down also synchronizes while a provider is selected
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    And the setting "syncProvider" is "webdav"
    When I open the app at "/todos"
    And I pull the page down by 200 pixels
    And I release the page
    Then the backend received "sync_now"

  @ui @US-Sync-02-WebDAV @mobile
  Scenario: Pulling down does not sync without a provider
    Given the platform is "mobile"
    And the viewport is 390 pixels wide
    When I open the app at "/todos"
    And I pull the page down by 200 pixels
    And I release the page
    Then the backend did not receive "sync_now"
