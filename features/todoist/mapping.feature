Feature: Todoist field mapping and merge
  Source: US-Todoist (US-Todoist-Sync); nunc-go todoist/mapping_test.go

  @domain @US-Todoist-Sync @wip
  Scenario: Todoist priorities map to todo.txt priorities

  @domain @US-Todoist-Sync @wip
  Scenario: A Todoist project becomes a project tag

  @domain @US-Todoist-Sync @wip
  Scenario: Todoist labels become context tags

  @domain @US-Todoist-Sync @wip
  Scenario: Spaces in project and label names become underscores

  @domain @US-Todoist-Sync @wip
  Scenario: HTML markup is removed from the task content

  @domain @US-Todoist-Sync @wip
  Scenario: A completed task becomes a completed todo

  @domain @US-Todoist-Sync @wip
  Scenario: The creation date is converted to a date

  @domain @US-Todoist-Sync @wip
  Scenario: Todo.txt priorities map to Todoist priorities

  @domain @US-Todoist-Sync @wip
  Scenario: The first project tag becomes the Todoist project and the others labels

  @domain @US-Todoist-Sync @wip
  Scenario: A completed todo closes the Todoist task

  @domain @US-Todoist-Sync @wip
  Scenario: The Todoist version wins when it is newer

  @domain @US-Todoist-Sync @wip
  Scenario: The local version wins when it is newer

  @domain @US-Todoist-Sync @wip
  Scenario: A tie goes to Todoist

  @domain @US-Todoist-Sync @wip
  Scenario: A new local todo is created in Todoist

  @domain @US-Todoist-Sync @wip
  Scenario: A new Todoist task is created locally

  @domain @US-Todoist-Sync @wip
  Scenario: A task deleted in Todoist is marked completed locally

  @domain @US-Todoist-Sync @wip
  Scenario: A todo deleted locally closes the task in Todoist

  @domain @US-Todoist-Sync @wip
  Scenario: Completed remote tasks are skipped
