Feature: Todoist sync
  Source: US-Todoist (US-Todoist-Sync); desktop only in v1

  @integration @US-Todoist-Sync @desktop @wip
  Scenario: A sync runs at app start

  @integration @US-Todoist-Sync @desktop @wip
  Scenario: A sync runs after a todo change

  @integration @US-Todoist-Sync @desktop @wip
  Scenario: Concurrent syncs are serialized

  @integration @US-Todoist-Sync @desktop @wip
  Scenario: The mapping between todos and tasks is persisted

  @integration @US-Todoist-Sync @desktop @wip
  Scenario: Missing projects and labels are created in Todoist

  @integration @US-Todoist-Sync @desktop @wip
  Scenario: A deleted todo is closed in Todoist instead of being deleted

  @ui @US-Todoist-Sync @desktop @wip
  Scenario: The status shows idle, syncing and error

  @ui @US-Todoist-Sync @desktop @wip
  Scenario: The sync now button starts a sync

  @ui @US-Todoist-Sync @desktop @wip
  Scenario: A failed token refresh is shown as a non-blocking notification
