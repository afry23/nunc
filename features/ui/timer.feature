Feature: Timer view
  Source: REQUIREMENTS 2.1, 3, 4.3, 13; ring, controls, break suggestion

  @ui @REQ-3
  Scenario: The ring shows the remaining time as MM:SS
    When I open the app at "/focus"
    Then the ring shows "25:00"
    When the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 754       | 1500  |
    Then the ring shows "12:34"
    When the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 5         | 1500  |
    Then the ring shows "00:05"

  @ui @REQ-3
  Scenario: The ring color depends on the session type
    When I open the app at "/focus"
    And the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 1000      | 1500  |
    Then the ring color is the "pomodoro" color
    When the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | short_break | 200       | 300   |
    Then the ring color is the "shortbrk" color
    When the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | long_break  | 900       | 1500  |
    Then the ring color is the "longbrk" color

  @ui @REQ-3
  Scenario: A running timer pulses and a paused timer does not
    When I open the app at "/focus"
    And the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 1000      | 1500  |
    Then the ring is pulsing
    When the backend reports the timer as:
      | state  | mode     | sessionType | remaining | total |
      | paused | pomodoro | work        | 1000      | 1500  |
    Then the ring is not pulsing

  @ui @REQ-2.1
  Scenario: Start, pause, resume and stop buttons reflect the timer state
    When I open the app at "/focus"
    Then I see the buttons "Start"
    And I do not see the button "Stop"
    When I click "Start"
    Then the backend received "start_timer"
    When the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 1400      | 1500  |
    Then I see the buttons "Pause, Stop"
    And I do not see the button "Start"
    When I click "Pause"
    Then the backend received "pause_timer"
    When the backend reports the timer as:
      | state  | mode     | sessionType | remaining | total |
      | paused | pomodoro | work        | 1400      | 1500  |
    Then I see the buttons "Fortsetzen, Stop"
    When I click "Fortsetzen"
    Then the backend received "resume_timer"
    When I click "Stop"
    Then the backend received "stop_timer"
    When the backend reports the timer as:
      | state | mode     | sessionType | remaining | total |
      | idle  | pomodoro | work        | 0         | 0     |
    Then I see the buttons "Start"

  @ui @REQ-4.3
  Scenario: A todo can be selected before starting the timer
    Given the todos:
      | rawText                 |
      | (A) Bericht +work       |
      | Einkaufen @street       |
    When I open the app at "/focus"
    And I select "Einkaufen" in the field "Aktive Aufgabe"
    And I click "Start"
    Then the backend received "start_timer" with:
      """
      { "mode": "pomodoro", "todoId": 2 }
      """

  @ui @US-UM07
  Scenario: The result of a completed session can be noted before the break
    Given the backend suggests a break of 5 minutes
    When I open the app at "/focus"
    And the backend reports a finished "pomodoro" session with a suggested break of 5 minutes
    And I enter "Draft done, send to Sam" into the field "Ergebnis / nächste Schritte"
    And I click "Pause starten"
    Then the backend received "set_session_note" with:
      """
      { "id": 1, "note": "Draft done, send to Sam" }
      """
    And the backend received "start_break"

  @ui @US-UM07
  Scenario: A stopped session asks for the result and saves it
    When I open the app at "/focus"
    And the backend reports a finished "stopwatch" session without a suggested break
    Then I see a dialog with the text "Session beendet"
    When I enter "Next: call the client" into the field "Ergebnis / nächste Schritte"
    And I click "Speichern"
    Then the backend received "set_session_note" with:
      """
      { "id": 1, "note": "Next: call the client" }
      """
    And I do not see a dialog

  @ui @US-UM07
  Scenario: Skipping the result saves nothing
    When I open the app at "/focus"
    And the backend reports a finished "stopwatch" session without a suggested break
    And I click "Überspringen"
    Then the backend did not receive "set_session_note"
    And I do not see a dialog

  @ui @US-UM07
  Scenario: An existing note of the session is shown and kept when nothing changes
    Given the session 1 has the note "Weekly"
    When I open the app at "/focus"
    And the backend reports a finished "stopwatch" session without a suggested break
    Then a field holds "Weekly"
    When I click "Speichern"
    Then the backend did not receive "set_session_note"

  @ui @US-UM07
  Scenario: Without the setting a stopped session shows no dialog
    Given the setting "askSessionResult" is "false"
    When I open the app at "/focus"
    And the backend reports a finished "stopwatch" session without a suggested break
    Then I do not see a dialog

  @ui @US-UM07
  Scenario: Without the setting the break suggestion has no result field
    Given the setting "askSessionResult" is "false"
    And the backend suggests a break of 5 minutes
    When I open the app at "/focus"
    And the backend reports a finished "pomodoro" session with a suggested break of 5 minutes
    Then I see a dialog with the text "Empfohlene Pause: 5 Minuten"
    And I do not see "Ergebnis / nächste Schritte"

  @ui @US-UM07
  Scenario: A session under a minute still offers the break but no result field
    Given the backend suggests a break of 5 minutes
    When I open the app at "/focus"
    And the backend reports a finished "pomodoro" session of 0 minutes with a suggested break of 5 minutes
    Then I see a dialog with the text "Empfohlene Pause: 5 Minuten"
    And I do not see "Ergebnis / nächste Schritte"

  @ui @REQ-2.1
  Scenario: A break suggestion dialog appears after a completed pomodoro
    Given the backend suggests a break of 5 minutes
    When I open the app at "/focus"
    And the backend reports a finished "pomodoro" session with a suggested break of 5 minutes
    Then I see a dialog with the text "Empfohlene Pause: 5 Minuten"
    When I click "Pause starten"
    Then the backend received "start_break"
    And I do not see a dialog

  @ui @REQ-13
  Scenario: The remaining time is announced to screen readers
    When I open the app at "/focus"
    And the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | pomodoro | work        | 1499      | 1500  |
    Then the screen reader announces "Fokus: noch 24 Minuten"
    When the backend reports the timer as:
      | state  | mode     | sessionType | remaining | total |
      | paused | pomodoro | work        | 1499      | 1500  |
    Then the screen reader announces "Fokus: noch 24 Minuten, pausiert"
