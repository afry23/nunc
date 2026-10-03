Feature: Sync merge
  Source: US-S04, US-Sync-02-WebDAV; nunc-go sync_merge_test.go, sync_webdav_merge_test.go
  Todos: last write wins by updated_at (ties keep the local version). Sessions: append-only by device and start.

  @domain @US-S04
  Scenario: A todo that exists only remotely is added
    Given the local todos:
      | sync_id | text | updated |
    And the remote todos:
      | sync_id | text        | updated              |
      | a       | remote task | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | remote task |  |
    And the todos added by the remote are "a"
    And the todos taken over from the remote are ""

  @domain @US-S04
  Scenario: A todo that exists only locally is kept
    Given the local todos:
      | sync_id | text       | updated              |
      | a       | local task | 2026-03-01T10:00:00Z |
    And the remote todos:
      | sync_id | text | updated |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | local task |  |
    And the todos added by the remote are ""
    And the todos taken over from the remote are ""

  @domain @US-S04
  Scenario: The remote todo wins when it is newer
    Given the local todos:
      | sync_id | text      | updated              |
      | a       | old local | 2026-03-01T09:00:00Z |
    And the remote todos:
      | sync_id | text       | updated              |
      | a       | new remote | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | new remote |  |
    And the todos added by the remote are ""
    And the todos taken over from the remote are "a"

  @domain @US-S04
  Scenario: The local todo wins when it is newer
    Given the local todos:
      | sync_id | text      | updated              |
      | a       | new local | 2026-03-01T11:00:00Z |
    And the remote todos:
      | sync_id | text       | updated              |
      | a       | old remote | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | new local |  |
    And the todos taken over from the remote are ""

  @domain @US-S04
  Scenario: On equal timestamps the local todo wins
    Given the local todos:
      | sync_id | text  | updated              |
      | a       | local | 2026-03-01T10:00:00Z |
    And the remote todos:
      | sync_id | text   | updated              |
      | a       | remote | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | local |  |
    And the todos taken over from the remote are ""

  @domain @US-S04
  Scenario Outline: Timestamps are compared as points in time, not as text
    Given the local todos:
      | sync_id | text  | updated  |
      | a       | local | <local>  |
    And the remote todos:
      | sync_id | text   | updated   |
      | a       | remote | <remote>  |
    When I merge the remote state into the local state
    Then the todos taken over from the remote are "<taken>"

    Examples:
      | local                     | remote                    | taken |
      | 2026-03-01T10:00:00Z      | 2026-03-01T10:00:00.5Z    | a     |
      | 2026-03-01T10:00:00.5Z    | 2026-03-01T10:00:00Z      |       |
      | 2026-03-01T10:00:00Z      | 2026-03-01T11:00:00+02:00 |       |
      | 2026-03-01T10:00:00Z      | 2026-03-01T13:00:00+02:00 | a     |
      | 2026-03-01T10:00:00+00:00 | 2026-03-01T10:00:00Z      |       |

  @domain @US-S04
  Scenario Outline: A todo without a readable timestamp loses against one with a timestamp
    Given the local todos:
      | sync_id | text  | updated  |
      | a       | local | <local>  |
    And the remote todos:
      | sync_id | text   | updated   |
      | a       | remote | <remote>  |
    When I merge the remote state into the local state
    Then the todos taken over from the remote are "<taken>"

    Examples:
      | local                | remote               | taken |
      |                      | 2026-03-01T10:00:00Z | a     |
      | 2026-03-01T10:00:00Z |                      |       |
      |                      |                      |       |
      | not a time           | 2026-03-01T10:00:00Z | a     |
      | 2026-03-01T10:00:00Z | not a time           |       |

  @domain @US-S04
  Scenario: Todos without a sync id are skipped
    Given the local todos:
      | sync_id | text | updated |
    And the remote todos:
      | sync_id | text         | updated              |
      |         | no sync id   | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
    And the todos added by the remote are ""

  @domain @US-S04
  Scenario: The deleted flag is propagated
    Given the local todos:
      | sync_id | text | updated              | deleted |
      | a       | task | 2026-03-01T09:00:00Z |         |
    And the remote todos:
      | sync_id | text | updated              | deleted |
      | a       | task | 2026-03-01T10:00:00Z | yes     |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | task | yes |
    And the todos added by the remote are ""
    And the todos taken over from the remote are "a"

  @domain @US-S04
  Scenario: A newer local todo stays deleted
    Given the local todos:
      | sync_id | text | updated              | deleted |
      | a       | task | 2026-03-01T11:00:00Z | yes     |
    And the remote todos:
      | sync_id | text | updated              | deleted |
      | a       | task | 2026-03-01T10:00:00Z |         |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | task | yes |

  @domain @US-S04
  Scenario: Conflicting todo versions are counted
    Given the local todos:
      | sync_id | text        | updated              |
      | a       | old local a | 2026-03-01T09:00:00Z |
      | c       | local only  | 2026-03-01T09:00:00Z |
    And the remote todos:
      | sync_id | text         | updated              |
      | a       | new remote a | 2026-03-01T10:00:00Z |
      | b       | remote only  | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | new remote a |  |
      | c | local only   |  |
      | b | remote only  |  |
    And the todos added by the remote are "b"
    And the todos taken over from the remote are "a"

  @domain @US-S04
  Scenario: A todo that appears twice remotely is merged once
    Given the local todos:
      | sync_id | text | updated |
    And the remote todos:
      | sync_id | text   | updated              |
      | a       | first  | 2026-03-01T09:00:00Z |
      | a       | second | 2026-03-01T10:00:00Z |
      | a       | third  | 2026-03-01T08:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | second |  |
    And the todos added by the remote are "a"
    And the todos taken over from the remote are "a"

  @domain @US-S04
  Scenario: Sessions are appended and never overwritten
    Given the local sessions:
      | device | started              | actual |
      | dev-a  | 2026-03-01T10:00:00Z | 25     |
    And the remote sessions:
      | device | started              | actual |
      | dev-a  | 2026-03-01T10:00:00Z | 99     |
      | dev-b  | 2026-03-01T11:00:00Z | 25     |
    When I merge the remote state into the local state
    Then the merged sessions are:
      | dev-a | 2026-03-01T10:00:00Z | 25 |
      | dev-b | 2026-03-01T11:00:00Z | 25 |
    And 1 sessions are new from the remote

  @domain @US-S04
  Scenario: Sessions are not duplicated
    Given the local sessions:
      | device | started              |
      | dev-a  | 2026-03-01T10:00:00Z |
    And the remote sessions:
      | device | started              |
      | dev-a  | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged sessions are:
      | dev-a | 2026-03-01T10:00:00Z | 25 |
    And 0 sessions are new from the remote

  @domain @US-S04
  Scenario: Duplicate remote sessions are removed
    Given the local sessions:
      | device | started |
    And the remote sessions:
      | device | started              |
      | dev-b  | 2026-03-01T11:00:00Z |
      | dev-b  | 2026-03-01T11:00:00Z |
    When I merge the remote state into the local state
    Then the merged sessions are:
      | dev-b | 2026-03-01T11:00:00Z | 25 |
    And 1 sessions are new from the remote

  @domain @US-S04
  Scenario: Sessions with a different device id but identical payload are not duplicated
    Given the local sessions:
      | device    | started              | ended                | planned | actual |
      | dev-local | 2026-03-01T11:00:00Z | 2026-03-01T11:25:00Z | 25      | 25     |
    And the remote sessions:
      | device     | started              | ended                | planned | actual |
      | dev-remote | 2026-03-01T11:00:00Z | 2026-03-01T11:25:00Z | 25      | 25     |
    When I merge the remote state into the local state
    Then the merged sessions are:
      | dev-local | 2026-03-01T11:00:00Z | 25 |
    And 0 sessions are new from the remote

  @domain @US-S04
  Scenario: The same start in a different notation is the same session
    Given the local sessions:
      | device | started              |
      | dev-a  | 2026-03-01T10:00:00Z |
    And the remote sessions:
      | device | started                   |
      | dev-a  | 2026-03-01T12:00:00+02:00 |
    When I merge the remote state into the local state
    Then 0 sessions are new from the remote

  @domain @US-S04
  Scenario: Sessions of one device with different starts are different sessions
    Given the local sessions:
      | device | started              |
      | dev-a  | 2026-03-01T10:00:00Z |
    And the remote sessions:
      | device | started              |
      | dev-a  | 2026-03-01T10:00:01Z |
    When I merge the remote state into the local state
    Then 1 sessions are new from the remote

  @domain @US-S04
  Scenario: Merging with an empty remote state keeps the local state
    Given the local todos:
      | sync_id | text | updated              |
      | a       | task | 2026-03-01T10:00:00Z |
    And the local sessions:
      | device | started              |
      | dev-a  | 2026-03-01T10:00:00Z |
    And the remote todos:
      | sync_id | text | updated |
    And the remote sessions:
      | device | started |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | task |  |
    And the merged sessions are:
      | dev-a | 2026-03-01T10:00:00Z | 25 |

  @domain @US-S04
  Scenario: Merging with an empty local state takes over the remote state
    Given the local todos:
      | sync_id | text | updated |
    And the local sessions:
      | device | started |
    And the remote todos:
      | sync_id | text | updated              |
      | a       | task | 2026-03-01T10:00:00Z |
    And the remote sessions:
      | device | started              |
      | dev-b  | 2026-03-01T10:00:00Z |
    When I merge the remote state into the local state
    Then the merged todos are:
      | a | task |  |
    And the todos added by the remote are "a"
    And the merged sessions are:
      | dev-b | 2026-03-01T10:00:00Z | 25 |
    And 1 sessions are new from the remote

  @domain @US-S04 @property @wip
  Scenario: Merging is idempotent

  @domain @US-S04 @property @wip
  Scenario: Merging is commutative apart from conflict counts
