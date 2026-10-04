Feature: Todoist field mapping and merge
  Source: US-Todoist (US-Todoist-Sync); nunc-go todoist/mapping_test.go
  Imported todos are canonical todo.txt lines; the export drops the tag words from the content.

  @domain @US-Todoist-Sync
  Scenario Outline: Todoist priorities map to todo.txt priorities
    Given the Todoist task:
      | id | content | priority   |
      | 1  | Test    | <priority> |
    When I map the task to a todo
    Then the mapped priority is <result>

    Examples:
      | priority | result |
      | 4        | A      |
      | 3        | B      |
      | 2        | C      |
      | 1        | none   |
      | 0        | none   |
      | 5        | none   |

  @domain @US-Todoist-Sync
  Scenario Outline: A Todoist project becomes a project tag
    Given the Todoist task:
      | id | content | priority | project   |
      | 1  | Test    | 1        | <project> |
    When I map the task to a todo
    Then the mapped project tags are "<tags>"
    And the mapped description is "<description>"

    Examples:
      | project   | tags       | description          |
      | MyProject | +MyProject | Test +MyProject      |
      | Inbox     |            | Test                 |
      | inbox     |            | Test                 |
      | INBOX     |            | Test                 |
      |           |            | Test                 |

  @domain @US-Todoist-Sync
  Scenario: Todoist labels become context tags
    Given the Todoist task:
      | id | content | priority | labels      |
      | 1  | Test    | 1        | work, phone |
    When I map the task to a todo
    Then the mapped context tags are "@work @phone"
    And the mapped description is "Test @work @phone"

  @domain @US-Todoist-Sync
  Scenario Outline: Spaces in project and label names become underscores
    Given the Todoist task:
      | id | content | priority | project   | labels   |
      | 1  | Test    | 1        | <project> | <label>  |
    When I map the task to a todo
    Then the mapped project tags are "<project tag>"
    And the mapped context tags are "<context tag>"

    Examples:
      | project    | label    | project tag | context tag |
      | My Project | my label | +My_Project | @my_label   |
      | A B C      | x y z    | +A_B_C      | @x_y_z      |

  @domain @US-Todoist-Sync
  Scenario: HTML markup is removed from the task content
    Given the Todoist task:
      | id | content                                  | priority |
      | 1  | <b>Call</b> Anna &amp; Bob<br> &lt;now&gt; | 1        |
    When I map the task to a todo
    Then the mapped description is "Call Anna & Bob <now>"

  @domain @US-Todoist-Sync
  Scenario Outline: Comparison signs in the content are not mistaken for markup
    Given the Todoist task:
      | id | content   | priority |
      | 1  | <content> | 1        |
    When I map the task to a todo
    Then the mapped description is "<description>"

    Examples:
      | content                | description            |
      | 1 < 2 and 3 > 2        | 1 < 2 and 3 > 2        |
      | I <3 this              | I <3 this              |
      | a > b                  | a > b                  |
      | Read <i>this</i> first | Read this first        |
      | line<br/>break         | linebreak              |
      | <!-- note -->kept      | kept                   |
      | a &amp;amp; b         | a &amp; b              |

  @domain @US-Todoist-Sync
  Scenario: A completed task becomes a completed todo
    Given today is 2026-10-03
    And the Todoist task:
      | id | content | priority | project | labels | completed | created              |
      | 1  | Report  | 4        | Work    | desk   | yes       | 2026-01-02T08:30:00Z |
    When I map the task to a todo
    Then the mapped todo is completed on 2026-10-03
    And the mapped priority is none
    And the mapped raw text is "x 2026-10-03 2026-01-02 Report +Work @desk"

  @domain @US-Todoist-Sync
  Scenario: An open task has no completion date
    Given the Todoist task:
      | id | content | priority | created              |
      | 1  | Report  | 4        | 2026-01-02T08:30:00Z |
    When I map the task to a todo
    Then the mapped todo is open
    And the mapped raw text is "(A) 2026-01-02 Report"

  @domain @US-Todoist-Sync
  Scenario Outline: The creation date is converted to a date
    Given the Todoist task:
      | id | content | priority | created   |
      | 1  | Test    | 1        | <created> |
    When I map the task to a todo
    Then the mapped creation date is <date>

    Examples:
      | created              | date       |
      | 2026-01-02T08:30:00Z | 2026-01-02 |
      | 2026-12-31T23:59:59Z | 2026-12-31 |
      |                      | none       |

  @domain @US-Todoist-Sync
  Scenario Outline: Todo.txt priorities map to Todoist priorities
    Given the todo "<line>"
    When I map the todo to a Todoist task
    Then the Todoist priority is <priority>

    Examples:
      | line     | priority |
      | (A) test | 4        |
      | (B) test | 3        |
      | (C) test | 2        |
      | (D) test | 1        |
      | (Z) test | 1        |
      | test     | 1        |

  @domain @US-Todoist-Sync
  Scenario: The first project tag becomes the Todoist project and the others labels
    Given the todo "(A) Task with multiple projects +frontend +backend +infra @urgent"
    When I map the todo to a Todoist task
    Then the Todoist content is "Task with multiple projects"
    And the Todoist project is "frontend"
    And the Todoist labels are "backend,infra,urgent"

  @domain @US-Todoist-Sync
  Scenario: Underscores in tags become spaces again
    Given the todo "Plan trip +My_Project @my_label @home"
    When I map the todo to a Todoist task
    Then the Todoist project is "My Project"
    And the Todoist labels are "my label,home"

  @domain @US-Todoist-Sync
  Scenario: A todo without tags has no project and no labels
    Given the todo "Just text"
    When I map the todo to a Todoist task
    Then the Todoist content is "Just text"
    And the Todoist project is ""
    And the Todoist labels are ""

  @domain @US-Todoist-Sync
  Scenario: Only real tags are removed from the content
    Given the todo "Pay + tip @ home +bills"
    When I map the todo to a Todoist task
    Then the Todoist content is "Pay + tip @ home"

  @domain @US-Todoist-Sync
  Scenario: A completed todo closes the Todoist task
    Given the nunc todos:
      | id | line                               | updated              |
      | 1  | x 2026-10-03 Report                | 2026-10-03T10:00:00Z |
    And the Todoist tasks:
      | id   | content | updated              |
      | tid1 | Report  | 2026-10-02T10:00:00Z |
    And the mappings:
      | nunc_id | task_id |
      | 1       | tid1    |
    When I merge the Todoist state
    Then the tasks to close in Todoist are "tid1"
    And the todos to update in Todoist are ""
    And the local changes are:

  @domain @US-Todoist-Sync
  Scenario: The Todoist version wins when it is newer
    Given the nunc todos:
      | id | line      | updated              |
      | 1  | (A) Task  | 2026-10-03T08:00:00Z |
    And the Todoist tasks:
      | id   | content      | priority | updated              |
      | tid1 | Task updated | 3        | 2026-10-03T09:00:00Z |
    And the mappings:
      | nunc_id | task_id |
      | 1       | tid1    |
    When I merge the Todoist state
    Then the local changes are:
      | update | 1 | Task updated |
    And the todos to update in Todoist are ""

  @domain @US-Todoist-Sync
  Scenario: The local version wins when it is newer
    Given the nunc todos:
      | id | line             | updated              |
      | 1  | (B) Local update | 2026-10-03T09:00:00Z |
    And the Todoist tasks:
      | id   | content     | priority | updated              |
      | tid1 | Old content | 3        | 2026-10-03T08:00:00Z |
    And the mappings:
      | nunc_id | task_id |
      | 1       | tid1    |
    When I merge the Todoist state
    Then the todos to update in Todoist are "1"
    And the local changes are:

  @domain @US-Todoist-Sync
  Scenario: A tie goes to Todoist
    Given the nunc todos:
      | id | line      | updated              |
      | 1  | Old local | 2026-10-03T09:00:00Z |
    And the Todoist tasks:
      | id   | content        | updated              |
      | tid1 | Remote content | 2026-10-03T09:00:00Z |
    And the mappings:
      | nunc_id | task_id |
      | 1       | tid1    |
    When I merge the Todoist state
    Then the local changes are:
      | update | 1 | Remote content |
    And the todos to update in Todoist are ""

  @domain @US-Todoist-Sync
  Scenario Outline: The Todoist time falls back to the creation time and missing times lose
    Given the nunc todos:
      | id | line  | updated   |
      | 1  | Local | <local>   |
    And the Todoist tasks:
      | id   | content | created   | updated   |
      | tid1 | Remote  | <created> | <updated> |
    And the mappings:
      | nunc_id | task_id |
      | 1       | tid1    |
    When I merge the Todoist state
    Then the todos to update in Todoist are "<push>"

    Examples:
      | local                | created              | updated              | push |
      | 2026-10-03T09:00:00Z | 2026-10-03T08:00:00Z |                      | 1    |
      | 2026-10-03T09:00:00Z | 2026-10-03T10:00:00Z |                      |      |
      | 2026-10-03T09:00:00Z |                      |                      | 1    |
      |                      |                      |                      |      |
      |                      | 2026-10-03T08:00:00Z |                      |      |

  @domain @US-Todoist-Sync
  Scenario: A new local todo is created in Todoist
    Given the nunc todos:
      | id | line           | updated              |
      | 1  | New local task | 2026-10-03T09:00:00Z |
    And the Todoist tasks:
      | id | content |
    And the mappings:
      | nunc_id | task_id |
    When I merge the Todoist state
    Then the todos to create in Todoist are "1"

  @domain @US-Todoist-Sync
  Scenario: A completed or deleted local todo without a mapping is not created in Todoist
    Given the nunc todos:
      | id | line                   | updated              | deleted |
      | 1  | x 2026-10-01 Done      | 2026-10-03T09:00:00Z |         |
      | 2  | Removed                | 2026-10-03T09:00:00Z | yes     |
    And the Todoist tasks:
      | id | content |
    And the mappings:
      | nunc_id | task_id |
    When I merge the Todoist state
    Then the todos to create in Todoist are ""
    And the tasks to close in Todoist are ""

  @domain @US-Todoist-Sync
  Scenario: A new Todoist task is created locally
    Given the nunc todos:
      | id | line | updated |
    And the Todoist projects:
      | id | name  |
      | p1 | Work  |
    And the Todoist tasks:
      | id   | content         | priority | project_id | labels |
      | tid1 | New remote task | 4        | p1         | desk   |
    And the mappings:
      | nunc_id | task_id |
    When I merge the Todoist state
    Then the local changes are:
      | insert | tid1 | New remote task +Work @desk |

  @domain @US-Todoist-Sync
  Scenario: A task deleted in Todoist is marked completed locally
    Given the nunc todos:
      | id | line             | updated              |
      | 1  | Was remote task  | 2026-10-03T05:00:00Z |
    And the Todoist tasks:
      | id | content |
    And the mappings:
      | nunc_id | task_id  |
      | 1       | tid_gone |
    When I merge the Todoist state
    Then the todos to complete locally are "1"
    And the local changes are:

  @domain @US-Todoist-Sync
  Scenario: A todo deleted locally closes the task in Todoist
    Given the nunc todos:
      | id | line            | deleted |
      | 1  | Deleted locally | yes     |
    And the Todoist tasks:
      | id   | content         |
      | tid1 | Deleted locally |
    And the mappings:
      | nunc_id | task_id |
      | 1       | tid1    |
    When I merge the Todoist state
    Then the tasks to close in Todoist are "tid1"
    And the local changes are:
    And the todos to complete locally are ""

  @domain @US-Todoist-Sync
  Scenario: Completed remote tasks are skipped
    Given the nunc todos:
      | id | line | updated |
    And the Todoist tasks:
      | id   | content      | completed |
      | tid1 | Already done | yes       |
    And the mappings:
      | nunc_id | task_id |
    When I merge the Todoist state
    Then the local changes are:

  @domain @US-Todoist-Sync
  Scenario: A task that is already mapped is not imported again
    Given the nunc todos:
      | id | line | updated |
    And the Todoist tasks:
      | id   | content    |
      | tid1 | Known task |
    And the mappings:
      | nunc_id | task_id |
      | 7       | tid1    |
    When I merge the Todoist state
    Then the local changes are:
    And the todos to complete locally are ""

  @domain @US-Todoist-Sync
  Scenario: Several changes are reported in the order of the input
    Given the nunc todos:
      | id | line   | updated              |
      | 1  | First  | 2026-10-03T09:00:00Z |
      | 2  | Second | 2026-10-03T09:00:00Z |
      | 3  | Third  | 2026-10-03T09:00:00Z |
    And the Todoist tasks:
      | id | content |
    And the mappings:
      | nunc_id | task_id |
    When I merge the Todoist state
    Then the todos to create in Todoist are "1 2 3"
