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

  @integration @US-Sync-01-WebDAV-Config @wip
  Scenario: The connection test sends a PROPFIND request

  @integration @US-Sync-01-WebDAV-Config @wip
  Scenario: The password is stored in the secret store and not in the database

  @integration @US-Sync-01-WebDAV-Config @wip
  Scenario: The other settings are stored in the database

  @integration @US-Sync-02-WebDAV @wip
  Scenario: Basic authentication is used

  @integration @US-Sync-02-WebDAV @wip
  Scenario: Digest authentication is used

  @integration @US-Sync-02-WebDAV @wip
  Scenario: The remote folder is created with MKCOL when missing

  @integration @US-Sync-02-WebDAV @wip
  Scenario: The sync file is created on the first sync

  @integration @US-Sync-02-WebDAV @wip
  Scenario: A missing remote file is treated as an empty remote state

  @integration @US-Sync-02-WebDAV @wip
  Scenario: Sync downloads, merges and uploads the sync file

  @integration @US-Sync-02-WebDAV @wip
  Scenario: A concurrent change on the server is detected with the ETag

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
