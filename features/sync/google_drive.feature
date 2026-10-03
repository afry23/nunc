Feature: Google Drive sync
  Source: US-Sync-GoogleDrive (S01-S07); desktop only in v1

  @ui @US-S01 @desktop @wip
  Scenario: The sync section shows not connected

  @integration @US-S01 @desktop @wip
  Scenario: Connecting uses OAuth2 with PKCE in the browser

  @integration @US-S01 @desktop @wip
  Scenario: Only the drive.file scope is requested

  @integration @US-S01 @desktop @wip
  Scenario: The token is stored in the secret store

  @ui @US-S01 @desktop @wip
  Scenario: The status shows the connected account

  @integration @US-S01 @desktop @wip
  Scenario: Disconnecting deletes the token locally

  @integration @US-S02 @desktop @wip
  Scenario: A sync runs at app start

  @integration @US-S02 @desktop @wip
  Scenario: A sync runs at quit and delays closing by at most five seconds

  @integration @US-S02 @desktop @wip
  Scenario: A failed sync does not prevent the app from starting

  @ui @US-S02 @desktop @wip
  Scenario: The sync status is shown while syncing

  @ui @US-S03 @desktop @wip
  Scenario: The sync now button is disabled while syncing

  @ui @US-S03 @desktop @wip
  Scenario: The result shows the number of imported sessions and todos

  @ui @US-S03 @desktop @wip
  Scenario: Errors are shown in plain text

  @integration @US-S04 @desktop @wip
  Scenario: Newer todo versions from another device win

  @ui @US-S04 @desktop @wip
  Scenario: The user is informed about taken-over newer versions

  @integration @US-S04 @desktop @wip
  Scenario: Sessions of both devices are merged

  @integration @US-S04 @desktop @wip
  Scenario: Deleted todos are propagated

  @integration @US-S05 @desktop @wip
  Scenario: The sync file is stored as nunc/nunc-sync.json

  @integration @US-S05 @desktop @wip
  Scenario: The folder is created when missing

  @integration @US-S05 @desktop @wip
  Scenario: A deleted remote file leads to uploading the local state

  @ui @US-S05 @desktop @wip
  Scenario: The drive path and a link to the folder are shown

  @integration @US-S06 @desktop @wip
  Scenario: An existing sync file is detected on a device without data

  @ui @US-S06 @desktop @wip
  Scenario: A dialog offers importing or starting empty

  @integration @US-S06 @desktop @wip
  Scenario: Importing takes over all sessions and todos

  @ui @US-S06 @desktop @wip
  Scenario: A progress bar and a summary are shown while importing

  @integration @US-S06 @desktop @wip
  Scenario: After the import the app works offline

  @ui @US-S07 @desktop @wip
  Scenario: Sync can be paused with a toggle

  @integration @US-S07 @desktop @wip
  Scenario: A paused sync makes no Drive access and keeps the token
