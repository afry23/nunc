Feature: WebDAV sync
  Source: US-WebDAV (US-Sync-01-WebDAV-Config, US-Sync-02-WebDAV); nunc-go sync_webdav*.go

  @ui @US-Sync-01-WebDAV-Config @wip
  Scenario: The configuration offers URL, authentication method, user, password and remote path

  @ui @US-Sync-01-WebDAV-Config @wip
  Scenario: The password is masked and can be revealed

  @ui @US-Sync-01-WebDAV-Config @wip
  Scenario: An invalid URL disables saving and testing

  @ui @US-Sync-01-WebDAV-Config @wip
  Scenario: Changes are only applied on explicit save

  @ui @US-Sync-01-WebDAV-Config @wip
  Scenario: A banner shows that WebDAV is selected but not configured

  @ui @US-Sync-01-WebDAV-Config @wip
  Scenario: Testing the connection shows success with the server name

  @ui @US-Sync-01-WebDAV-Config @wip
  Scenario: Testing the connection shows a readable error

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
  @integration @sync @US-Sync-02-WebDAV
  Scenario: The remote folder is created with MKCOL when missing
    Given a WebDAV server without folders
    When I sync with the remote path "/files/nunc/"
    Then the sync succeeds
    And the server created the folders "/files/ /files/nunc/" in this order
    And the server holds the sync file "/files/nunc/nunc-sync.json"
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
    And the sync file on the server keeps the device "fixture-device-a"
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
  @integration @US-Sync-02-WebDAV @wip
  Scenario: Concurrent sync calls are serialized

  @integration @US-Sync-02-WebDAV @wip
  Scenario: A sync runs at app start

  @integration @US-Sync-02-WebDAV @wip
  Scenario: A sync runs after a finished or aborted session

  @integration @US-Sync-02-WebDAV @wip
  Scenario: A sync runs after a todo change, debounced

  @integration @US-Sync-02-WebDAV @desktop @wip
  Scenario: A sync runs at quit and delays closing by at most five seconds

  @integration @US-Sync-02-WebDAV @mobile @wip
  Scenario: A sync runs on resume and when going to the background

  @ui @US-Sync-02-WebDAV @wip
  Scenario: The status bar shows idle, syncing and error

  @ui @US-Sync-02-WebDAV @wip
  Scenario: A sync error is shown as a non-blocking notification

  @ui @US-Sync-02-WebDAV @wip
  Scenario: The sync now button starts a sync
