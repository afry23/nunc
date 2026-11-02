Feature: Journal CRDT
  Source: ADR 0026
  Day notes, week notes and achievements merge entry by entry. Notes have an id that follows from their day, the
  week note lives on the Sunday of its week, and an empty text deletes an entry.

  @domain @US-RV02
  Scenario Outline: The week note lives on the Sunday of its week
    Then the week of <day> ends on <sunday>

    Examples:
      | day        | sunday     |
      | 2026-03-02 | 2026-03-08 |
      | 2026-03-04 | 2026-03-08 |
      | 2026-03-08 | 2026-03-08 |
      | 2026-02-28 | 2026-03-01 |
      | 2026-12-30 | 2027-01-03 |

  @domain @US-RV02
  Scenario: A week note has the same id whichever day of the week it was written on
    When device A writes the week note "Good week" for 2026-03-04 at 1000
    And device B writes the week note "Busy week" for 2026-03-07 at 2000
    And the devices are merged at 5000
    Then the merged state has 1 journal entries
    And the merged entry "note:week:2026-03-08" reads "Busy week"
    And the merged entry "note:week:2026-03-08" is dated 2026-03-08

  @domain @US-RV02
  Scenario: The same day note written on two devices is one entry, the later write wins
    When device A writes the day note "from A" for 2026-03-02 at 1000
    And device B writes the day note "from B" for 2026-03-02 at 2000
    And the devices are merged at 5000
    Then the merged state has 1 journal entries
    And the merged entry "note:day:2026-03-02" reads "from B"

  @domain @US-RV02
  Scenario: Achievements of different devices are all kept
    When device A adds the achievement "Shipped" with id "a1" for 2026-03-02 at 1000
    And device B adds the achievement "Led workshop" with id "a2" for 2026-03-02 at 1500
    And the devices are merged at 5000
    Then the merged state has 2 journal entries
    And the merged entry "a1" reads "Shipped"
    And the merged entry "a2" reads "Led workshop"

  @domain @US-RV02
  Scenario: The merge does not depend on the order
    When device A writes the day note "from A" for 2026-03-02 at 1000
    And device B writes the day note "from B" for 2026-03-02 at 2000
    And device A adds the achievement "Shipped" with id "a1" for 2026-03-03 at 1000
    And device B adds the achievement "Shipped" with id "a1" for 2026-03-04 at 1000
    Then the merge does not depend on the order

  @domain @US-RV02
  Scenario: An empty text deletes a note and a later text brings it back
    When device A writes the day note "Plan" for 2026-03-02 at 1000
    And device A writes the day note "   " for 2026-03-02 at 2000
    And the devices are merged at 5000
    Then the merged entry "note:day:2026-03-02" is deleted
    When device A writes the day note " Done " for 2026-03-02 at 3000
    And the devices are merged at 5000
    Then the merged entry "note:day:2026-03-02" reads "Done"

  @domain @US-RV02
  Scenario: A newer edit on one device keeps an entry that another device deleted
    Given device A adds the achievement "Shipped" with id "a1" for 2026-03-02 at 1000
    And device B has a copy of the entry "a1" of device A
    When device A deletes the entry "a1" at 2000
    And device B moves the achievement "a1" to 2026-03-05 at 3000
    And the devices are merged at 5000
    Then the merged entry "a1" reads "Shipped"
    And the merged entry "a1" is dated 2026-03-05

  @domain @US-RV02
  Scenario: A newer delete wins over an older edit
    Given device A adds the achievement "Shipped" with id "a1" for 2026-03-02 at 1000
    And device B has a copy of the entry "a1" of device A
    When device B moves the achievement "a1" to 2026-03-05 at 2000
    And device A deletes the entry "a1" at 3000
    And the devices are merged at 5000
    Then the merged entry "a1" is deleted

  @domain @US-RV02
  Scenario: Only an achievement can be moved to another day
    When device A writes the day note "Plan" for 2026-03-02 at 1000
    And device A adds the achievement "Shipped" with id "a1" for 2026-03-02 at 1000
    Then moving the entry "note:day:2026-03-02" of device A to 2026-03-09 is refused
    And moving the entry "a1" of device A to 2026-03-09 is accepted

  @domain @US-RV02
  Scenario Outline: The text is limited to 4000 characters
    Then a text of <length> characters is <result>

    Examples:
      | length | result   |
      | 4000   | accepted |
      | 4001   | rejected |

  @domain @US-RV02
  Scenario: Surrounding white space is removed from a text
    Then the text "  Weekly  " is stored as "Weekly"

  @domain @US-RV02
  Scenario: Deleted entries are collected after 90 days, living ones never
    When device A writes the day note "Old" for 2026-03-02 at 1000
    And device A writes the day note "" for 2026-03-02 at 2000
    And device A writes the day note "Young" for 2026-03-03 at 7000000000
    And device A writes the day note "Gone" for 2026-03-04 at 7000000000
    And device A writes the day note "" for 2026-03-04 at 8000000000
    And device A writes the day note "Alive" for 2026-03-05 at 1000
    And the devices are merged at 8000000000
    And the garbage is collected at 8000000000
    Then the merged state has 3 journal entries
    And the merged entry "note:day:2026-03-02" does not exist
    And the merged entry "note:day:2026-03-03" reads "Young"
    And the merged entry "note:day:2026-03-04" is deleted
    And the merged entry "note:day:2026-03-05" reads "Alive"

  @domain @US-RV02
  Scenario: A remote clock far in the future is capped and a remote text is shortened
    When device B writes a day note of 4500 characters for 2026-03-02 at 999999999999999
    And device A writes the day note "Mine" for 2026-03-02 at 200000000
    And the devices are merged at 100000000
    Then the merged entry "note:day:2026-03-02" reads "Mine"
    When device B writes a day note of 4500 characters for 2026-03-03 at 1000
    And the devices are merged at 100000000
    Then the merged entry "note:day:2026-03-03" has 4000 characters

  @domain @US-RV02
  Scenario: A tombstone is kept up to exactly 90 days and collected after that
    When device A writes the day note "Old" for 2026-03-02 at 1000
    And device A writes the day note "" for 2026-03-02 at 2000
    And the devices are merged at 7776002000
    And the garbage is collected at 7776002000
    Then the merged entry "note:day:2026-03-02" is deleted
    When the garbage is collected at 7776002001
    Then the merged entry "note:day:2026-03-02" does not exist

  @domain @US-RV02
  Scenario: A remote date or delete stamp in the far future is capped too
    Given device A adds the achievement "Shipped" with id "a1" for 2026-03-02 at 1000
    And device B has a copy of the entry "a1" of device A
    When device B moves the achievement "a1" to 2026-03-05 at 999999999999999
    And device A moves the achievement "a1" to 2026-03-06 at 200000000
    And the devices are merged at 100000000
    Then the merged entry "a1" is dated 2026-03-06
    When device B deletes the entry "a1" at 999999999999999
    And device A edits the entry "a1" to "Shipped v2" at 300000000
    And the devices are merged at 100000000
    Then the merged entry "a1" reads "Shipped v2"

  @domain @US-RV02
  Scenario Outline: The kind is stored as text
    Then the kind <kind> is stored as "<text>"

    Examples:
      | kind        | text        |
      | DayNote     | day_note    |
      | WeekNote    | week_note   |
      | Achievement | achievement |

  @domain @property @US-RV02
  Scenario: Journal entries merge like a CRDT for random states
    Then the property "merging journal entries is commutative, associative and idempotent" holds for 300 random cases
