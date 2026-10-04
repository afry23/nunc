Feature: Sync file compatibility with the Go app
  Fixtures: tests/fixtures/nunc-sync.json, tests/fixtures/webdav/nunc-sync.json (written by the Go code)

  @domain @REQ-compat
  Scenario: A sync file written by the Go app is read
    When I read the sync file "nunc-sync.json"
    Then the file has schema version 1 and device "fixture-device-a"
    And the file has 6 todos and 11 sessions
    And the todo "t-0004" is completed on "2026-09-25"
    And the todo "t-0006" is marked as deleted

  @domain @REQ-compat
  Scenario: A sync file written by nunc is read by the Go format definition
    When I read the sync file "nunc-sync.json"
    And I write the sync file again
    Then the written todo "t-0003" has the keys "sync_id device_id raw_text completed created_at description project_tags context_tags pomodoro_count position updated_at deleted"
    And the written todo "t-0004" has the keys "sync_id device_id raw_text completed completed_at created_at description project_tags context_tags pomodoro_count position updated_at deleted"
    And the written todo "t-0001" has the keys "sync_id device_id raw_text priority completed created_at description project_tags context_tags pomodoro_count position updated_at deleted"

  @domain @REQ-compat
  Scenario Outline: The roundtrip Go to nunc to Go keeps all todos and sessions
    When I read the sync file "<file>"
    And I write the sync file again
    Then the written file equals the original file

    Examples:
      | file                  |
      | nunc-sync.json        |
      | webdav/nunc-sync.json |

  @domain @REQ-compat
  Scenario: The WebDAV file of another device is merged with the local sync file
    Given the sync file "nunc-sync.json" is the local state
    And the sync file "webdav/nunc-sync.json" is the remote state
    When I merge the remote file into the local file
    Then the merged file has 7 todos and 13 sessions
    And the todos added by the remote are "t-0100"
    And the todos taken over from the remote are "t-0002 t-0005"
    And 2 sessions are new from the remote
    And the merged todo "t-0002" says "(A) 2026-09-21 Call dentist tomorrow @phone"
    And the merged todo "t-0003" says "2026-09-22 Buy groceries +home @errands"
    And the merged file keeps the header of the local file

  @integration @sync @REQ-compat
  Scenario: The WebDAV file of another device is merged with the local database
    Given the Go fixture database
    And a WebDAV server with the folder "/nunc/" holding the sync file "webdav/nunc-sync.json"
    When I sync
    Then the sync succeeds with 1 imported todos, 2 imported sessions and 2 conflicts
    And the local database has the todos:
      | sync_id | rawText                                      | deleted |
      | t-0001  | (A) 2026-09-20 Write quarterly report +work @desk | false |
      | t-0002  | (A) 2026-09-21 Call dentist tomorrow @phone  | false   |
      | t-0003  | 2026-09-22 Buy groceries +home @errands      | false   |
      | t-0004  | x 2026-09-25 2026-09-18 Plan sprint +work    | false   |
      | t-0005  | (C) Read Rust book +learning +rust @evening | true    |
      | t-0006  | Old idea that was deleted                    | true    |
      | t-0100  | 2026-09-29 Water the plants @home            | false   |
    And the local database has 13 sessions

  @e2e @US-Sync-02-WebDAV @desktop @manual
  Scenario: Go app and nunc sync alternately against a WebDAV server
    Given the repository checkout contains ../nunc-go and Go is installed
    When I run "tests/interop/webdav-go-nunc.sh"
    Then the Go app logic and nunc synchronize alternately against one WebDAV server
    And the todos of the Go database and the nunc database are the same
    And a todo added, changed and deleted on either side reaches the other side
    And the completed sessions of both databases are the same
