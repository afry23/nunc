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

  @integration @REQ-compat @wip
  Scenario: The WebDAV file of another device is merged with the local database

  @e2e @US-Sync-02-WebDAV @desktop @wip
  Scenario: Go app and nunc sync alternately against a WebDAV server
