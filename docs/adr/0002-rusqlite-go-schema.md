# ADR 0002 – SQLite über rusqlite, Schema der Go-App unverändert

- Status: akzeptiert
- Datum: 2026-10-03

## Kontext

Die Rust-App muss dieselben Datenbanken lesen und schreiben wie die Go-App (`modernc.org/sqlite`, Migrationen
001–005, Tabelle `schema_migrations`), damit Umstieg, Backup-Restore und paralleler Betrieb funktionieren.

## Entscheidung

- **`rusqlite` mit Feature `bundled`** (eigenes SQLite, identisches Verhalten auf allen Plattformen inkl. Mobile),
  WAL-Modus, `PRAGMA foreign_keys = ON`, eine geteilte Verbindung hinter `Mutex<Connection>` im Tauri-State.
- Eigener, dünner **Migrationsrunner** in `nunc-store`, kompatibel zur Tabelle `schema_migrations` der Go-App.
  Migrationen 001–005 werden unverändert übernommen (`include_str!`), neue Migrationen sind **ausschließlich
  additiv** (z. B. 006 `active_timer`; die Go-App ignoriert unbekannte Tabellen).
- Kein ORM und kein `sqlx`: wenige, einfache Queries; synchrone Zugriffe aus dem Command-Thread reichen.
- Backup per `VACUUM INTO`, Restore mit Validierung (SQLite-Integrität + Schema-Version) und atomarem Austausch.

Alternativen: `sqlx` (async/Compile-Time-Checks, aber Runtime- und Mobile-Overhead, Migrationen nicht kompatibel),
`diesel` (ORM-Schema statt der vorhandenen Migrationen), `tauri-plugin-sql` (Zugriff aus dem Frontend, unterläuft die
Schicht-Trennung).

## Konsequenzen

- Bestehende Datenbanken (Schema 001–005, Go-Fixtures) werden durch `existing_database.feature` abgesichert (M0-13, M2-08). Seit ADR 0016 gibt es keinen Parallelbetrieb mit der Go-App mehr; das Schema bleibt für Bestandsdaten erhalten.
- Schemaänderungen sind append-only (neue, additive Migrationen); Umbauten bestehender Spalten sind ausgeschlossen,
  damit Bestandsdaten und Backups älterer Versionen weiter funktionieren.
- `bundled` verlängert die Erstkompilierung (C-Compiler nötig); in CI mit Rust-Cache abgefangen.

## Umsetzung (M2)

- **Migrationen:** 001–005 byte-identisch aus `nunc-go`, 006 `active_timer` (eine Zeile, JSON des `Timer`). Jede
  Migration läuft samt `schema_migrations`-Eintrag in einer Transaktion; maßgeblich ist wie in Go das Vorhandensein
  der Version, höhere unbekannte Versionen bleiben unberührt. Das DDL von `schema_migrations` ist wortgleich zu Go.
- **Session abschließen:** `commit_finish` schreibt Session, Pomodoro-Zähler und neuen Timer-Zustand in **einer**
  Transaktion (sonst entstünde nach einem Absturz eine doppelte Session).
- **Backup:** `VACUUM INTO` in `….db.tmp`, dann `rename`. Regeln (wann, Namen, Rotation) in `nunc-core`.
- **Restore:** Kopie neben der Datenbank vorbereiten, **dort** prüfen (SQLite-Kopf, `quick_check`, Basis-Migration,
  keine unbekannte neuere Schema-Version) und migrieren, Notfall-Backup, Verbindung schließen, `rename`, neu öffnen.
  Die Originaldatenbank bleibt bis zum Austausch unberührt; die gewählte Datei wird nie geöffnet (schon ein
  Read-only-Öffnen einer WAL-Datenbank legt `-shm` daneben an).
- **Legacy-Übernahme:** gleiches Vorgehen für `…/nunc/nunc.db` der Go-App inklusive noch nicht eingespieltem `-wal`.
- **Interop** (entfallen mit ADR 0016): Das Skript `tests/interop/go-reads-nunc-db.sh` ließ die Go-Logik eine von nunc
  geschriebene Datenbank lesen. Die Go-App wird nicht mehr berücksichtigt.
