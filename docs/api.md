# Local API

nunc can run a small HTTP server so other programs on the same computer (scripts, launchers, editor plugins, other
todo.txt tools) can exchange tasks in todo.txt format and query their status.

- **Desktop only** (macOS, Windows, Linux). There is no server on Android or iOS.
- **Off by default.** Enable it in *Settings → Local interface*.
- Listens on `127.0.0.1` only, never on other interfaces.
- Current version: `v1`.

Design decisions and the threat model: [ADR 0013](./adr/0013-lokale-api.md).

## Setup

1. Open *Settings → Local interface*, switch the interface on and save.
2. Click "Show token" and copy the token. "Regenerate" invalidates the old token and restarts the server.
3. Note the port. The default is `42820`. If it is taken, nunc uses the next free port (up to `42829`); the settings
   show the actual port.

The token lives in the system keychain (service `nunc`, account `local-api`). On Linux without a Secret Service the
token cannot be stored and the server does not start; the settings show the reason.

## Requests

Every request needs:

| Header | Value |
|---|---|
| `Authorization` | `Bearer <token>` |
| `Host` | `127.0.0.1:<port>` or `localhost:<port>` (default for any normal HTTP client) |

Requests that carry an `Origin` header are rejected. Browsers add it to calls from web pages, so web pages cannot use
the API. There is no CORS.

Limits: body up to 5 MB, 10 s to send the headers, 30 s per connection, at most 32 concurrent connections, one request
per connection.

## Endpoints

| Request | Description |
|---|---|
| `GET /v1/health` | App name and version |
| `GET /v1/todos` | List active tasks |
| `GET /v1/todos/{id}` | One task with status |
| `POST /v1/todos` | Add new tasks |
| `PATCH /v1/todos/{id}` | Mark a task done or open |

### `GET /v1/health`

```json
{ "app": "nunc", "version": "0.2.0", "apiVersion": 1 }
```

### `GET /v1/todos`

Returns all active tasks in manual order.

- Default: `text/plain`, one todo.txt line per task.
- With `Accept: application/json`: a JSON array of task objects (see below).

Query parameters (all optional, combinable):

| Parameter | Meaning |
|---|---|
| `completed` | `true` or `false` |
| `project` | project tag; `work` and `%2Bwork` both match `+work` (repeatable) |
| `context` | context tag; `desk` and `%40desk` both match `@desk` (repeatable) |
| `priority` | one letter `A`–`Z` (repeatable) |
| `q` | text search |

Unknown parameters are ignored. Invalid values give `400 invalid_input`.

### `GET /v1/todos/{id}`

Returns one task as a JSON object. `404 not_found` if the id does not exist, `400 invalid_input` if it is not a number.

### `POST /v1/todos`

Adds only new lines. Duplicates and blank lines are skipped. The body is either

- `text/plain` (any content type other than JSON): one or more todo.txt lines, UTF-8, or
- `application/json`: `{ "rawText": "line 1\nline 2" }`.

Response: `201` if at least one task was added, otherwise `200`.

```json
{ "added": 1, "duplicates": 1, "todos": [ { "id": 12, "rawText": "(A) Write report +work" } ] }
```

`todos` holds the added tasks as full task objects (shortened above).

### `PATCH /v1/todos/{id}`

Body `application/json`:

```json
{ "completed": true }
```

Returns the updated task object. Other fields cannot be changed.

### Task object

```json
{
  "id": 4,
  "rawText": "x 2026-09-25 2026-09-18 Plan sprint +work",
  "priority": null,
  "completed": true,
  "completedAt": "2026-09-25",
  "createdAt": "2026-09-18",
  "description": "Plan sprint +work",
  "projectTags": ["+work"],
  "contextTags": [],
  "pomodoroCount": 3,
  "position": 4
}
```

| Field | Meaning |
|---|---|
| `id` | task id |
| `rawText` | the full todo.txt line |
| `priority` | `"A"`–`"Z"` or `null` |
| `completed`, `completedAt` | status; date as `YYYY-MM-DD`, `null` if open |
| `createdAt` | creation date or `null` |
| `description` | text including the tags |
| `projectTags`, `contextTags` | tags with prefix (`+work`, `@desk`) |
| `pomodoroCount` | completed pomodoros for this task |
| `position` | manual sort position |

The plain-text export has no pomodoro counter, so `pomodoroCount` is only available as JSON.

## Errors

Errors are JSON `{ "code": "…", "message": "…" }` (message in English).

| Status | `code` | Cause |
|---|---|---|
| 400 | `invalid_input` | bad id, query value, JSON or encoding |
| 401 | `unauthorized` | token missing or wrong (`WWW-Authenticate: Bearer`) |
| 403 | `forbidden_host` | `Host` header is not `127.0.0.1:<port>` or `localhost:<port>` |
| 403 | `forbidden_origin` | request has an `Origin` header |
| 404 | `not_found` | unknown path or task |
| 405 | `method_not_allowed` | wrong method (`Allow` header lists the right ones) |
| 413 | `payload_too_large` | body over 5 MB |
| 500 | `internal` | unexpected error; details are in `nunc.log` |

Checks run in this order: host → origin → token → size → path and method → content.

## Examples

```bash
T=…                                  # token from Settings
B=http://127.0.0.1:42820

curl -H "Authorization: Bearer $T" $B/v1/health

# todo.txt
curl -H "Authorization: Bearer $T" $B/v1/todos

# JSON, open tasks of project "work"
curl -H "Authorization: Bearer $T" -H "Accept: application/json" \
     "$B/v1/todos?completed=false&project=work"

# one task
curl -H "Authorization: Bearer $T" $B/v1/todos/3

# add tasks from a file (new lines only)
curl -X POST -H "Authorization: Bearer $T" --data-binary @todo.txt $B/v1/todos

# add a task as JSON
curl -X POST -H "Authorization: Bearer $T" -H "Content-Type: application/json" \
     -d '{"rawText":"(A) Write report +work @desk"}' $B/v1/todos

# mark done
curl -X PATCH -H "Authorization: Bearer $T" -H "Content-Type: application/json" \
     -d '{"completed":true}' $B/v1/todos/3
```

## Behavior notes

- Changes made through the API reload the UI and trigger a sync, just like changes made in the app.
- The API only exposes tasks. Sessions, settings, backups and accounts are not reachable.
- Traffic is unencrypted (loopback only). Anyone who knows the token has full access to the tasks.
- Not part of v1: timer state, statistics, change notifications (SSE/webhook).
