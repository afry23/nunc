Feature: Habit CRDT
  Source: ADR 0028
  Habits merge field by field, a tick for a day has an id that follows from habit and day, and removed ticks and
  deleted habits are tombstones that are collected after 90 days.

  @domain @US-HB01
  Scenario: Different fields changed on two devices are both kept
    Given device A creates the habit "h1" named "Read" at 1000
    And device B has a copy of the habit "h1" of device A
    When device A renames the habit "h1" to "Read books" at 2000
    And device B sets the schedule of the habit "h1" to "weekly:3" at 3000
    And the devices are merged at 5000
    Then the merged habit "h1" is named "Read books"
    And the merged habit "h1" has the schedule "weekly:3"

  @domain @US-HB01
  Scenario: The later rename wins
    Given device A creates the habit "h1" named "Read" at 1000
    And device B has a copy of the habit "h1" of device A
    When device A renames the habit "h1" to "From A" at 2000
    And device B renames the habit "h1" to "From B" at 3000
    And the devices are merged at 5000
    Then the merged habit "h1" is named "From B"

  @domain @US-HB01
  Scenario: Goal and archiving are registers too
    Given device A creates the habit "h1" named "Read" at 1000
    And device B has a copy of the habit "h1" of device A
    When device A sets the goal of the habit "h1" to "minutes:30:+work" at 2000
    And device B archives the habit "h1" at 3000
    And the devices are merged at 5000
    Then the merged habit "h1" has the goal "minutes:30:+work"
    And the merged habit "h1" is archived

  @domain @US-HB01
  Scenario: The same day ticked on two devices is one log, the later write wins
    When device A ticks the habit "h1" on 2026-03-02 at 1000
    And device B unticks the habit "h1" on 2026-03-02 at 2000
    And the devices are merged at 5000
    Then the merged state has 1 logs
    And the merged log of "h1" on 2026-03-02 is undone
    When device A ticks the habit "h1" on 2026-03-02 at 3000
    And the devices are merged at 5000
    Then the merged log of "h1" on 2026-03-02 is done

  @domain @US-HB01
  Scenario: Ticks on different days are all kept
    When device A ticks the habit "h1" on 2026-03-02 at 1000
    And device B ticks the habit "h1" on 2026-03-03 at 1000
    And the devices are merged at 5000
    Then the merged state has 2 logs

  @domain @US-HB01
  Scenario: The merge does not depend on the order
    Given device A creates the habit "h1" named "Read" at 1000
    And device B creates the habit "h1" named "Write" at 2000
    When device A ticks the habit "h1" on 2026-03-02 at 1000
    And device B unticks the habit "h1" on 2026-03-02 at 2000
    Then the merge does not depend on the order

  @domain @US-HB01
  Scenario: The id of a log follows from habit and day
    Then the log id of "h1" on 2026-03-02 is "log:h1:2026-03-02"

  @domain @US-HB01
  Scenario: A newer delete wins over an older edit, a newer edit brings the habit back
    Given device A creates the habit "h1" named "Read" at 1000
    And device B has a copy of the habit "h1" of device A
    When device B renames the habit "h1" to "Read more" at 2000
    And device A deletes the habit "h1" at 3000
    And the devices are merged at 5000
    Then the merged habit "h1" is deleted
    When device B renames the habit "h1" to "Back" at 4000
    And the devices are merged at 5000
    Then the merged habit "h1" is named "Back"
    And the merged habit "h1" is not deleted

  @domain @US-HB01
  Scenario Outline: Tombstones are collected after 90 days
    Given device A creates the habit "h1" named "Read" at 1000
    And device A ticks the habit "h1" on 2026-03-02 at 1000
    When device A <change> at <at>
    And the devices are merged at 5000
    And the garbage is collected at <now>
    Then the merged state has <habits> habits and <logs> logs

    Examples:
      | change                                  | at   | now         | habits | logs |
      | deletes the habit "h1"                  | 2000 | 7776002001  | 0      | 0    |
      | deletes the habit "h1"                  | 2000 | 7776002000  | 1      | 1    |
      | unticks the habit "h1" on 2026-03-02    | 2000 | 7776002001  | 1      | 0    |
      | unticks the habit "h1" on 2026-03-02    | 2000 | 7776002000  | 1      | 1    |
      | renames the habit "h1" to "Still here"  | 2000 | 7776002001  | 1      | 1    |

  @domain @US-HB01
  Scenario: Logs of a habit that is gone are collected
    When device A ticks the habit "ghost" on 2026-03-02 at 1000
    And the devices are merged at 5000
    And the garbage is collected at 6000
    Then the merged state has 0 habits and 0 logs

  @domain @US-HB01
  Scenario: Fields of another device are capped and the name is shortened
    Given device B creates a habit "h1" named with 150 characters at 9999999999999
    When the devices are merged at 5000
    Then the merged habit "h1" has a name of 100 characters
    And the merged habit "h1" has a stamp not after 5000

  @domain @US-HB01
  Scenario: The stamp of a tick from another device is capped
    Given device B ticks the habit "h1" on 2026-03-02 at 9999999999999
    When the devices are merged at 5000
    Then the merged log of "h1" on 2026-03-02 has a stamp not after 5000

  @domain @US-HB01
  Scenario: The earlier creation day is kept
    Given device A creates the habit "h1" named "Read" on 2026-03-05 at 1000
    And device B creates the habit "h1" named "Read" on 2026-03-02 at 1000
    When the devices are merged at 5000
    Then the merged habit "h1" was created on 2026-03-02

  @domain @US-HB01
  Scenario: Habits and logs survive writing and reading the sync file
    Given device A creates the habit "h1" named "Read" at 1000
    And device A ticks the habit "h1" on 2026-03-02 at 1000
    When the habits of device A are written to a sync file and read back
    Then the merged state has 1 habits and 1 logs

  @domain @US-HB01
  Scenario: A version 7 sync file without habits is still read
    When a version 7 sync file without habits is read
    Then the merged state has 0 habits and 0 logs
