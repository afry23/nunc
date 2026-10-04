Feature: todo.txt parsing and serialization
  Source: REQUIREMENTS 4.1; nunc-go todos_parse_test.go
  The description keeps the tags exactly as in the Go app; the tag lists keep their prefix.

  @domain @REQ-4.1
  Scenario: Parse a simple description
    When I parse the line "Buy milk"
    Then the todo is not completed
    And the priority is none
    And the creation date is none
    And the completion date is none
    And the description is "Buy milk"
    And the project tags are ""
    And the context tags are ""

  @domain @REQ-4.1
  Scenario: Parse a priority
    When I parse the line "(A) Buy milk"
    Then the priority is A
    And the description is "Buy milk"

  @domain @REQ-4.1
  Scenario: Parse priority with project and context
    When I parse the line "(B) Write report +work @office"
    Then the priority is B
    And the description is "Write report +work @office"
    And the project tags are "+work"
    And the context tags are "@office"

  @domain @REQ-4.1
  Scenario: Parse a creation date
    When I parse the line "2025-01-10 Plain task"
    Then the creation date is 2025-01-10
    And the priority is none
    And the description is "Plain task"

  @domain @REQ-4.1
  Scenario: Parse priority and creation date
    When I parse the line "(C) 2025-01-10 Do the thing +project"
    Then the priority is C
    And the creation date is 2025-01-10
    And the description is "Do the thing +project"
    And the project tags are "+project"

  @domain @REQ-4.1
  Scenario: Parse a completed todo with completion and creation date
    When I parse the line "x 2025-01-15 2025-01-10 Write report +work"
    Then the todo is completed
    And the completion date is 2025-01-15
    And the creation date is 2025-01-10
    And the description is "Write report +work"
    And the project tags are "+work"

  @domain @REQ-4.1
  Scenario: Parse a completed todo with completion date only
    When I parse the line "x 2025-01-15 Call dentist @phone"
    Then the todo is completed
    And the completion date is 2025-01-15
    And the creation date is none
    And the description is "Call dentist @phone"
    And the context tags are "@phone"

  @domain @REQ-4.1
  Scenario: Parse a completed todo without any date
    When I parse the line "x Call dentist"
    Then the todo is completed
    And the completion date is none
    And the creation date is none
    And the description is "Call dentist"

  @domain @REQ-4.1
  Scenario: Parse multiple projects and contexts
    When I parse the line "Plan trip +holiday +family @home @phone +holiday"
    Then the project tags are "+holiday +family +holiday"
    And the context tags are "@home @phone"
    And the description is "Plan trip +holiday +family @home @phone +holiday"

  @domain @REQ-4.1
  Scenario Outline: A lowercase x is not a completion marker
    When I parse the line "<line>"
    Then the todo is not completed
    And the description is "<description>"

    Examples:
      | line                   | description        |
      | X 2025-01-15 Task      | X 2025-01-15 Task  |
      | xylophone lesson       | xylophone lesson   |
      | x2025-01-15 Task       | x2025-01-15 Task   |

  @domain @REQ-4.1
  Scenario Outline: Only uppercase letters A to Z are priorities
    When I parse the line "<line>"
    Then the priority is <priority>
    And the description is "<description>"

    Examples:
      | line          | priority | description   |
      | (A) Task      | A        | Task          |
      | (Z) Task      | Z        | Task          |
      | (a) Task      | none     | (a) Task      |
      | (1) Task      | none     | (1) Task      |
      | (AB) Task     | none     | (AB) Task     |
      | (A)Task       | none     | (A)Task       |
      | [A] Task      | none     | [A] Task      |
      | Task (A) now  | none     | Task (A) now  |

  @domain @REQ-4.1
  Scenario: A completed todo has no priority
    When I parse the line "x 2025-01-15 (A) Task"
    Then the priority is none
    And the description is "(A) Task"

  @domain @REQ-4.1
  Scenario Outline: A date only counts when it is valid and followed by a space
    When I parse the line "<line>"
    Then the creation date is <created>
    And the description is "<description>"

    Examples:
      | line               | created    | description        |
      | 2025-01-10 Task    | 2025-01-10 | Task               |
      | 2024-02-29 Leap    | 2024-02-29 | Leap               |
      | 2025-02-29 Leap    | none       | 2025-02-29 Leap    |
      | 2025-13-01 Month   | none       | 2025-13-01 Month   |
      | 2025-1-10 Short    | none       | 2025-1-10 Short    |
      | 2025-01-10Task     | none       | 2025-01-10Task     |
      | Task 2025-01-10    | none       | Task 2025-01-10    |
      | 2025/01-10 Task    | none       | 2025/01-10 Task    |
      | 2025-01/10 Task    | none       | 2025-01/10 Task    |
      | 20x5-01-10 Task    | none       | 20x5-01-10 Task    |
      | 2025-0x-10 Task    | none       | 2025-0x-10 Task    |
      | 2025-01-1x Task    | none       | 2025-01-1x Task    |
      | 2025-01-0: Task    | none       | 2025-01-0: Task    |
      | 2025-0:-10 Task    | none       | 2025-0:-10 Task    |
      | 202:-01-10 Task    | none       | 202:-01-10 Task    |

  @domain @REQ-4.1
  Scenario: The description keeps the project and context tags
    When I parse the line "(A) 2025-01-10 Write +work report @desk"
    Then the description is "Write +work report @desk"
    And the project tags are "+work"
    And the context tags are "@desk"

  @domain @REQ-4.1
  Scenario Outline: A prefix character alone is not a tag
    When I parse the line "<line>"
    Then the project tags are "<projects>"
    And the context tags are "<contexts>"

    Examples:
      | line                | projects | contexts |
      | Buy + milk @ home   |          |          |
      | Buy +milk @ home    | +milk    |          |
      | a+b c@d             |          |          |

  @domain @REQ-4.1
  Scenario Outline: Build the raw text from the structured fields
    Given a todo with completed <completed>, priority "<priority>", completion date "<done>", creation date "<created>" and description "<description>"
    When I build the raw text
    Then the raw text is "<raw>"

    Examples:
      | completed | priority | done       | created    | description        | raw                                      |
      | false     |          |            |            | Buy milk           | Buy milk                                 |
      | false     | A        |            |            | Buy milk           | (A) Buy milk                             |
      | false     |          |            | 2025-01-10 | Plain task         | 2025-01-10 Plain task                    |
      | false     | C        |            | 2025-01-10 | Do it +project     | (C) 2025-01-10 Do it +project            |
      | true      |          | 2025-01-15 | 2025-01-10 | Write report +work | x 2025-01-15 2025-01-10 Write report +work |
      | true      |          | 2025-01-15 |            | Call dentist       | x 2025-01-15 Call dentist                |
      | true      |          |            |            | Call dentist       | x Call dentist                           |
      | true      | A        | 2025-01-15 |            | Call dentist       | x 2025-01-15 Call dentist                |

  @domain @REQ-4.1
  Scenario Outline: Serializing a parsed todo gives back the original line
    When I parse the line "<line>" and build the raw text again
    Then the raw text is "<line>"

    Examples:
      | line                                        |
      | Buy milk                                    |
      | (A) Buy milk                                |
      | 2025-01-10 Plain task                       |
      | (C) 2025-01-10 Do the thing +project        |
      | x 2025-01-15 2025-01-10 Write report +work  |
      | x 2025-01-15 Call dentist @phone            |
      | x Call dentist                              |
      | Plan  trip   with  spaces +a @b             |

  @domain @property @REQ-4.1
  Scenario: Any well-formed todo survives serializing and parsing
    Then the property "any well-formed todo survives serializing and parsing" holds for 300 random cases

  @domain @property @REQ-4.1
  Scenario: Parsing and serializing is stable for any text
    Then the property "parsing and serializing is stable for any text" holds for 300 random cases
