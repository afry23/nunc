Feature: Habit schedule, goal and name
  Source: ADR 0028
  A habit is due daily, on chosen weekdays or a number of days per week. Its goal is ticking it off by hand or
  reaching a number of minutes on a project or context. Both are stored as text and read back only in their
  canonical form.

  @domain @US-HB01
  Scenario Outline: A schedule is read back from its text
    Then the schedule text "<text>" reads as "<result>"

    Examples:
      | text        | result      |
      | daily       | daily       |
      | days:1      | days:1      |
      | days:1,3,5  | days:1,3,5  |
      | days:1,2,3,4,5,6,7 | days:1,2,3,4,5,6,7 |
      | weekly:1    | weekly:1    |
      | weekly:7    | weekly:7    |
      |             | invalid     |
      | Daily       | invalid     |
      | daily:1     | invalid     |
      | days:       | invalid     |
      | days:0      | invalid     |
      | days:8      | invalid     |
      | days:3,1    | invalid     |
      | days:1,1    | invalid     |
      | days:1,,3   | invalid     |
      | days:x      | invalid     |
      | weekly:0    | invalid     |
      | weekly:8    | invalid     |
      | weekly:03   | invalid     |
      | weekly:     | invalid     |
      | weekly:-1   | invalid     |
      | monthly:1   | invalid     |

  @domain @US-HB01
  Scenario Outline: Weekdays are ordered and checked
    Then the weekdays "<days>" are checked as "<result>"

    Examples:
      | days  | result     |
      | 5,1   | days:1,5   |
      | 3,3,3 | days:3     |
      | 1,2,3,4,5,6,7 | days:1,2,3,4,5,6,7 |
      |       | invalid    |
      | 0     | invalid    |
      | 8     | invalid    |
      | 1,8   | invalid    |

  @domain @US-HB01
  Scenario Outline: A weekly target is between one and seven days
    Then the weekly target <count> is checked as "<result>"

    Examples:
      | count | result   |
      | 0     | invalid  |
      | 1     | weekly:1 |
      | 7     | weekly:7 |
      | 8     | invalid  |

  @domain @US-HB01
  Scenario Outline: A goal is read back from its text
    Then the goal text "<text>" reads as "<result>"

    Examples:
      | text                 | result               |
      | check                | check                |
      | minutes:30:+work     | minutes:30:+work     |
      | minutes:1:@desk      | minutes:1:@desk      |
      | minutes:1440:+a:b    | minutes:1440:+a:b    |
      |                      | invalid              |
      | Check                | invalid              |
      | minutes              | invalid              |
      | minutes:30           | invalid              |
      | minutes:0:+work      | invalid              |
      | minutes:1441:+work   | invalid              |
      | minutes:030:+work    | invalid              |
      | minutes:x:+work      | invalid              |
      | minutes:30:work      | invalid              |
      | minutes:30:+         | invalid              |
      | minutes:30:+a b      | invalid              |
      | minutes:30: +work    | invalid              |
      | hours:1:+work        | invalid              |

  @domain @US-HB01
  Scenario Outline: A minutes goal is checked
    Then the minutes goal <minutes> on "<tag>" is checked as "<result>"

    Examples:
      | minutes | tag     | result           |
      | 30      | +work   | minutes:30:+work |
      | 30      |  @desk  | minutes:30:@desk |
      | 0       | +work   | invalid          |
      | 1441    | +work   | invalid          |
      | 1       | work    | invalid          |
      | 1       | +       | invalid          |
      | 1       | @a b    | invalid          |
      | 1440    | +x      | minutes:1440:+x  |

  @domain @US-HB01
  Scenario Outline: The name is trimmed and limited
    Then the name of <length> characters padded with <pad> spaces is <result>

    Examples:
      | length | pad | result   |
      | 0      | 3   | rejected |
      | 1      | 2   | accepted |
      | 100    | 0   | accepted |
      | 101    | 0   | rejected |
      | 100    | 5   | accepted |

  @domain @US-HB01
  Scenario: A name is stored without surrounding spaces
    Then the name "  Read  " is stored as "Read"
