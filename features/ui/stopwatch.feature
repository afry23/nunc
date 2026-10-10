Feature: Stopwatch in the UI
  Source: ADR 0018

  @ui @US-B01
  Scenario: The stopwatch can be chosen as the timer mode
    When I open the app
    And I click "Stoppuhr"
    Then the backend received "save_settings" with:
      """
      { "settings": { "activeMode": "stopwatch" } }
      """

  @ui @US-B01
  Scenario: The stopwatch is started with its own mode
    Given the setting "activeMode" is "stopwatch"
    When I open the app
    And I click "Start"
    Then the backend received "start_timer" with:
      """
      { "mode": "stopwatch", "todoId": null }
      """

  @ui @US-B01
  Scenario: A running stopwatch counts up in its own color
    When I open the app
    And the backend reports the timer as:
      | state   | mode      | sessionType | remaining | total |
      | running | stopwatch | work        | 3725      | 0     |
    Then the ring shows "1:02:05"
    And the ring color is the "stopwatch" color
    And I see "Stoppuhr"

  @ui @US-B01
  Scenario: Stopping a stopwatch shows no break recommendation
    When I open the app
    And the backend reports a finished "stopwatch" session without a suggested break
    Then I do not see a dialog
