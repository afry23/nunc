Feature: Session export
  Source: REQUIREMENTS 11; nunc-go stats_test.go
  Dates and times are local; the CSV has the Go columns plus session_type at the end.

  Background:
    Given the statistics range is 2025-06-01 to 2025-06-30
    And the todos:
      | id | line                                | pomodoros | position |
      | 1  | (A) Write report +work +q4 @desk    | 0         | 1        |
      | 2  | Say "hello", then call @phone       | 0         | 2        |
      | 3  | -rm everything +danger              | 0         | 3        |
    And the sessions:
      | start            | end              | mode     | type        | status    | minutes | todo |
      | 2025-06-03 08:00 |                  | flowtime | work        | completed |         | 3    |
      | 2025-06-02 14:00 | 2025-06-02 14:12 | pomodoro | work        | aborted   | 12      | 2    |
      | 2025-06-01 09:30 | 2025-06-01 09:35 | pomodoro | short_break | completed | 5       |      |
      | 2025-06-01 09:00 | 2025-06-01 09:25 | pomodoro | work        | completed | 25      | 1    |

  @domain @REQ-11
  Scenario: Export all sessions as CSV with the specified columns
    When I export the sessions
    Then the CSV export is:
      """
      date,start_time,end_time,duration_min,mode,status,todo_text,project_tags,context_tags,session_type
      2025-06-01,09:00:00,09:25:00,25,pomodoro,completed,(A) Write report +work +q4 @desk,'+work +q4,'@desk,work
      2025-06-01,09:30:00,09:35:00,5,pomodoro,completed,,,,short_break
      2025-06-02,14:00:00,14:12:00,12,pomodoro,aborted,"Say ""hello"", then call @phone",,'@phone,work
      2025-06-03,08:00:00,,,flowtime,completed,'-rm everything +danger,'+danger,,work
      """

  @domain @REQ-11
  Scenario: Export all sessions as JSON
    Given the statistics range is 2025-06-01 to 2025-06-01
    When I export the sessions
    Then the JSON export is:
      """
      [
        {
          "date": "2025-06-01",
          "start_time": "09:00:00",
          "end_time": "09:25:00",
          "duration_min": 25,
          "mode": "pomodoro",
          "status": "completed",
          "todo_text": "(A) Write report +work +q4 @desk",
          "project_tags": ["+work", "+q4"],
          "context_tags": ["@desk"],
          "session_type": "work"
        },
        {
          "date": "2025-06-01",
          "start_time": "09:30:00",
          "end_time": "09:35:00",
          "duration_min": 5,
          "mode": "pomodoro",
          "status": "completed",
          "todo_text": "",
          "project_tags": [],
          "context_tags": [],
          "session_type": "short_break"
        }
      ]
      """
    And the JSON export is indented with 2 spaces

  @domain @REQ-11
  Scenario: A running session is exported without end time and duration
    Given the statistics range is 2025-06-03 to 2025-06-03
    When I export the sessions
    Then the JSON export is:
      """
      [
        {
          "date": "2025-06-03",
          "start_time": "08:00:00",
          "end_time": null,
          "duration_min": null,
          "mode": "flowtime",
          "status": "completed",
          "todo_text": "-rm everything +danger",
          "project_tags": ["+danger"],
          "context_tags": [],
          "session_type": "work"
        }
      ]
      """

  @domain @REQ-11
  Scenario: Export an empty CSV contains only the header
    Given the statistics range is 2025-07-01 to 2025-07-31
    When I export the sessions
    Then the CSV export has 1 lines
    And the CSV export is:
      """
      date,start_time,end_time,duration_min,mode,status,todo_text,project_tags,context_tags,session_type
      """

  @domain @REQ-11
  Scenario: Export an empty JSON is an empty list
    Given the statistics range is 2025-07-01 to 2025-07-31
    When I export the sessions
    Then the JSON export is an empty list

  @domain @REQ-11
  Scenario: Export a filtered date range
    Given the statistics range is 2025-06-02 to 2025-06-02
    When I export the sessions
    Then the CSV export has 2 lines

  @domain @REQ-11
  Scenario: The range includes both boundary days
    Given the statistics range is 2025-06-01 to 2025-06-03
    When I export the sessions
    Then the CSV export has 5 lines

  @domain @REQ-11
  Scenario: CSV values with commas and quotes are escaped
    Given the statistics range is 2025-06-02 to 2025-06-02
    When I export the sessions
    Then the CSV export is:
      """
      date,start_time,end_time,duration_min,mode,status,todo_text,project_tags,context_tags,session_type
      2025-06-02,14:00:00,14:12:00,12,pomodoro,aborted,"Say ""hello"", then call @phone",,'@phone,work
      """

  @domain @REQ-11
  Scenario: Text that looks like a formula is neutralized
    Given the statistics range is 2025-06-03 to 2025-06-03
    When I export the sessions
    Then the CSV export is:
      """
      date,start_time,end_time,duration_min,mode,status,todo_text,project_tags,context_tags,session_type
      2025-06-03,08:00:00,,,flowtime,completed,'-rm everything +danger,'+danger,,work
      """

  @domain @REQ-11
  Scenario: Date and times are local
    Given the sessions:
      | start            | end              | mode     | type | status    | minutes | todo |
      | 2025-06-01 23:30 | 2025-06-01 23:55 | pomodoro | work | completed | 25      |      |
    And the local time zone is UTC+2
    And the statistics range is 2025-06-02 to 2025-06-02
    When I export the sessions
    Then the CSV export is:
      """
      date,start_time,end_time,duration_min,mode,status,todo_text,project_tags,context_tags,session_type
      2025-06-02,01:30:00,01:55:00,25,pomodoro,completed,,,,work
      """

  @domain @property @REQ-11
  Scenario: The CSV export can be read back for any text
    Then the property "the CSV export can be read back for any text" holds for 200 random cases

  @ui @REQ-11 @mobile @wip
  Scenario: The export is handed to the share sheet
