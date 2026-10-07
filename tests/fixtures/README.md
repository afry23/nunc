# Test-Fixtures aus der Go-App

Mit der echten Go-Logik (`nunc-go/internal/db`, `services`) erzeugt, damit die Rust-Tests gegen genau das
Schema und die Datenformate laufen, die `nunc-go` schreibt. Die Daten sind frei erfunden und deterministisch.

| Datei | Inhalt |
|---|---|
| `go-v5.db` | SQLite nach den Migrationen 001–005 (`schema_migrations` = 1…5, WAL): 6 Todos (davon 1 erledigt, 1 soft-deleted), 11 Sessions (Pomodoro, Flowtime, Pausen, abgebrochen, übersprungen), veränderte Settings, `device_id = fixture-device-a`, `sync_provider = webdav` |
| `nunc-sync.json` | Sync-Datei im alten Format (vor ADR 0016) von Gerät A zum Stand der DB (`schema_version` 1, Format `models.SyncFile`) |
| `webdav/nunc-sync.json` | Remote-Datei im alten Format von Gerät B für Merge-Tests: `t-0002` neuer, `t-0003` älter, `t-0005` gelöscht, neues Todo `t-0100`, zwei neue Sessions, eine Session-Dublette |

Hinweise aus dem Erzeugen:

- Die Go-`description` enthält die Tags (`Write quarterly report +work @desk`); der Rust-Parser muss das gleich abbilden.
- `SetCompleted` der Go-App schreibt in `raw_text` das heutige Datum und formatiert ein DATE-`created_at` als RFC 3339
  (`x 2026-10-03 2026-09-18T00:00:00Z …`). Der Generator setzt `raw_text` deshalb auf die kanonische Form zurück.
- `go-v5.db` ist ausdrücklich von `*.db` in `.gitignore` ausgenommen.

## Neu erzeugen

`generator/main.go.txt` ist das Go-Programm (Endung `.txt`, damit es hier nicht als Go-Code gilt). Es braucht den
`internal/`-Baum von `nunc-go`:

```bash
tmp=$(mktemp -d) && cp -R ../nunc-go/internal ../nunc-go/go.mod ../nunc-go/go.sum "$tmp"
mkdir -p "$tmp/cmd/gen" && cp tests/fixtures/generator/main.go.txt "$tmp/cmd/gen/main.go"
mkdir -p "$tmp/out" && (cd "$tmp" && go run ./cmd/gen out)
cp "$tmp/out/go-v5.db" "$tmp/out/nunc-sync.json" tests/fixtures/ && cp "$tmp/out/webdav/nunc-sync.json" tests/fixtures/webdav/
```
