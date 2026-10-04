Feature: Todoist sync
  Source: US-Todoist (US-Todoist-Sync); desktop only in v1

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A sync runs at app start
    Given a Todoist server
    And Todoist is connected
    And no database exists yet
    And the local todos "Local task"
    And the coordinator runs the Todoist runner
    When the app starts
    Then within 3000 ms Todoist holds the task "Local task"

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A sync runs after a todo change
    Given a Todoist server
    And Todoist is connected
    And no database exists yet
    And the local todos "Changed task"
    And the coordinator runs the Todoist runner
    And the debounce time is 50 ms
    When a todo changes
    Then within 3000 ms Todoist holds the task "Changed task"

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: Concurrent syncs are serialized
    Given a sync coordinator whose runs take 100 ms
    When 3 syncs are requested at the same time
    Then 3 sync runs have happened one after the other

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: The mapping between todos and tasks is persisted
    Given a Todoist server
    And Todoist is connected
    And no database exists yet
    And the local todos "Write report, Call Anna"
    When I sync with Todoist
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And the local todos are mapped to the Todoist tasks:
      | todo         | task         |
      | Write report | Write report |
      | Call Anna    | Call Anna    |
    When I sync with Todoist
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And Todoist received 2 create, 0 update and 0 close requests
    And there are 2 Todoist mappings

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: Missing projects and labels are created in Todoist
    Given a Todoist server
    And Todoist is connected
    And Todoist knows the project "work" and the label "desk"
    And no database exists yet
    And the local todos "(A) Report +Work @desk @new, Plan +Home"
    When I sync with Todoist
    Then Todoist has the active tasks:
      | content | priority | project | labels    |
      | Report  | 4        | work    | desk, new |
      | Plan    | 1        | Home    |           |
    And Todoist knows the project "Home" once
    And Todoist knows the label "new" once
    And Todoist knows the label "desk" once

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A deleted todo is closed in Todoist instead of being deleted
    Given a Todoist server
    And Todoist is connected
    And Todoist holds the tasks:
      | content  |
      | Old task |
    And no database exists yet
    And the local todos "Old task"
    And the todo "Old task" is mapped to the Todoist task "Old task"
    And the todo "Old task" was deleted
    When I sync with Todoist
    Then Todoist closed the task "Old task"
    And Todoist deleted nothing
    And there are 0 Todoist mappings
    When I sync with Todoist
    Then Todoist received 0 create, 0 update and 1 close requests

  @ui @US-Todoist-Sync @desktop
  Scenario: The status shows idle, syncing and error
    Given Todoist is connected
    And the Todoist sync status is "idle" with the last sync at "12:00"
    When I open the app
    Then I see the Todoist status line "✓ Todoist: 12:00"
    When the backend reports the Todoist sync status "syncing"
    Then I see the Todoist status line "⟳ Todoist synchronisiert…"
    When the backend reports the Todoist sync status "error" with "Verbindung abgelehnt – Server nicht erreichbar"
    Then I see the Todoist status line "⚠ Todoist-Sync fehlgeschlagen"
    And the Todoist status line has the tooltip "Verbindung abgelehnt – Server nicht erreichbar"

  @ui @US-Todoist-Sync @desktop
  Scenario: The sync now button starts a sync
    Given Todoist is connected
    And the next Todoist sync imports 1 todos and replaces 0
    When I open the app at "/settings"
    And I click "Todoist jetzt synchronisieren"
    Then the backend received "todoist_sync_now"
    And I see "Todoist: 1 Aufgabe importiert."

  @ui @US-Todoist-Sync @desktop
  Scenario: A failed token refresh is shown as a non-blocking notification
    Given Todoist is connected
    And the next Todoist sync fails with "Todoist: Anmeldung abgelaufen – bitte neu verbinden"
    When I open the app at "/settings"
    And I click "Todoist jetzt synchronisieren"
    Then I see "Todoist: Anmeldung abgelaufen – bitte neu verbinden"
    And I do not see a dialog
    And I see the buttons "Meldung schließen"
    And the button "Todoist jetzt synchronisieren" is enabled

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: New tasks in Todoist become todos with project and labels
    Given a Todoist server
    And Todoist is connected
    And Todoist holds the tasks:
      | content  | priority | project | labels | updated              |
      | Buy milk | 4        | Home    | errand | 2026-10-03T10:00:00Z |
    And no database exists yet
    When I sync with Todoist
    Then the sync succeeds with 1 imported todos, 0 imported sessions and 0 conflicts
    And the local todos are "(A) 2026-10-04 Buy milk +Home @errand"
    And the local todo "(A) 2026-10-04 Buy milk +Home @errand" has the change time "2026-10-03T10:00:00Z"
    And the local todos are mapped to the Todoist tasks:
      | todo                                  | task     |
      | (A) 2026-10-04 Buy milk +Home @errand | Buy milk |

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A newer Todoist version replaces the local todo and the next sync is quiet
    Given a Todoist server
    And Todoist is connected
    And Todoist holds the tasks:
      | content   | priority |
      | New title | 3        |
    And no database exists yet
    And the local todos "Old title"
    And the todo "Old title" is mapped to the Todoist task "New title"
    When I sync with Todoist
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 1 conflicts
    And the local todos are "(B) 2026-10-04 New title"
    When I sync with Todoist
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And Todoist received 0 create, 0 update and 0 close requests

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A newer local version is sent to Todoist and the next sync is quiet
    Given a Todoist server
    And Todoist is connected
    And Todoist holds the tasks:
      | content | priority | updated              |
      | Old     | 1        | 2026-10-04T08:00:00Z |
    And no database exists yet
    And the local todos "(B) Newer text +Home @desk"
    And the todo "(B) Newer text +Home @desk" is mapped to the Todoist task "Old"
    When I sync with Todoist
    Then Todoist has the active tasks:
      | content    | priority | project | labels |
      | Newer text | 3        | Home    | desk   |
    And Todoist received 0 create, 1 update and 0 close requests
    When I sync with Todoist
    And I sync with Todoist
    Then the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts
    And Todoist received 0 create, 1 update and 0 close requests
    And the local todos are "(B) Newer text +Home @desk"

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A task closed in Todoist completes the local todo
    Given a Todoist server
    And Todoist is connected
    And Todoist holds the tasks:
      | content        |
      | Done elsewhere |
    And no database exists yet
    And the local todos "Done elsewhere"
    And the todo "Done elsewhere" is mapped to the Todoist task "Done elsewhere"
    And the Todoist task "Done elsewhere" was closed
    When I sync with Todoist
    Then the local todos are "x 2026-10-04 Done elsewhere"
    When I sync with Todoist
    Then the local todos are "x 2026-10-04 Done elsewhere"
    And the sync succeeds with 0 imported todos, 0 imported sessions and 0 conflicts

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A todo completed locally closes the task in Todoist
    Given a Todoist server
    And Todoist is connected
    And Todoist holds the tasks:
      | content  | updated              |
      | Finished | 2026-10-04T08:00:00Z |
    And no database exists yet
    And the local todos "x 2026-10-03 Finished"
    And the todo "x 2026-10-03 Finished" is mapped to the Todoist task "Finished"
    When I sync with Todoist
    Then Todoist closed the task "Finished"
    And Todoist received 0 create, 0 update and 1 close requests

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: Long lists are read page by page
    Given a Todoist server
    And Todoist is connected
    And Todoist lists 2 items per page
    And Todoist holds the tasks:
      | content |
      | One     |
      | Two     |
      | Three   |
      | Four    |
      | Five    |
    And no database exists yet
    When I sync with Todoist
    Then the sync succeeds with 5 imported todos, 0 imported sessions and 0 conflicts

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A failing Todoist is reported
    Given a Todoist server
    And Todoist is connected
    And Todoist answers every request with 503
    And no database exists yet
    When I sync with Todoist
    Then the sync fails with "Unerwartete Antwort des Servers (503) beim Laden der Aufgaben"

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: Nothing happens while Todoist is not connected
    Given a Todoist server
    And no database exists yet
    And the local todos "Task"
    When the Todoist runner runs
    Then the runner did nothing
    And Todoist was not contacted

  @integration @sync @US-Todoist-Sync @desktop
  Scenario: A successful run records the time of the last sync
    Given a Todoist server
    And Todoist is connected
    And no database exists yet
    When the Todoist runner runs
    Then the runner reports the finish time "2026-10-04T09:00:00Z"
    And the last Todoist sync time in the settings is "2026-10-04T09:00:00Z"

  @ui @US-Todoist-Sync @desktop
  Scenario: The status line is only shown while a Todoist account is connected
    When I open the app
    Then I do not see "Todoist:"

  @ui @US-Todoist-Sync @desktop
  Scenario: Tasks imported from Todoist appear without reloading the app
    Given Todoist is connected
    When I open the app
    And the backend finishes a Todoist sync that imported the todos:
      | rawText      |
      | From Todoist |
    Then I see "Todoist: 1 Aufgabe importiert."
