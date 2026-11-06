Feature: Stopwatch in the UI
  Source: ADR 0018

  @ui @US-B01
  Scenario: The stopwatch can be chosen as the timer mode
    When I open the app at "/focus"
    And I click "Stoppuhr"
    Then the backend received "save_settings" with:
      """
      { "settings": { "activeMode": "stopwatch" } }
      """

  @ui @US-B01
  Scenario: The stopwatch is started with its own mode
    Given the setting "activeMode" is "stopwatch"
    When I open the app at "/focus"
    And I click "Start"
    Then the backend received "start_timer" with:
      """
      { "mode": "stopwatch", "todoId": null }
      """

  @ui @US-B01
  Scenario: A running stopwatch counts up in its own color
    When I open the app at "/focus"
    And the backend reports the timer as:
      | state   | mode      | sessionType | remaining | total |
      | running | stopwatch | work        | 3725      | 0     |
    Then the ring shows "1:02:05"
    And the ring color is the "stopwatch" color
    And I see "Stoppuhr"

  @ui @US-B01
  Scenario: Stopping a stopwatch shows no break recommendation
    When I open the app at "/focus"
    And the backend reports a finished "stopwatch" session without a suggested break
    Then I do not see "Empfohlene Pause"
    And I do not see the button "Pause starten"

  @ui @US-B01
  Scenario: A stopwatch without a task is started with projects, contexts and a note
    Given the setting "activeMode" is "stopwatch"
    When I open the app at "/focus"
    And I click "Freie Tätigkeit starten"
    And I enter "meetings" into the field "Projekt hinzufügen"
    And I press the key "Enter" in the field "Projekt hinzufügen"
    And I enter "@office" into the field "Kontext hinzufügen"
    And I press the key "Enter" in the field "Kontext hinzufügen"
    And I enter "Weekly" into the field "Tätigkeit"
    And I click "Starten"
    Then the backend received "start_timer" with:
      """
      { "mode": "stopwatch", "todoId": null, "labels": { "projectTags": ["+meetings"], "contextTags": ["@office"], "note": "Weekly" } }
      """

  @ui @US-B01
  Scenario: A running stopwatch without a task shows its assignment
    When I open the app at "/focus"
    And the backend reports the timer as:
      | state   | mode      | sessionType | remaining | total | note   | projects  | contexts |
      | running | stopwatch | work        | 60        | 0     | Weekly | +meetings | @office  |
    Then I see "Weekly · +meetings · @office"

  @ui @US-B01
  Scenario: The free activity is offered only without a task
    Given the todos:
      | rawText      | completed |
      | Write report | false     |
    When I open the app at "/focus"
    Then I see the buttons "Freie Tätigkeit starten"
    When I select "Write report" in the field "Aktive Aufgabe"
    Then I do not see the button "Freie Tätigkeit starten"

  @ui @US-B01
  Scenario: A chosen project can be removed again
    When I open the app at "/focus"
    And I click "Freie Tätigkeit starten"
    And I enter "+meetings" into the field "Projekt hinzufügen"
    And I press the key "Enter" in the field "Projekt hinzufügen"
    Then I see "+meetings ×"
    When I click "+meetings entfernen"
    Then I do not see "+meetings ×"
