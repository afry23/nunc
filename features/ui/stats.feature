Feature: Statistics view
  Source: REQUIREMENTS 10, 11; periods, charts, streaks, export

  @ui @REQ-10
  Scenario: Switch between day, week, month and year
    When I open the app at "/stats"
    Then the statistics were requested for the period "week"
    When I click "Tag"
    Then the statistics were requested for the period "day"
    When I click "Monat"
    Then the statistics were requested for the period "month"
    When I click "Jahr"
    Then the statistics were requested for the period "year"

  @ui @REQ-10
  Scenario: Bar, line, heatmap calendar and pie charts render
    Given the statistics contain daily data:
      | daysAgo | pomodoros | minutes |
      | 0       | 4         | 100     |
      | 1       | 2         | 50      |
      | 3       | 6         | 150     |
    And the statistics contain the projects:
      | project | minutes |
      | +work   | 200     |
      | +home   | 100     |
    When I open the app at "/stats"
    Then I see the charts "bar, line, heatmap, pie"
    And the statistic "Fokuszeit" is "5h"

  @ui @REQ-10
  Scenario: An empty period shows an empty state
    When I open the app at "/stats"
    Then I see "Keine Sitzungen im gewählten Zeitraum."
    And I do not see the charts "bar, line, heatmap, pie"

  @ui @REQ-10.4
  Scenario: The current and the longest streak are shown
    Given the statistics contain daily data:
      | daysAgo | pomodoros | minutes |
      | 0       | 1         | 25      |
    And the statistics have a current streak of 3 and a longest streak of 8
    When I open the app at "/stats"
    Then the statistic "Aktuelle Serie" is "3 Tage"
    And the statistic "Längste Serie" is "8 Tage"

  @ui @REQ-11
  Scenario: The export button offers CSV and JSON
    Given the statistics contain daily data:
      | daysAgo | pomodoros | minutes |
      | 0       | 1         | 25      |
    When I open the app at "/stats"
    And I click "Exportieren"
    Then I see the buttons "CSV, JSON"

  @ui @REQ-11 @desktop
  Scenario: Saving the export writes a file
    Given the statistics contain daily data:
      | daysAgo | pomodoros | minutes |
      | 0       | 1         | 25      |
    And the export is saved to "/tmp/nunc-export.json"
    When I open the app at "/stats"
    And I click "Exportieren"
    And I click "JSON"
    Then the backend received "save_export_to_file" with:
      """
      { "format": "json", "request": { "period": "week" } }
      """
    And I see "Export gespeichert: /tmp/nunc-export.json"

  @ui @REQ-10
  Scenario: The number of pomodoros in the period is shown, not the lifetime total
    Given the statistics contain daily data:
      | daysAgo | pomodoros | minutes |
      | 0       | 4         | 100     |
      | 1       | 2         | 50      |
      | 3       | 6         | 150     |
    And the statistics count 250 pomodoros over all time
    When I open the app at "/stats"
    Then the statistic "Pomodoros" is "12"
    And the statistic "Ø Pomodoros / Tag" is "1.7"
    And the statistic "Gesamt" is "250 Pomodoros"
    When I click "Tag"
    Then the statistic "Pomodoros" is "4"

  @ui @REQ-10
  Scenario: The pomodoros of each day are labelled in the line chart
    Given the statistics contain daily data:
      | daysAgo | pomodoros | minutes |
      | 0       | 4         | 100     |
      | 2       | 6         | 150     |
    When I open the app at "/stats"
    Then the line chart labels the days with "0, 6, 0, 4"

  @ui @REQ-10
  Scenario: Pomodoros per month are listed for month and year
    Given the statistics contain daily data:
      | daysAgo | pomodoros | minutes |
      | 0       | 4         | 100     |
    And the statistics contain monthly data:
      | year | month | pomodoros | minutes |
      | 2026 | 9     | 30        | 750     |
      | 2026 | 10    | 12        | 300     |
    When I open the app at "/stats"
    Then I do not see the monthly table
    When I click "Jahr"
    Then the month "September 2026" shows 30 pomodoros
    And the month "Oktober 2026" shows 12 pomodoros

