Feature: Local HTTP interface
  Source: nunc extension (not in the Go app). Another program on the same desktop exchanges todo.txt lines with
  nunc and reads the status of tasks. The server listens on 127.0.0.1 only and needs a bearer token.

  Background:
    Given the API token is "secret-token" and the port is 42820
    And the app has the todos:
      | id | line                         | pomodoros | position |
      | 1  | (A) Write report +work @desk | 3         | 1        |
      | 2  | Call dentist @phone          | 0         | 2        |
      | 3  | x 2025-03-10 Plan trip +home | 1         | 3        |

  # --- Zugriffsschutz ---------------------------------------------------------------------------------------

  @domain @api @REQ-API
  Scenario: A request without a token is rejected
    Given the request has no Authorization header
    When I send "GET /v1/todos"
    Then the response status is 401
    And the error code is "unauthorized"
    And the response has the header "WWW-Authenticate" with "Bearer"

  @domain @api @REQ-API
  Scenario Outline: A wrong token is rejected
    Given the request header "Authorization" is "<header>"
    When I send "GET /v1/todos"
    Then the response status is 401
    And the error code is "unauthorized"

    Examples:
      | header                  |
      | Bearer wrong-token      |
      | Bearer secret-toke      |
      | Bearer secret-token-x   |
      | Bearer                  |
      | Basic secret-token      |
      | secret-token            |

  @domain @api @REQ-API
  Scenario: The authorization scheme is not case sensitive
    Given the request header "Authorization" is "bearer secret-token"
    When I send "GET /v1/health"
    Then the response status is 200

  @domain @api @REQ-API
  Scenario: The token does not leak into the error
    Given the request header "Authorization" is "Bearer wrong-token"
    When I send "GET /v1/todos"
    Then the response body does not contain "secret-token"
    And the response body does not contain "wrong-token"

  @domain @api @REQ-API
  Scenario: A browser request is rejected even with the right token
    Given the request header "Origin" is "https://evil.example"
    When I send "GET /v1/todos"
    Then the response status is 403
    And the error code is "forbidden_origin"

  @domain @api @REQ-API
  Scenario Outline: Only the loopback host name of the configured port is accepted
    Given the request header "Host" is "<host>"
    When I send "GET /v1/health"
    Then the response status is <status>

    Examples:
      | host                | status |
      | 127.0.0.1:42820     | 200    |
      | localhost:42820     | 200    |
      | LOCALHOST:42820     | 200    |
      | 127.0.0.1:42821     | 403    |
      | evil.example:42820  | 403    |
      | evil.example        | 403    |
      | 127.0.0.1.evil.test:42820 | 403 |

  @domain @api @REQ-API
  Scenario: A request without a Host header is rejected
    Given the request has no Host header
    When I send "GET /v1/health"
    Then the response status is 403
    And the error code is "forbidden_host"

  @domain @api @REQ-API
  Scenario: The health check reports the app and the API version
    When I send "GET /v1/health"
    Then the response status is 200
    And the response JSON has "app" equal to "nunc"
    And the response JSON has "apiVersion" equal to 1

  # --- Lesen ------------------------------------------------------------------------------------------------

  @domain @api @REQ-API
  Scenario: The todos are returned as a todo.txt file by default
    When I send "GET /v1/todos"
    Then the response status is 200
    And the response content type is "text/plain; charset=utf-8"
    And the response body is:
      """
      (A) Write report +work @desk
      Call dentist @phone
      x 2025-03-10 Plan trip +home
      """

  @domain @api @REQ-API
  Scenario: The todos can be requested as JSON with their status
    Given the request header "Accept" is "application/json"
    When I send "GET /v1/todos"
    Then the response status is 200
    And the response content type is "application/json"
    And the response todos are:
      | id | rawText                      | completed | pomodoroCount | priority |
      | 1  | (A) Write report +work @desk | false     | 3             | A        |
      | 2  | Call dentist @phone          | false     | 0             |          |
      | 3  | x 2025-03-10 Plan trip +home | true      | 1             |          |

  @domain @api @REQ-API
  Scenario Outline: The list can be filtered
    Given the request header "Accept" is "application/json"
    When I send "GET /v1/todos?<query>"
    Then the response status is 200
    And the response todo ids are "<ids>"

    Examples:
      | query                       | ids   |
      | completed=false             | 1 2   |
      | completed=true              | 3     |
      | project=work                | 1     |
      | project=%2Bwork             | 1     |
      | context=phone               | 2     |
      | context=%40desk             | 1     |
      | priority=A                  | 1     |
      | q=dentist                   | 2     |
      | completed=false&context=phone | 2   |
      | project=home&completed=false  |     |

  @domain @api @REQ-API
  Scenario: An invalid filter value is a bad request
    When I send "GET /v1/todos?completed=maybe"
    Then the response status is 400
    And the error code is "invalid_input"

  @domain @api @REQ-API
  Scenario: A single task returns its status
    When I send "GET /v1/todos/3"
    Then the response status is 200
    And the response JSON has "id" equal to 3
    And the response JSON has "completed" equal to true
    And the response JSON has "pomodoroCount" equal to 1

  @domain @api @REQ-API
  Scenario Outline: An unknown or malformed task id is not found
    When I send "GET /v1/todos/<id>"
    Then the response status is <status>
    And the error code is "<code>"

    Examples:
      | id   | status | code          |
      | 99   | 404    | not_found     |
      | abc  | 400    | invalid_input |

  # --- Schreiben --------------------------------------------------------------------------------------------

  @domain @api @REQ-API
  Scenario: Posted todo.txt lines are appended, duplicates are skipped
    When I send "POST /v1/todos" with the body:
      """
      Buy milk @shop
      Call dentist @phone

      (B) Water plants
      Buy milk @shop
      """
    Then the response status is 201
    And the response JSON has "added" equal to 2
    And the response JSON has "duplicates" equal to 2
    And the response todos are:
      | id | rawText        |
      | 4  | Buy milk @shop |
      | 5  | (B) Water plants |
    And the app's todo lines are:
      """
      (A) Write report +work @desk
      Call dentist @phone
      x 2025-03-10 Plan trip +home
      Buy milk @shop
      (B) Water plants
      """

  @domain @api @REQ-API
  Scenario: Posting only known lines adds nothing
    When I send "POST /v1/todos" with the body:
      """
      Call dentist @phone
      """
    Then the response status is 200
    And the response JSON has "added" equal to 0
    And the response JSON has "duplicates" equal to 1

  @domain @api @REQ-API
  Scenario: A single todo can be posted as JSON
    Given the request header "Content-Type" is "application/json"
    When I send "POST /v1/todos" with the body:
      """
      { "rawText": "Buy milk @shop" }
      """
    Then the response status is 201
    And the response JSON has "added" equal to 1

  @domain @api @REQ-API
  Scenario Outline: A malformed body is a bad request
    Given the request header "Content-Type" is "<type>"
    When I send "POST /v1/todos" with the body:
      """
      <body>
      """
    Then the response status is 400
    And the error code is "invalid_input"

    Examples:
      | type             | body                |
      | application/json | not json            |
      | application/json | {}                  |
      | application/json | { "rawText": 5 }    |

  @domain @api @REQ-API
  Scenario: A body that is not UTF-8 is a bad request
    When I send "POST /v1/todos" with the invalid UTF-8 bytes
    Then the response status is 400
    And the error code is "invalid_input"

  @domain @api @REQ-API
  Scenario: A task can be completed and reopened
    Given the request header "Content-Type" is "application/json"
    When I send "PATCH /v1/todos/2" with the body:
      """
      { "completed": true }
      """
    Then the response status is 200
    And the response JSON has "completed" equal to true
    When I send "PATCH /v1/todos/2" with the body:
      """
      { "completed": false }
      """
    Then the response status is 200
    And the response JSON has "completed" equal to false

  @domain @api @REQ-API
  Scenario Outline: Completing needs a known task and a valid body
    Given the request header "Content-Type" is "application/json"
    When I send "PATCH /v1/todos/<id>" with the body:
      """
      <body>
      """
    Then the response status is <status>
    And the error code is "<code>"

    Examples:
      | id | body                  | status | code          |
      | 99 | { "completed": true } | 404    | not_found     |
      | 2  | { "completed": "yes" }| 400    | invalid_input |
      | 2  | {}                    | 400    | invalid_input |

  # --- Rahmen -----------------------------------------------------------------------------------------------

  @domain @api @REQ-API
  Scenario Outline: Unsupported routes and methods are rejected
    When I send "<request>"
    Then the response status is <status>
    And the error code is "<code>"

    Examples:
      | request                | status | code               |
      | GET /v2/todos          | 404    | not_found          |
      | GET /                  | 404    | not_found          |
      | DELETE /v1/todos/1     | 405    | method_not_allowed |
      | PUT /v1/todos          | 405    | method_not_allowed |
      | POST /v1/health        | 405    | method_not_allowed |
      | PATCH /v1/todos        | 405    | method_not_allowed |

  @domain @api @REQ-API
  Scenario: A failing backend is reported without details
    Given the app fails with "disk on fire"
    When I send "GET /v1/todos"
    Then the response status is 500
    And the error code is "internal"
    And the response body does not contain "disk on fire"

  @domain @api @REQ-API
  Scenario: The error answers are JSON with a code and a message
    Given the request has no Authorization header
    When I send "GET /v1/todos"
    Then the response content type is "application/json"
    And the response JSON has a non-empty "message"

  # --- Server (echter Socket) -------------------------------------------------------------------------------

  @integration @api @REQ-API
  Scenario: The server answers over a real connection and only on the loopback address
    Given the API server is running with the token "secret-token"
    When I request "/v1/todos" over HTTP with the token
    Then the HTTP status is 200
    And the server is bound to 127.0.0.1

  @integration @api @REQ-API
  Scenario: The server rejects a request without a token over a real connection
    Given the API server is running with the token "secret-token"
    When I request "/v1/todos" over HTTP without a token
    Then the HTTP status is 401

  @integration @api @REQ-API
  Scenario: A request body above the limit is refused
    Given the API server is running with the token "secret-token"
    When I post 6 MB to "/v1/todos" over HTTP with the token
    Then the HTTP status is 413

  @integration @api @REQ-API
  Scenario Outline: Bodies up to the limit are accepted over a real connection
    Given the API server is running with the token "secret-token"
    When I post <size> MB to "/v1/todos" over HTTP with the token
    Then the HTTP status is 201

    Examples:
      | size |
      | 1    |
      | 5    |

  @integration @api @REQ-API
  Scenario: When the port is taken the server takes the next free one
    Given another program holds the port
    When the API server is started on that port
    Then the server is running on a different port

  @integration @api @REQ-API
  Scenario: Stopping the server closes the port
    Given the API server is running with the token "secret-token"
    When the API server is stopped
    Then connecting to the port fails
