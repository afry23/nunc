# M11 – Zeiterfassung (Projektzeit, Korrektur, Timeline)

## Context

nunc misst Fokuszeit schon (Tabelle `sessions`), nutzt sie aber nur für Statistik. Ziel: Zeiterfassung ähnlich
TickTick / Super Productivity – „Wie viel Zeit ging diese Woche in +ProjektX?“, falsche/vergessene Sessions
korrigieren, Tag als Zeitstrahl sehen. **Kalender-Integration bewusst weggelassen** (Backlog bleibt).

Bestand (Analyse):
- `nunc-core/src/stats.rs` rechnet Projekt-/Kontext-/Todo-Zeit schon aus Roh-Sessions (Tags des **aktuellen** Todos).
  `StatsView` zeigt Projekte als Karte. `export.rs` liefert CSV/JSON mit `project_tags`.
- **Lücke 1 – Sync:** `SyncSession` (`nunc-core/src/model/sync.rs:49`) hat **keinen Todo-Bezug**. Sessions anderer
  Geräte zählen daher nie zu Projekten. `crdt_repo::local_state` exportiert nur `completed`.
- **Lücke 2 – Unveränderlich:** Sessions sind ein Grow-only-Set, Schlüssel `device|started_at`
  (`crdt.rs::session_key`), Merge = größerer Wert gewinnt. Bearbeiten/Löschen geht so nicht synchron.
- **Lücke 3 – Schema:** `sessions.mode CHECK IN ('pomodoro','flowtime')` (Migration 001, unantastbar) → manuelle
  Einträge brauchen Zusatztabelle statt neuem Modus.

## Ansatz

### M11-01 ADR 0017 „Sessions als CRDT“ + `@wip`-Features
- Session bekommt stabile `sync_id` (UUID), Felder als LWW-Register (HLC aus `hlc.rs`, wie `TodoCrdt`):
  `started_at`, `actual_min`, `todo` (Todo-`sync_id`), `note`, `deleted`. Tombstone-GC 90 Tage wie Todos.
- Sync-Format bleibt `nunc-sync.v2.json`, neue Felder optional (`serde(default)`); Alt-Sessions ohne `sync_id`
  bekommen deterministische ID aus `session_key` → keine Dubletten. Prüfen: `SyncDocument` ohne
  `deny_unknown_fields`, sonst Formatversion anheben.
- Features: `features/timetracking/{report,edit,manual,timeline}.feature`, Tags `@domain`/`@integration`/`@ui`,
  Story-Tags `@US-TT01…04`.

### M11-02 Kern (`nunc-core`, pur)
- `crdt.rs`: `SessionCrdt` analog `TodoCrdt` (merge, sanitized, is_collectable); `SyncState.sessions` darauf umstellen.
- Neues Modul `timesheet.rs`: `timesheet(sessions, todos, range, group_by: Project|Context|Todo, local_date)` →
  Matrix Gruppe × Tag + Summen; Sessions ohne Tag → Gruppe „ohne Projekt“. Pausen zählen nicht.
- `timeline.rs`: Sessions eines lokalen Tages → Blöcke (Start, Ende, Typ, Todo), Lücken, Überlappungs-Erkennung.
- Validierung manueller Einträge: Ende > Start, max. 24 h, keine Zukunft (Clock injiziert), Überlappung = Warnung.

### M11-03 Store (`nunc-store`) – Migration `008_session_crdt.sql` (additiv)
- `session_meta(session_id PK→sessions, sync_id UNIQUE, origin 'timer'|'manual', note, deleted)`,
  `session_field_clock(session_id, field, hlc…)` wie `todo_field_clock`. Bestand: `sync_id` aus `session_key`.
- `session_repo.rs`: `insert_manual`, `update_session` (Felder + Stempel), `delete_session` (Tombstone), alles in
  einer Transaktion; `list_sessions` filtert Tombstones (Stats/Export erben das).
- `crdt_repo::local_state`/`apply_state`: Sessions mit Todo-`sync_id` und Registern; `todo_id` beim Anwenden über
  `sync_id` auflösen. Manuelle Einträge: `mode='flowtime'`, `session_type='work'`, `status='completed'`.

### M11-04 Commands (`src-tauri/src/commands/`)
- `get_timesheet`, `get_day_timeline`, `create_manual_session`, `update_session`, `delete_session`
  (specta, `AppError`-Codes `session.invalidRange` u. a. in `de.ts`/`en.ts`), Sync-Trigger nach Änderung wie bei Todos.
- `export_bindings` laufen lassen.

### M11-05 UI
- `StatsView`: neuer Reiter „Zeiten“ – Wochen-Timesheet (Gruppe × Mo–So, Summen, Umschalter Projekt/Kontext/Aufgabe,
  Woche vor/zurück), CSV über vorhandenen Export (`export.rs`, Zeilen um `note`/`origin` erweitern).
- Tages-Timeline (`components/Timeline.tsx`): vertikaler Zeitstrahl, Blöcke farbig nach Typ, Tippen → Bearbeiten-Dialog
  (`Dialog`), „+ Eintrag“ für Nachtrag, Löschen mit `ConfirmDialog`. Mobil ≈ 411 px ohne Überlauf.
- Texte nur in `src/i18n`, Tokens aus `styles.css`, Logik in `src/lib/timesheet.ts` (100 % Coverage).

### Später (nicht M11)
Stoppuhr-Modus ohne Pausenlogik, Kalender (ICS-Export → Google/CalDAV), Zeit-Schätzung je Aufgabe, Lokale API
`GET /timesheet`.

## Kritische Dateien
`crates/nunc-core/src/{crdt.rs,model/sync.rs,stats.rs,export.rs}`, neu `timesheet.rs`/`timeline.rs`;
`crates/nunc-store/src/{session_repo.rs,crdt_repo.rs}`, `migrations/008_session_crdt.sql`;
`src-tauri/src/commands/`, `src/views/StatsView.tsx`, `src/components/charts.tsx`, `src/i18n/{de,en}.ts`;
`docs/adr/0017-session-crdt.md`, `roadmap.md` (M11 eintragen).

## Verifikation
- `cargo test -p nunc-core --test bdd` (Timesheet-`Examples`, `@property`: Session-Merge kommutativ/assoziativ/idempotent).
- `cargo test -p nunc-store --test bdd` (Migration 008 auf Bestands-DB, Backup alter Version restorebar,
  Bearbeiten/Löschen, Alt-Sync-Datei ohne neue Felder).
- `cargo test -p nunc-sync --test bdd`: zwei Geräte, eins bearbeitet, eins löscht → konvergiert; Projektzeit
  des anderen Geräts erscheint.
- `npm run test:bdd` (Timesheet/Timeline/Dialoge), alle Pflicht-Checks, `cargo mutants --in-diff`.
- Manuell: macOS-App + Android-Emulator, Session nachtragen, auf zweitem Gerät nach Sync prüfen.
