Feature: Manual time entries
  Source: ADR 0017
  Forgotten work can be added afterwards. Manual entries are stored as completed flowtime work sessions.

  Background:
    Given the current time is "2026-03-02 12:00"

  @domain @US-TT03
  Scenario: A valid manual entry is accepted
    When I check a manual entry from "2026-03-02 09:00" to "2026-03-02 10:30"
    Then the entry is accepted without warnings

  @domain @US-TT03
  Scenario Outline: An invalid range is rejected
    When I check a manual entry from "<start>" to "<end>"
    Then the entry is rejected with "<message>"

    Examples:
      | start            | end              | message                                       |
      | 2026-03-02 09:00 | 2026-03-02 09:00 | Das Ende muss nach dem Beginn liegen.         |
      | 2026-03-02 09:00 | 2026-03-02 08:00 | Das Ende muss nach dem Beginn liegen.         |
      | 2026-03-01 09:00 | 2026-03-02 09:01 | Ein Eintrag darf höchstens 24 Stunden lang sein. |
      | 2026-03-02 11:00 | 2026-03-02 12:01 | Das Ende darf nicht in der Zukunft liegen.    |

  @domain @US-TT03
  Scenario: An entry of exactly 24 hours that ends now is accepted
    When I check a manual entry from "2026-03-01 12:00" to "2026-03-02 12:00"
    Then the entry is accepted without warnings

  @domain @US-TT03
  Scenario: An entry that overlaps another session is accepted with a warning
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 60      |      |
    When I check a manual entry from "2026-03-02 09:30" to "2026-03-02 10:30"
    Then the entry is accepted with a warning about 1 overlapping session

  @domain @US-TT03
  Scenario: An entry that only touches another session does not overlap
    Given the sessions:
      | start            | mode     | type | status    | minutes | todo |
      | 2026-03-02 09:00 | flowtime | work | completed | 60      |      |
    When I check a manual entry from "2026-03-02 10:00" to "2026-03-02 11:00"
    Then the entry is accepted without warnings
