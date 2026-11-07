Feature: Journal in the database
  Source: ADR 0026
  Day notes, week notes and achievements are stored with a stamp per field. Two devices that exchange their state
  end up with the same journal.

  @integration @US-RV02
  Scenario: A day note is saved and listed for its day
    When device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    Then on device A the journal from 2026-03-02 to 2026-03-02 lists "2026-03-02 day_note: Calm day"

  @integration @US-RV02
  Scenario: Only entries inside the range are listed
    When device A writes the day note "Monday" for 2026-03-02 at 10:00:00
    And device A writes the day note "Tuesday" for 2026-03-03 at 10:00:00
    And device A writes the day note "Wednesday" for 2026-03-04 at 10:00:00
    Then on device A the journal from 2026-03-03 to 2026-03-03 lists "2026-03-03 day_note: Tuesday"
    And on device A the journal from 2026-03-02 to 2026-03-04 lists "2026-03-02 day_note: Monday | 2026-03-03 day_note: Tuesday | 2026-03-04 day_note: Wednesday"

  @integration @US-RV02
  Scenario: A week note written on any day of the week is stored under its Sunday
    When device A writes the week note "Good week" for 2026-03-04 at 10:00:00
    And device A writes the week note "Very good week" for 2026-03-06 at 11:00:00
    Then on device A the journal from 2026-03-02 to 2026-03-08 lists "2026-03-08 week_note: Very good week"
    And on device A the journal from 2026-03-02 to 2026-03-07 lists ""

  @integration @US-RV02
  Scenario: Text is trimmed before it is saved
    When device A writes the day note "  Calm day  " for 2026-03-02 at 10:00:00
    Then on device A the journal from 2026-03-02 to 2026-03-02 lists "2026-03-02 day_note: Calm day"

  @integration @US-RV02
  Scenario: An empty text deletes the note
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    When device A writes the day note "" for 2026-03-02 at 11:00:00
    Then on device A the journal from 2026-03-02 to 2026-03-02 lists ""

  @integration @US-RV02
  Scenario: Writing an empty text for a day without a note stores nothing
    When device A writes the day note "" for 2026-03-02 at 10:00:00
    Then device A has 0 journal rows

  @integration @US-RV02
  Scenario: A deleted note comes back when text is written again
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And device A writes the day note "" for 2026-03-02 at 11:00:00
    When device A writes the day note "Calm again" for 2026-03-02 at 12:00:00
    Then on device A the journal from 2026-03-02 to 2026-03-02 lists "2026-03-02 day_note: Calm again"

  @integration @US-RV02
  Scenario: Saving an unchanged text does not stamp the entry again
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    When device A writes the day note "Calm day" for 2026-03-02 at 11:00:00
    Then on device A the field "text" of the entry "note:day:2026-03-02" was last written at 10:00:00

  @integration @US-RV02
  Scenario: Deleting a missing note again does not stamp it again
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And device A writes the day note "" for 2026-03-02 at 11:00:00
    When device A writes the day note "" for 2026-03-02 at 12:00:00
    Then on device A the field "deleted" of the entry "note:day:2026-03-02" was last written at 11:00:00

  @integration @US-RV02
  Scenario: A text that is too long is rejected and nothing is stored
    When device A writes a day note of 4001 characters for 2026-03-02 at 10:00:00
    Then the journal reports "Der Text darf höchstens 4000 Zeichen lang sein."
    And device A has 0 journal rows

  @integration @US-RV02
  Scenario: A text of exactly 4000 characters is accepted
    When device A writes a day note of 4000 characters for 2026-03-02 at 10:00:00
    Then device A has 1 journal rows

  @integration @US-RV02
  Scenario: An achievement can be added, moved to another day, reworded and deleted
    Given device A adds the achievement "a-1" "Released v1" for 2026-03-02 at 10:00:00
    When device A changes the achievement "a-1" to 2026-03-03 "Released v1.0" at 11:00:00
    Then on device A the journal from 2026-03-02 to 2026-03-04 lists "2026-03-03 achievement: Released v1.0"
    When device A deletes the achievement "a-1" at 12:00:00
    Then on device A the journal from 2026-03-02 to 2026-03-04 lists ""

  @integration @US-RV02
  Scenario: Changing only the text of an achievement leaves its date stamp alone
    Given device A adds the achievement "a-1" "Released v1" for 2026-03-02 at 10:00:00
    When device A changes the achievement "a-1" to 2026-03-02 "Released v1.0" at 11:00:00
    Then on device A the field "text" of the entry "a-1" was last written at 11:00:00
    And on device A the field "date" of the entry "a-1" was last written at 10:00:00

  @integration @US-RV02
  Scenario: Changing only the day of an achievement leaves its text stamp alone
    Given device A adds the achievement "a-1" "Released v1" for 2026-03-02 at 10:00:00
    When device A changes the achievement "a-1" to 2026-03-03 "Released v1" at 11:00:00
    Then on device A the field "date" of the entry "a-1" was last written at 11:00:00
    And on device A the field "text" of the entry "a-1" was last written at 10:00:00

  @integration @US-RV02
  Scenario: An achievement needs a text
    When device A adds the achievement "a-1" "  " for 2026-03-02 at 10:00:00
    Then the journal reports "Der Text darf nicht leer sein."
    And device A has 0 journal rows

  @integration @US-RV02
  Scenario: An achievement cannot be reworded to nothing
    Given device A adds the achievement "a-1" "Released v1" for 2026-03-02 at 10:00:00
    When device A changes the achievement "a-1" to 2026-03-02 "" at 11:00:00
    Then the journal reports "Der Text darf nicht leer sein."
    And on device A the journal from 2026-03-02 to 2026-03-02 lists "2026-03-02 achievement: Released v1"

  @integration @US-RV02
  Scenario: Changing or deleting an unknown achievement fails
    When device A changes the achievement "nope" to 2026-03-02 "Text" at 10:00:00
    Then the journal reports "Journal-Eintrag nope nicht gefunden"

  @integration @US-RV02
  Scenario: A note is not an achievement
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    When device A deletes the achievement "note:day:2026-03-02" at 11:00:00
    Then the journal reports "Journal-Eintrag note:day:2026-03-02 nicht gefunden"
    And on device A the journal from 2026-03-02 to 2026-03-02 lists "2026-03-02 day_note: Calm day"

  @integration @US-RV02
  Scenario: A deleted achievement cannot be changed
    Given device A adds the achievement "a-1" "Released v1" for 2026-03-02 at 10:00:00
    And device A deletes the achievement "a-1" at 11:00:00
    When device A changes the achievement "a-1" to 2026-03-02 "Back" at 12:00:00
    Then the journal reports "Journal-Eintrag a-1 nicht gefunden"

  @integration @US-RV02
  Scenario: Entries written on one device appear on the other
    Given device A writes the day note "From A" for 2026-03-02 at 10:00:00
    And device B adds the achievement "b-1" "From B" for 2026-03-03 at 10:00:00
    When the devices exchange their state
    Then on both devices the journal from 2026-03-02 to 2026-03-03 lists "2026-03-02 day_note: From A | 2026-03-03 achievement: From B"

  @integration @US-RV02
  Scenario: Two devices writing the same week note keep the later text
    Given device A writes the week note "From A" for 2026-03-04 at 10:00:00
    And device B writes the week note "From B" for 2026-03-06 at 10:00:05
    When the devices exchange their state
    Then on both devices the journal from 2026-03-02 to 2026-03-08 lists "2026-03-08 week_note: From B"

  @integration @US-RV02
  Scenario: A deletion reaches the other device
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And the devices exchange their state
    When device A writes the day note "" for 2026-03-02 at 11:00:00
    And the devices exchange their state
    Then on both devices the journal from 2026-03-02 to 2026-03-02 lists ""

  @integration @US-RV02
  Scenario: A deleted entry that never reached this device is not created
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And device A writes the day note "" for 2026-03-02 at 11:00:00
    When the devices exchange their state
    Then device B has 0 journal rows

  @integration @US-RV02
  Scenario: A text edited after a concurrent deletion survives
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And the devices exchange their state
    When device A writes the day note "" for 2026-03-02 at 11:00:00
    And device B writes the day note "Better day" for 2026-03-02 at 12:00:00
    And the devices exchange their state
    Then on both devices the journal from 2026-03-02 to 2026-03-02 lists "2026-03-02 day_note: Better day"

  @integration @US-RV02
  Scenario: Exchanging identical journals changes nothing
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And the devices exchange their state
    When the devices exchange their state
    Then on device B the field "text" of the entry "note:day:2026-03-02" was last written at 10:00:00

  @integration @US-RV02
  Scenario: A deletion older than 90 days is removed from the database
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And device A writes the day note "" for 2026-03-02 at 11:00:00
    When the devices exchange their state on 2027-01-10
    Then device A has 0 journal rows

  @integration @US-RV02
  Scenario: A deletion younger than 90 days is kept in the database
    Given device A writes the day note "Calm day" for 2026-03-02 at 10:00:00
    And device A writes the day note "" for 2026-03-02 at 11:00:00
    When the devices exchange their state on 2026-10-04
    Then device A has 1 journal rows
