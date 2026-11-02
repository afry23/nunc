Feature: Habits in the database
  Source: ADR 0028
  Habits and ticks are stored with a stamp per field. Two devices that exchange their state end up with the same
  habits and ticks.

  @integration @US-HB01
  Scenario: A habit is created and listed
    When device A creates the habit "h1" "  Read  " on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    Then on device A the habits list "Read [daily, check]"

  @integration @US-HB01
  Scenario: Habits are listed by the day they were created, then by id
    When device A creates the habit "h2" "Second" on 2026-03-03 with schedule "daily" and goal "check" at 10:00:00
    And device A creates the habit "h3" "Third" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A creates the habit "h1" "First" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    Then on device A the habits list "First [daily, check] | Third [daily, check] | Second [daily, check]"

  @integration @US-HB01
  Scenario: A habit needs a name of at most 100 characters
    When device A creates a habit with a name of 101 characters at 10:00:00
    Then the habits report "Der Name darf höchstens 100 Zeichen lang sein."
    And device A has 0 habit rows
    When device A creates the habit "h1" "  " on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    Then the habits report "Der Name darf nicht leer sein."
    And device A has 0 habit rows

  @integration @US-HB01
  Scenario: A name of exactly 100 characters is accepted
    When device A creates a habit with a name of 100 characters at 10:00:00
    Then device A has 1 habit rows

  @integration @US-HB01
  Scenario Outline: An invalid schedule or goal is rejected and nothing is stored
    When device A creates the habit "h1" "Read" on 2026-03-02 with schedule "<schedule>" and goal "<goal>" at 10:00:00
    Then the habits report "<message>"
    And device A has 0 habit rows

    Examples:
      | schedule | goal            | message                  |
      | weekly:8 | check           | Der Rhythmus ist ungültig. |
      | days:    | check           | Der Rhythmus ist ungültig. |
      | daily    | minutes:0:+work | Das Ziel ist ungültig.   |
      | daily    | minutes:30:work | Das Ziel ist ungültig.   |

  @integration @US-HB01
  Scenario: Only the changed fields of a habit get a new stamp
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    When device A changes the habit "h1" to "Read books" with schedule "daily" and goal "check" at 11:00:00
    Then on device A the field "name" of the habit "h1" was last written at 11:00:00
    And on device A the field "schedule" of the habit "h1" was last written at 10:00:00
    And on device A the field "goal" of the habit "h1" was last written at 10:00:00
    When device A changes the habit "h1" to "Read books" with schedule "weekly:3" and goal "minutes:30:+work" at 12:00:00
    Then on device A the field "name" of the habit "h1" was last written at 11:00:00
    And on device A the field "schedule" of the habit "h1" was last written at 12:00:00
    And on device A the field "goal" of the habit "h1" was last written at 12:00:00
    And on device A the habits list "Read books [weekly:3, minutes:30:+work]"

  @integration @US-HB01
  Scenario: Changing an unknown or deleted habit fails
    When device A changes the habit "nope" to "Read" with schedule "daily" and goal "check" at 10:00:00
    Then the habits report "Gewohnheit nope nicht gefunden"
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A deletes the habit "h1" at 11:00:00
    When device A changes the habit "h1" to "Back" with schedule "daily" and goal "check" at 12:00:00
    Then the habits report "Gewohnheit h1 nicht gefunden"

  @integration @US-HB01
  Scenario: A habit can be archived and brought back
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    When device A archives the habit "h1" at 11:00:00
    Then on device A the habits list "Read [daily, check] archived"
    When device A unarchives the habit "h1" at 12:00:00
    Then on device A the habits list "Read [daily, check]"

  @integration @US-HB01
  Scenario: Archiving a habit that is already archived does not stamp it again
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A archives the habit "h1" at 11:00:00
    When device A archives the habit "h1" at 12:00:00
    Then on device A the field "archived" of the habit "h1" was last written at 11:00:00

  @integration @US-HB01
  Scenario: A deleted habit is no longer listed
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    When device A deletes the habit "h1" at 11:00:00
    Then on device A the habits list ""
    When device A deletes the habit "h1" at 12:00:00
    Then the habits report "Gewohnheit h1 nicht gefunden"

  @integration @US-HB01
  Scenario: A tick is set, listed by day and removed again
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    When device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device A ticks the habit "h1" on 2026-03-05 at 11:00:00
    Then on device A the ticks from 2026-03-02 to 2026-03-04 list "2026-03-03 h1"
    And on device A the ticks from 2026-03-02 to 2026-03-05 list "2026-03-03 h1 | 2026-03-05 h1"
    When device A unticks the habit "h1" on 2026-03-03 at 12:00:00
    Then on device A the ticks from 2026-03-02 to 2026-03-05 list "2026-03-05 h1"

  @integration @US-HB01
  Scenario: Ticking twice or removing a tick that does not exist does not stamp anything
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    When device A ticks the habit "h1" on 2026-03-03 at 12:00:00
    And device A unticks the habit "h1" on 2026-03-04 at 12:00:00
    Then on device A the tick of the habit "h1" on 2026-03-03 was last written at 11:00:00
    And device A has 1 habit log rows

  @integration @US-HB01
  Scenario: A removed tick can be set again
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device A unticks the habit "h1" on 2026-03-03 at 12:00:00
    When device A ticks the habit "h1" on 2026-03-03 at 13:00:00
    Then on device A the ticks from 2026-03-03 to 2026-03-03 list "2026-03-03 h1"

  @integration @US-HB01
  Scenario: A habit that is measured from sessions has no ticks
    Given device A creates the habit "h1" "Deep work" on 2026-03-02 with schedule "daily" and goal "minutes:30:+work" at 10:00:00
    When device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    Then the habits report "Für diese Gewohnheit gibt es keine Haken (sie wird aus den Sessions berechnet)."
    And device A has 0 habit log rows

  @integration @US-HB01
  Scenario: Ticking an unknown habit fails
    When device A ticks the habit "nope" on 2026-03-03 at 11:00:00
    Then the habits report "Gewohnheit nope nicht gefunden"

  @integration @US-HB01
  Scenario: Habits and ticks written on one device appear on the other
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device B creates the habit "h2" "Run" on 2026-03-02 with schedule "weekly:3" and goal "check" at 10:00:00
    When the devices exchange their state
    Then on both devices the habits list "Read [daily, check] | Run [weekly:3, check]"
    And on both devices the ticks from 2026-03-02 to 2026-03-05 list "2026-03-03 h1"

  @integration @US-HB01
  Scenario: Two devices ticking different days of one habit keep both
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And the devices exchange their state
    When device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device B ticks the habit "h1" on 2026-03-04 at 11:00:00
    And the devices exchange their state
    Then on both devices the ticks from 2026-03-02 to 2026-03-05 list "2026-03-03 h1 | 2026-03-04 h1"

  @integration @US-HB01
  Scenario: Two devices ticking the same day end up with one tick
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And the devices exchange their state
    When device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device B ticks the habit "h1" on 2026-03-03 at 11:00:05
    And the devices exchange their state
    Then on both devices the ticks from 2026-03-02 to 2026-03-05 list "2026-03-03 h1"
    And device A has 1 habit log rows

  @integration @US-HB01
  Scenario: A tick removed later wins over the tick on the other device
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And the devices exchange their state
    When device B unticks the habit "h1" on 2026-03-03 at 12:00:00
    And the devices exchange their state
    Then on both devices the ticks from 2026-03-02 to 2026-03-05 list ""

  @integration @US-HB01
  Scenario: Different fields changed on two devices are both kept
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And the devices exchange their state
    When device A changes the habit "h1" to "Read books" with schedule "daily" and goal "check" at 11:00:00
    And device B archives the habit "h1" at 11:00:05
    And the devices exchange their state
    Then on both devices the habits list "Read books [daily, check] archived"

  @integration @US-HB01
  Scenario: A deletion reaches the other device
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And the devices exchange their state
    When device A deletes the habit "h1" at 11:00:00
    And the devices exchange their state
    Then on both devices the habits list ""

  @integration @US-HB01
  Scenario: A deleted habit that never reached this device is not created
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A deletes the habit "h1" at 11:00:00
    When the devices exchange their state
    Then device B has 0 habit rows

  @integration @US-HB01
  Scenario: A removed tick that never reached this device is not created
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And the devices exchange their state
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device A unticks the habit "h1" on 2026-03-03 at 12:00:00
    When the devices exchange their state
    Then device B has 0 habit log rows

  @integration @US-HB01
  Scenario: Exchanging identical habits changes nothing
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And the devices exchange their state
    When the devices exchange their state
    Then on device B the field "name" of the habit "h1" was last written at 10:00:00
    And on device B the tick of the habit "h1" on 2026-03-03 was last written at 11:00:00

  @integration @US-HB01
  Scenario: A deleted habit older than 90 days and its ticks are removed from the database
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device A deletes the habit "h1" at 12:00:00
    When the devices exchange their state on 2027-01-10
    Then device A has 0 habit rows
    And device A has 0 habit log rows

  @integration @US-HB01
  Scenario: A deleted habit younger than 90 days and its ticks are kept in the database
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device A deletes the habit "h1" at 12:00:00
    When the devices exchange their state on 2026-10-04
    Then device A has 1 habit rows
    And device A has 1 habit log rows

  @integration @US-HB01
  Scenario: A removed tick older than 90 days is removed from the database
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device A unticks the habit "h1" on 2026-03-03 at 12:00:00
    When the devices exchange their state on 2027-01-10
    Then device A has 1 habit rows
    And device A has 0 habit log rows

  @integration @US-HB01
  Scenario: A removed tick younger than 90 days is kept in the database
    Given device A creates the habit "h1" "Read" on 2026-03-02 with schedule "daily" and goal "check" at 10:00:00
    And device A ticks the habit "h1" on 2026-03-03 at 11:00:00
    And device A unticks the habit "h1" on 2026-03-03 at 12:00:00
    When the devices exchange their state on 2026-10-04
    Then device A has 1 habit log rows
