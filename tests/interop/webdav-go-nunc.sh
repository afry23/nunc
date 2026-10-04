#!/usr/bin/env bash
# Interop-Check (manuell, Szenario "Go app and nunc sync alternately against a WebDAV server"):
# Die Go-App (echte WebDAVSyncService-Logik) und nunc (nunc-sync) synchronisieren abwechselnd gegen denselben
# WebDAV-Server; danach müssen beide Datenbanken dieselben Todos und Sessions haben.
#
# Voraussetzung: Go, sqlite3, python3 und ../nunc-go. Der Server ist `golang.org/x/net/webdav` (Basic-Auth,
# Ordner im Temp-Verzeichnis) und braucht kein Docker. Gegen einen eigenen Server (z. B. `rclone serve webdav`):
#   WEBDAV_URL=http://localhost:8080 WEBDAV_USER=me WEBDAV_PASSWORD=secret tests/interop/webdav-go-nunc.sh
# Ausgabe: je Schritt eine Zeile; Exit-Code != 0 bei der ersten Abweichung.
set -euo pipefail
cd "$(dirname "$0")/../.."

tmp=$(mktemp -d)
server_pid=""
cleanup() {
  if [ -n "$server_pid" ]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
  rm -rf "$tmp"
}
trap cleanup EXIT

user=${WEBDAV_USER:-me}
password=${WEBDAV_PASSWORD:-secret}
remote=/interop/
fail() { echo "FEHLER: $*" >&2; exit 1; }
count() { sqlite3 "$1" "$2"; }
expect() { [ "$2" = "$3" ] || fail "$1: erwartet $3, erhalten $2"; echo "ok: $1 = $2"; }

# Go-Helfer bauen (Server und Go-Sync)
cp -R ../nunc-go/internal ../nunc-go/go.mod ../nunc-go/go.sum "$tmp/"
mkdir -p "$tmp/cmd/interop" && cp tests/interop/go_webdav.go.txt "$tmp/cmd/interop/main.go"
(cd "$tmp" && go build -o "$tmp/interop" ./cmd/interop)

if [ -z "${WEBDAV_URL:-}" ]; then
  port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1]); s.close()')
  mkdir -p "$tmp/dav"
  "$tmp/interop" serve "$tmp/dav" "$port" "$user" "$password" &
  server_pid=$!
  WEBDAV_URL="http://127.0.0.1:$port"
  for _ in $(seq 50); do curl -s -o /dev/null "$WEBDAV_URL/" && break; sleep 0.1; done
fi

go_sync() { "$tmp/interop" sync "$tmp/go.db" "$WEBDAV_URL" "$user" "$password" go-device "$remote" >/dev/null; }
nunc_sync() { cargo run -q -p nunc-sync --example webdav_sync -- "$tmp/nunc.db" "$WEBDAV_URL" "$user" "$password" "$remote" "$@"; }

# 1. Go: Fixture-Datenbank der Go-App (6 Todos, davon 1 gelöscht; 9 abgeschlossene Sessions) lädt hoch
cp tests/fixtures/go-v5.db "$tmp/go.db"
go_sync
echo "ok: Go hat die Sync-Datei angelegt"

# 2. nunc: leere Datenbank, legt ein eigenes Todo an und übernimmt den Stand der Go-App
result=$(nunc_sync "Todo aus nunc")
echo "nunc: $result"
expect "nunc importiert Todos" "$(echo "$result" | python3 -c 'import json,sys; print(json.load(sys.stdin)["importedTodos"])')" 5
expect "nunc importiert Sessions" "$(echo "$result" | python3 -c 'import json,sys; print(json.load(sys.stdin)["importedSessions"])')" 9
expect "nunc: aktive Todos" "$(count "$tmp/nunc.db" 'SELECT COUNT(*) FROM todos WHERE deleted = 0')" 6

# 3. Go: übernimmt das Todo von nunc
go_sync
expect "Go: aktive Todos" "$(count "$tmp/go.db" 'SELECT COUNT(*) FROM todos WHERE deleted = 0')" 6
expect "Go: Todo aus nunc vorhanden" "$(count "$tmp/go.db" "SELECT COUNT(*) FROM todos WHERE raw_text = 'Todo aus nunc' AND deleted = 0")" 1

# 4. nunc: neue abgeschlossene Session und eine Änderung an einem Todo der Go-App
sqlite3 "$tmp/nunc.db" "INSERT INTO sessions (mode, session_type, status, started_at, ended_at, planned_min, actual_min, sync_device_id)
  VALUES ('pomodoro','work','completed','2026-10-04T08:00:00Z','2026-10-04T08:25:00Z',25,25,(SELECT value FROM settings WHERE key='device_id'))"
sqlite3 "$tmp/nunc.db" "UPDATE todos SET raw_text='(A) 2026-09-22 Buy groceries from nunc', updated_at='2099-01-01T00:00:00Z' WHERE sync_id='t-0003'"
nunc_sync >/dev/null
go_sync
expect "Go: Sessions" "$(count "$tmp/go.db" "SELECT COUNT(*) FROM sessions WHERE status='completed'")" 10
expect "Go: Todo von nunc geändert" "$(count "$tmp/go.db" "SELECT raw_text FROM todos WHERE sync_id='t-0003'")" "(A) 2026-09-22 Buy groceries from nunc"

# 5. Go: löscht ein Todo (weich), nunc übernimmt die Löschung
sqlite3 "$tmp/go.db" "UPDATE todos SET deleted=1, updated_at='2099-02-01T00:00:00Z' WHERE sync_id='t-0001'"
go_sync
nunc_sync >/dev/null
expect "nunc: Löschung von Go übernommen" "$(count "$tmp/nunc.db" "SELECT deleted FROM todos WHERE sync_id='t-0001'")" 1

# 6. Beide Seiten sind jetzt gleich (aktive Todos, abgeschlossene Sessions; ein dort schon gelöschtes Todo legt nunc nicht an)
todos() { sqlite3 "$1" "SELECT raw_text FROM todos WHERE deleted = 0 ORDER BY raw_text"; }
if [ "$(todos "$tmp/go.db")" != "$(todos "$tmp/nunc.db")" ]; then
  diff <(todos "$tmp/go.db") <(todos "$tmp/nunc.db") >&2 || true
  fail "Todos beider Seiten unterscheiden sich"
fi
echo "ok: Todos stimmen überein"
expect "Sessions beider Seiten" \
  "$(count "$tmp/nunc.db" "SELECT COUNT(*) FROM sessions WHERE status='completed'")" \
  "$(count "$tmp/go.db" "SELECT COUNT(*) FROM sessions WHERE status='completed'")"
echo "Interop Go <-> nunc bestanden"
