Feature: Review report
  Source: ADR 0025
  A review sums up a period from the raw sessions and todos and compares it with the period before.

  Background:
    Given the todos:
      | id | line                                  | pomodoros | position |
      | 1  | Write report +work @desk              | 0         | 1        |
      | 2  | x 2026-03-04 2026-03-02 Plan trip +home | 0       | 2        |
      | 3  | x 2026-02-25 2026-02-20 Old task +work  | 0       | 3        |
    And the sessions:
      | start            | mode     | type        | status    | minutes | todo |
      | 2026-02-23 09:00 | flowtime | work        | completed | 40      | 1    |
      | 2026-03-02 09:00 | flowtime | work        | completed | 60      | 1    |
      | 2026-03-02 10:30 | pomodoro | short_break | completed | 5       |      |
      | 2026-03-03 09:00 | pomodoro | work        | completed | 25      | 1    |
      | 2026-03-03 14:00 | flowtime | work        | aborted   | 30      | 2    |

  @domain @US-RV01
  Scenario: Focus time is summed for the period
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review focus time is 85 minutes

  @domain @US-RV01
  Scenario: Breaks and aborted sessions do not count as focus time
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review has 2 completed and 1 aborted work sessions
    And the review abort rate is 33 percent

  @domain @US-RV01
  Scenario: The period is compared with the period before
    When I build the review from 2026-03-02 to 2026-03-08
    Then the previous period is 2026-02-23 to 2026-03-01
    And the previous focus time is 40 minutes
    And the focus time changed by 45 minutes

  @domain @US-RV01
  Scenario: A calendar month is compared with the previous calendar month
    When I build the review from 2026-03-01 to 2026-03-31
    Then the previous period is 2026-02-01 to 2026-02-28
    And the previous focus time is 40 minutes

  @domain @US-RV01
  Scenario: A custom range is compared with the range of equal length before
    When I build the review from 2026-03-02 to 2026-03-04
    Then the previous period is 2026-02-27 to 2026-03-01

  @domain @US-RV01
  Scenario: Completed todos are listed by completion date in the period
    When I build the review from 2026-03-02 to 2026-03-08
    Then the completed todos are:
      | completed  | text            |
      | 2026-03-04 | Plan trip +home |

  @domain @US-RV01
  Scenario: Time is grouped by project
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review projects are:
      | group | minutes |
      | +work | 85      |

  @domain @US-RV01
  Scenario: Pomodoros, flowtime and stopwatch are counted separately
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo |
      | 2026-03-02 09:00 | pomodoro  | work | completed | 25      | 1    |
      | 2026-03-02 10:00 | pomodoro  | work | completed | 25      | 1    |
      | 2026-03-02 11:00 | pomodoro  | work | aborted   | 10      | 1    |
      | 2026-03-03 09:00 | flowtime  | work | completed | 40      | 1    |
      | 2026-03-03 14:00 | stopwatch | work | completed | 30      |      |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review has 2 pomodoros
    And the review modes are:
      | mode      | sessions | minutes |
      | pomodoro  | 2        | 50      |
      | flowtime  | 1        | 40      |
      | stopwatch | 1        | 30      |

  @domain @US-RV01
  Scenario: Every day with focus time shows minutes and pomodoros
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo |
      | 2026-03-03 09:00 | pomodoro  | work | completed | 25      | 1    |
      | 2026-03-02 09:00 | pomodoro  | work | completed | 25      | 1    |
      | 2026-03-02 10:00 | flowtime  | work | completed | 40      | 1    |
      | 2026-03-05 09:00 | pomodoro  | work | aborted   | 10      | 1    |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review days are:
      | date       | minutes | pomodoros |
      | 2026-03-02 | 65      | 1         |
      | 2026-03-03 | 25      | 1         |

  @domain @US-RV01
  Scenario: Time, pomodoros and sessions are summed per task
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo |
      | 2026-03-02 09:00 | pomodoro  | work | completed | 25      | 1    |
      | 2026-03-02 10:00 | pomodoro  | work | completed | 25      | 1    |
      | 2026-03-02 11:00 | flowtime  | work | completed | 40      | 1    |
      | 2026-03-03 09:00 | pomodoro  | work | completed | 25      | 2    |
      | 2026-03-03 10:00 | stopwatch | work | completed | 90      |      |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review tasks are:
      | text                     | minutes | pomodoros | sessions |
      | Write report +work @desk | 90      | 2         | 3        |
      | Plan trip +home          | 25      | 1         | 1        |

  @domain @US-RV01
  Scenario: The tags of a session stay together as one combination
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects | contexts       |
      | 2026-03-02 09:00 | stopwatch | work | completed | 45      |      | +foo     | @work @meeting |
      | 2026-03-02 10:00 | stopwatch | work | completed | 15      |      | +foo     | @meeting @work |
      | 2026-03-03 09:00 | stopwatch | work | completed | 20      |      | +foo     | @work          |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review tag combinations are:
      | projects | contexts       | sessions | minutes |
      | +foo     | @meeting @work | 2        | 60      |
      | +foo     | @work          | 1        | 20      |

  @domain @US-RV01
  Scenario: Session notes are listed under their combination, oldest first
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects | contexts       |
      | 2026-03-03 09:00 | stopwatch | work | completed | 15      |      | +foo     | @meeting @work |
      | 2026-03-02 09:00 | stopwatch | work | completed | 45      |      | +foo     | @work @meeting |
      | 2026-03-02 14:00 | stopwatch | work | completed | 20      |      | +foo     | @work          |
      | 2026-03-04 09:00 | stopwatch | work | completed | 10      |      | +foo     | @work          |
      | 2026-03-05 09:00 | stopwatch | work | aborted   | 10      |      | +foo     | @work          |
    And the session notes:
      | start            | note              |
      | 2026-03-03 09:00 | Follow-up         |
      | 2026-03-02 09:00 |   Weekly planning |
      | 2026-03-02 14:00 | Review draft      |
      | 2026-03-04 09:00 |                   |
      | 2026-03-05 09:00 | Aborted call      |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the combination "+foo @meeting @work" has the notes:
      | date       | text             |
      | 2026-03-02 | Weekly planning  |
      | 2026-03-03 | Follow-up        |
    And the combination "+foo @work" has the notes:
      | date       | text         |
      | 2026-03-02 | Review draft |

  @domain @US-RV01
  Scenario: Combinations come from the todo or from the session's own tags
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02 09:00 | flowtime  | work | completed | 60      | 1    |          |          |
      | 2026-03-02 10:00 | flowtime  | work | completed | 20      | 2    |          |          |
      | 2026-03-02 11:00 | flowtime  | work | completed | 10      | 3    |          |          |
      | 2026-03-03 09:00 | stopwatch | work | completed | 30      |      | +y +x    | @call    |
      | 2026-03-03 10:00 | stopwatch | work | completed | 5       |      |          | @call    |
      | 2026-03-03 11:00 | stopwatch | work | completed | 3       |      |          |          |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review tag combinations are:
      | projects | contexts | sessions | minutes |
      | +work    | @desk    | 1        | 60      |
      | +x +y    | @call    | 1        | 30      |
      | +home    | (none)   | 1        | 20      |
      | +work    | (none)   | 1        | 10      |
      | (none)   | @call    | 1        | 5       |
      | (none)   | (none)   | 1        | 3       |

  @domain @US-RV01
  Scenario: Combinations with equal time are ordered by projects, then contexts, without a tag last
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02 09:00 | stopwatch | work | completed | 10      |      |          |          |
      | 2026-03-02 10:00 | stopwatch | work | completed | 10      |      |          | @b       |
      | 2026-03-02 11:00 | stopwatch | work | completed | 10      |      | +a       |          |
      | 2026-03-02 12:00 | stopwatch | work | completed | 10      |      | +a       | @z       |
      | 2026-03-02 13:00 | stopwatch | work | completed | 10      |      | +a       | @c       |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review tag combinations are:
      | projects | contexts | sessions | minutes |
      | +a       | @c       | 1        | 10      |
      | +a       | @z       | 1        | 10      |
      | +a       | (none)   | 1        | 10      |
      | (none)   | @b       | 1        | 10      |
      | (none)   | (none)   | 1        | 10      |

  @domain @US-RV01
  Scenario Outline: A period is derived from a day
    When I ask for the <kind> containing <day>
    Then the period is <from> to <to>

    Examples:
      | kind    | day        | from       | to         |
      | week    | 2026-03-04 | 2026-03-02 | 2026-03-08 |
      | month   | 2026-02-10 | 2026-02-01 | 2026-02-28 |
      | quarter | 2026-05-15 | 2026-04-01 | 2026-06-30 |
      | quarter | 2026-06-30 | 2026-04-01 | 2026-06-30 |
      | quarter | 2026-11-20 | 2026-10-01 | 2026-12-31 |
      | month   | 2026-12-10 | 2026-12-01 | 2026-12-31 |
      | week    | 2026-03-08 | 2026-03-02 | 2026-03-08 |

  @domain @US-RV01
  Scenario: The abort rate is rounded to whole percent
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 10:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 11:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 12:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 13:00 | flowtime | work | completed | 10      | 1    |
      | 2026-03-02 14:00 | flowtime | work | aborted   | 10      | 1    |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review abort rate is 17 percent

  @domain @US-RV01
  Scenario: Time without a project or context is grouped as none
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 20      |      |
      | 2026-03-02 10:00 | flowtime | work | completed | 15      |      |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review projects are:
      | group  | minutes |
      | (none) | 35      |

  @domain @US-RV01
  Scenario: An empty period gives a zero review
    When I build the review from 2026-04-01 to 2026-04-07
    Then the review focus time is 0 minutes
    And the focus time changed by 0 minutes

  @domain @US-RV01
  Scenario: The review exports as Markdown
    When I build the review from 2026-03-02 to 2026-03-08
    And I export the review as Markdown in English
    Then the Markdown contains "# Review 2026-03-02 – 2026-03-08"
    And the Markdown contains "+45 min"
    And the Markdown contains "+work"
    And the Markdown contains "Plan trip"
    And the Markdown contains "## Projects"
    And the Markdown contains "- +work: 85 min"
    And the Markdown contains "## Contexts"
    And the Markdown contains "- @desk: 85 min"
    And the Markdown contains "## Timing technique"
    And the Markdown contains "- Pomodoros: 1"
    And the Markdown contains "- Flowtime: 1 sessions, 60 min"
    And the Markdown contains "- Write report +work @desk: 85 min, 1 Pomodoros, 2 sessions"
    And the Markdown contains "- +work @desk: 85 min, 2 sessions"
    And the Markdown contains "- 2026-03-03: 25 min, 1 Pomodoros"

  @domain @US-RV01
  Scenario: An empty review exports without detail sections
    When I build the review from 2026-04-01 to 2026-04-07
    And I export the review as Markdown in English
    Then the Markdown contains "- Pomodoros: 0"
    And the Markdown does not contain "## Timing technique"
    And the Markdown does not contain "## Projects"
    And the Markdown does not contain "## Tasks"
    And the Markdown does not contain "## Days"

  @domain @US-RV01
  Scenario: Session notes appear in the Markdown below their combination
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects | contexts       |
      | 2026-03-02 09:00 | stopwatch | work | completed | 45      |      | +foo     | @work @meeting |
    And the session notes:
      | start            | note            |
      | 2026-03-02 09:00 | Weekly planning |
    When I build the review from 2026-03-02 to 2026-03-08
    And I export the review as Markdown in English
    Then the Markdown contains:
      """
      - +foo @meeting @work: 45 min, 1 sessions
        - 2026-03-02: Weekly planning
      """

  @domain @US-RV01
  Scenario: The Markdown export follows the language of the app
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects | contexts |
      | 2026-03-02 09:00 | stopwatch | work | completed | 45      |      | +foo     |          |
      | 2026-03-02 10:00 | pomodoro  | work | aborted   | 10      |      |          |          |
    And the session notes:
      | start            | note            |
      | 2026-03-02 09:00 | Weekly planning |
    When I build the review from 2026-03-01 to 2026-03-08
    And I export the review as Markdown in German
    Then the Markdown contains "# Rückblick 2026-03-01 – 2026-03-08"
    And the Markdown contains "- Erfasste Arbeitszeit: 45 Min. (+45 Min. gegenüber 2026-02-21 – 2026-02-28)"
    And the Markdown contains "- Arbeitssessions: 1 abgeschlossen, 1 abgebrochen (50 % abgebrochen)"
    And the Markdown contains "## Zeitmessung"
    And the Markdown contains "- Stoppuhr: 1 Sessions, 45 Min."
    And the Markdown contains "## Projekte und Kontexte zusammen"
    And the Markdown contains "- +foo: 45 Min., 1 Sessions"
    And the Markdown contains "  - 2026-03-02: Weekly planning"
    And the Markdown does not contain "Tracked work time"

  @domain @US-RV02
  Scenario: The review lists achievements, week notes and day notes of the period
    Given the journal entries:
      | kind        | date       | text              | deleted |
      | achievement | 2026-03-05 | Shipped the beta  |         |
      | achievement | 2026-03-03 | Fixed the sync    |         |
      | achievement | 2026-02-27 | Before the period |         |
      | achievement | 2026-03-04 | Taken back        | yes     |
      | week_note   | 2026-03-04 | Good week         |         |
      | week_note   | 2026-02-25 | Previous week     |         |
      | day_note    | 2026-03-06 | Long Friday       |         |
      | day_note    | 2026-03-09 | After the period  |         |
    When I build the review from 2026-03-02 to 2026-03-08
    Then the review achievements are "2026-03-03: Fixed the sync | 2026-03-05: Shipped the beta"
    And the review week_notes are "2026-03-08: Good week"
    And the review day_notes are "2026-03-06: Long Friday"

  @domain @US-RV02
  Scenario: A month review shows the week notes of weeks that end in the month
    Given the journal entries:
      | kind      | date       | text            | deleted |
      | week_note | 2026-03-04 | Week ending 8th |         |
      | week_note | 2026-03-30 | Week ending 5th |         |
    When I build the review from 2026-03-01 to 2026-03-31
    Then the review week_notes are "2026-03-08: Week ending 8th"

  @domain @US-RV02
  Scenario: The Markdown export has sections for achievements, week review and day notes
    Given the journal entries:
      | kind        | date       | text                | deleted |
      | achievement | 2026-03-05 | Shipped the beta    |         |
      | week_note   | 2026-03-04 | Good week\nBusy one |         |
      | day_note    | 2026-03-06 | Long Friday         |         |
    When I build the review from 2026-03-02 to 2026-03-08
    And I export the review as Markdown in English
    Then the Markdown contains:
      """
      ## Achievements

      - 2026-03-05: Shipped the beta
      """
    And the Markdown contains:
      """
      ## Week review

      - 2026-03-08: Good week
        Busy one
      """
    And the Markdown contains:
      """
      ## Day notes

      - 2026-03-06: Long Friday
      """

  @domain @US-RV02
  Scenario: The Markdown export uses German headings for the journal
    Given the journal entries:
      | kind        | date       | text             | deleted |
      | achievement | 2026-03-05 | Shipped the beta |         |
      | week_note   | 2026-03-04 | Good week        |         |
      | day_note    | 2026-03-06 | Long Friday      |         |
    When I build the review from 2026-03-02 to 2026-03-08
    And I export the review as Markdown in German
    Then the Markdown contains "## Erfolge"
    And the Markdown contains "## Wochenrückblick"
    And the Markdown contains "## Tagesnotizen"

  @domain @US-RV02
  Scenario: Journal sections are left out when there are no entries
    When I build the review from 2026-03-02 to 2026-03-08
    And I export the review as Markdown in English
    Then the Markdown does not contain "Achievements"
    And the Markdown does not contain "Week review"
    And the Markdown does not contain "Day notes"

