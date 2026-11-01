Feature: Day timeline
  Source: ADR 0017
  The sessions of one local day are shown as blocks on a time axis, with gaps and overlaps.

  @domain @US-TT04
  Scenario: Sessions of one day become blocks with start, end, type and todo
    Given the sessions:
      | start            | mode     | type        | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work        | completed | 60      | 1    |
      | 2026-03-02 10:00 | pomodoro | short_break | completed | 5       |      |
      | 2026-03-03 09:00 | flowtime | work        | completed | 30      | 2    |
    When I build the timeline for 2026-03-02
    Then the timeline blocks are:
      | start            | end              | type        | todo | overlaps |
      | 2026-03-02 09:00 | 2026-03-02 10:00 | work        | 1    | no       |
      | 2026-03-02 10:00 | 2026-03-02 10:05 | short_break |      | no       |

  @domain @US-TT04
  Scenario: Gaps between blocks are reported
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 30      |      |
      | 2026-03-02 10:00 | flowtime | work | completed | 30      |      |
      | 2026-03-02 10:30 | flowtime | work | completed | 30      |      |
    When I build the timeline for 2026-03-02
    Then the timeline gaps are:
      | start            | end              | minutes |
      | 2026-03-02 09:30 | 2026-03-02 10:00 | 30      |

  @domain @US-TT04
  Scenario: Overlapping blocks are marked
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 60      |      |
      | 2026-03-02 09:30 | flowtime | work | completed | 30      |      |
      | 2026-03-02 11:00 | flowtime | work | completed | 30      |      |
    When I build the timeline for 2026-03-02
    Then the timeline blocks are:
      | start            | end              | type | todo | overlaps |
      | 2026-03-02 09:00 | 2026-03-02 10:00 | work |      | yes      |
      | 2026-03-02 09:30 | 2026-03-02 10:00 | work |      | yes      |
      | 2026-03-02 11:00 | 2026-03-02 11:30 | work |      | no       |
    And the timeline gaps are:
      | start            | end              | minutes |
      | 2026-03-02 10:00 | 2026-03-02 11:00 | 60      |

  @domain @US-TT04
  Scenario: A session that crosses midnight is shown on the day it started
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 23:45 | flowtime | work | completed | 30      |      |
    When I build the timeline for 2026-03-02
    Then the timeline blocks are:
      | start            | end              | type | todo | overlaps |
      | 2026-03-02 23:45 | 2026-03-03 00:15 | work |      | no       |
    When I build the timeline for 2026-03-03
    Then the timeline has no blocks

  @domain @US-TT04
  Scenario: A block without a todo carries the own projects and contexts of its session
    Given the sessions:
      | start            | mode      | type | status    | minutes | todo | projects        | contexts |
      | 2026-03-02 09:00 | stopwatch | work | completed | 45      |      | +meetings +team | @office  |
    When I build the timeline for 2026-03-02
    Then the block starting 2026-03-02 09:00 has projects "+meetings +team" and contexts "@office"
