Feature: Flowtime in the UI
  Source: STORIES US-FT-02; stopwatch, break recommendation, settings controls

  @ui @US-FT-02
  Scenario: The ratio slider ranges from 1:0.1 to 1:0.5 in steps of 0.05
    When I open the app at "/settings"
    Then the slider "Arbeit-zu-Pause-Verhältnis" ranges from 0.1 to 0.5 in steps of 0.05

  @ui @US-FT-02
  Scenario: Anchor labels STRICT, BALANCED and RELAXED are shown below the slider
    When I open the app at "/settings"
    Then I see "Strikt (1 : 0.1)"
    And I see "Ausgewogen"
    And I see "Entspannt (1 : 0.5)"

  @ui @US-FT-02
  Scenario: The selected ratio is shown as a badge
    When I open the app at "/settings"
    Then the ratio badge shows "1 : 0.2"
    When I set the slider "Arbeit-zu-Pause-Verhältnis" to "0.35"
    Then the ratio badge shows "1 : 0.35"

  @ui @US-FT-02
  Scenario: The minimum flow state accepts 1 to 60 minutes
    When I open the app at "/settings"
    Then the field "Minimale Flow-Zeit" accepts values from 1 to 60
    When I enter "45" into the field "Minimale Flow-Zeit"
    And I click "Speichern"
    Then the backend received "save_settings" with:
      """
      { "settings": { "flowtimeMinFlowStateMin": 45 } }
      """

  @ui @US-FT-02
  Scenario: The dynamic break scaling info banner is shown
    When I open the app at "/settings"
    Then I see "Die Pausenlänge wird dynamisch aus der Arbeitszeit berechnet."

  @ui @US-FT-02
  Scenario: The stopwatch is displayed while a flowtime session runs
    When I open the app
    And the backend reports the timer as:
      | state   | mode     | sessionType | remaining | total |
      | running | flowtime | work        | 754       | 0     |
    Then the ring shows "12:34"
    And the ring color is the "flowtime" color
    And I see "Flowtime"

  @ui @US-FT-02
  Scenario: The break recommendation is shown after stopping
    Given the backend suggests a break of 10 minutes
    When I open the app
    And the backend reports a finished "flowtime" session with a suggested break of 10 minutes
    Then I see a dialog with the text "Empfohlene Pause: 10 Minuten"
    When I click "Später"
    Then I do not see a dialog
    And I see "Empfohlene Pause: 10 Minuten"
