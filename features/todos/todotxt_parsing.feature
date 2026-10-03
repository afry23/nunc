Feature: todo.txt parsing and serialization
  Source: REQUIREMENTS 4.1; nunc-go todos_parse_test.go

  @domain @REQ-4.1 @wip
  Scenario: Parse a simple description

  @domain @REQ-4.1 @wip
  Scenario: Parse a priority

  @domain @REQ-4.1 @wip
  Scenario: Parse priority with project and context

  @domain @REQ-4.1 @wip
  Scenario: Parse a creation date

  @domain @REQ-4.1 @wip
  Scenario: Parse priority and creation date

  @domain @REQ-4.1 @wip
  Scenario: Parse a completed todo with completion and creation date

  @domain @REQ-4.1 @wip
  Scenario: Parse a completed todo with completion date only

  @domain @REQ-4.1 @wip
  Scenario: Parse a completed todo without any date

  @domain @REQ-4.1 @wip
  Scenario: Parse multiple projects and contexts

  @domain @REQ-4.1 @wip
  Scenario: A lowercase x is not a completion marker

  @domain @REQ-4.1 @wip
  Scenario: Only uppercase letters A to Z are priorities

  @domain @REQ-4.1 @wip
  Scenario: The description keeps the project and context tags

  @domain @REQ-4.1 @wip
  Scenario: Build the raw text from the structured fields

  @domain @REQ-4.1 @wip
  Scenario: Serializing a parsed todo gives back the original line
