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

- Kompatibilität wird durch `compat_go.feature` mit echten Go-Fixtures abgesichert (M0-13, M2-08).
- Schemaänderungen sind in der Praxis append-only; Umbauten bestehender Spalten sind ausgeschlossen, solange
  die Go-App parallel genutzt wird.
- `bundled` verlängert die Erstkompilierung (C-Compiler nötig); in CI mit Rust-Cache abgefangen.
