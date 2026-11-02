Feature: Habit with a minutes goal
  Source: ADR 0028
  A habit with a minutes goal needs no ticks: a day counts when the work time on its project or context reaches the
  goal. Work intervals that were not skipped count, the tags of the todo win over those of the session.

  @domain @US-HB02
  Scenario: Work time of several sessions adds up over a day
    Given the todos for the habits:
      | id | line           |
      | 1  | Report +work   |
    And the work sessions:
      | start            | type  | status    | minutes | todo | projects | contexts |
      | 2026-03-02T08:00 | work  | completed | 25      | 1    |          |          |
      | 2026-03-02T10:00 | work  | completed | 35      |      | +work    |          |
      | 2026-03-03T08:00 | work  | completed | 25      | 1    |          |          |
    When I find the days with 60 minutes on "+work"
    Then the days are 2026-03-02

  @domain @US-HB02
  Scenario: Skipped sessions and breaks do not count, aborted ones do
    Given the work sessions:
      | start            | type        | status    | minutes | todo | projects | contexts |
      | 2026-03-02T08:00 | work        | completed | 20      |      | +work    |          |
      | 2026-03-02T09:00 | work        | skipped   | 40      |      | +work    |          |
      | 2026-03-02T10:00 | short_break | completed | 40      |      | +work    |          |
      | 2026-03-02T11:00 | work        | aborted   | 10      |      | +work    |          |
    When I find the days with 30 minutes on "+work"
    Then the days are 2026-03-02
    When I find the days with 31 minutes on "+work"
    Then there are no days

  @domain @US-HB02
  Scenario: The tags of the todo win over those of the session
    Given the todos for the habits:
      | id | line         |
      | 1  | Report +work |
    And the work sessions:
      | start            | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02T08:00 | work | completed | 30      | 1    | +other   | @home    |
    When I find the days with 30 minutes on "+work"
    Then the days are 2026-03-02
    When I find the days with 30 minutes on "+other"
    Then there are no days
    When I find the days with 30 minutes on "@home"
    Then there are no days

  @domain @US-HB02
  Scenario: A project tag and a context tag are not mixed up
    Given the work sessions:
      | start            | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02T08:00 | work | completed | 30      |      | +desk    |          |
      | 2026-03-03T08:00 | work | completed | 30      |      |          | @desk    |
    When I find the days with 30 minutes on "@desk"
    Then the days are 2026-03-03
    When I find the days with 30 minutes on "+desk"
    Then the days are 2026-03-02

  @domain @US-HB02
  Scenario: A session without minutes counts as zero
    Given the work sessions:
      | start            | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02T08:00 | work | completed |         |      | +work    |          |
    When I find the days with 1 minutes on "+work"
    Then there are no days

  @domain @US-HB02
  Scenario: A session of an unknown todo falls back to its own tags
    Given the work sessions:
      | start            | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02T08:00 | work | completed | 30      | 9    | +work    |          |
    When I find the days with 30 minutes on "+work"
    Then the days are 2026-03-02
