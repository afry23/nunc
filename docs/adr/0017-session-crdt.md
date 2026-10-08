# ADR 0017 – Sessions als CRDT

- Status: vorgeschlagen
- Datum: 2026-10-08
- Ersetzt teilweise: ADR 0016 (Sessions als G-Set, Dateiformat `nunc-sync.v2.json`)

## Kontext

Für die Zeiterfassung (M11) müssen Sessions nachträglich bearbeitet, gelöscht und von Hand angelegt werden können,
und Projektzeit muss über alle Geräte hinweg stimmen. ADR 0016 lässt das nicht zu:

- **Kein Todo-Bezug im Sync.** `SyncSession` (`nunc-core/src/model/sync.rs`) kennt das zugehörige Todo nicht. Sessions
  anderer Geräte zählen deshalb nie zu einem Projekt.
- **Unveränderlich.** Sessions sind ein Grow-only-Set mit dem Schlüssel `device|started_at` (`crdt.rs::session_key`),
  der Merge lässt den größeren Wert gewinnen. Bearbeiten (ändert den Schlüssel) und Löschen lassen sich nicht
  synchronisieren.
- **Schema.** `sessions.mode CHECK IN ('pomodoro','flowtime')` (Migration 001, unantastbar) lässt keinen neuen Modus für
  manuelle Einträge zu.

## Entscheidung

**Sessions werden ein Todo-ähnlicher, zustandsbasierter CRDT** (`SessionCrdt`, pur in `nunc-core`, analog `TodoCrdt`).

- **Identität:** Jede Session bekommt eine stabile `sync_id` (UUID, bei Anlage vergeben). Bestehende Sessions ohne
  `sync_id` bekommen eine **deterministische** ID, abgeleitet aus `session_key`; zwei Geräte vergeben so für dieselbe
  Alt-Session dieselbe ID, es entstehen keine Dubletten.
- **Felder** sind LWW-Register `(Wert, HLC)` (`hlc.rs`, wie bei Todos): `started_at`, `actual_min`, `todo`
  (`sync_id` des Todos, nicht die lokale Zeilen-ID), `note`, `deleted` (Tombstone). Unveränderlich bleiben
  `device_id`, `mode`, `session_type`, `status`, `planned_min`, `break_min`; sie werden beim Anlegen gesetzt.
- Der Merge ist kommutativ, assoziativ und idempotent; Gleichstand entscheidet `device_id`. `sanitized` kappt Remote-
  Stempel wie bei Todos (24 h Drift). Ein Tombstone gewinnt nur, wenn er jünger ist als jede Bearbeitung.
- **Tombstone-GC:** wie Todos nach 90 Tagen (`TOMBSTONE_TTL_MS`, `is_collectable`). Dasselbe Risiko wie in ADR 0016:
  ein länger als 90 Tage offline gewesenes Gerät legt die Session wieder an.
- **Format:** `nunc-sync.v3.json`, `SCHEMA_VERSION = 3`. Gelesen werden v3 und v2; v2-Sessions werden beim Einlesen
  zu `SessionCrdt` mit Stempeln aus dem Pseudo-Gerät `legacy` gehoben. Geschrieben wird nur v3. Der Versionscheck
  (`sync_document.rs`) vergleicht strikt auf Gleichheit: Ältere Clients lehnen v3 mit `UnsupportedVersion` ab, statt
  neue Felder still zu verwerfen und beim Zurückschreiben zu löschen. Die v2-Datei bleibt unangetastet liegen.
- **Store:** additive Migration `008_session_crdt.sql`:
  `session_meta(session_id PK→sessions, sync_id UNIQUE, origin 'timer'|'manual', note, deleted)` und
  `session_field_clock(session_id, field, hlc…)` nach dem Muster von `todo_field_clock`. Bestandssessions erhalten
  ihre `sync_id` aus `session_key`. `sessions` bleibt der aufgelöste Stand; `list_sessions` blendet Tombstones aus, damit
  Statistik und Export sie erben. Mehrschrittige Schreibvorgänge laufen in einer Transaktion.
- **Manuelle Einträge** liegen als `mode='flowtime'`, `session_type='work'`, `status='completed'` in `sessions`,
  `origin='manual'` in `session_meta`. Der Modus-CHECK bleibt unberührt.
- **Validierung** (pur, `Clock` injiziert): Ende > Start, höchstens 24 h, nicht in der Zukunft. Überlappung mit
  anderen Sessions ist eine Warnung, kein Fehler.
- **Einspielen:** `local_state`/`apply_state` (`crdt_repo`) übertragen Sessions mit Todo-`sync_id` und Registern;
  die lokale `todo_id` wird beim Anwenden über `sync_id` aufgelöst. Ist das Todo (noch) unbekannt, bleibt die Session
  ohne Todo-Bezug und wird beim nächsten Lauf nachgezogen.
- Abgeleitete Auswertungen (Timesheet, Timeline) werden nie gespeichert, sondern aus den Roh-Sessions berechnet.

## Verworfene Alternativen

- **Neuer Modus `manual` in `sessions`:** bräuchte Änderung an Migration 001 (unantastbar) bzw. Tabellen-Neuaufbau.
- **G-Set beibehalten, Korrekturen als Ereignisse (Op-Log):** Kompaktion und Zustellgarantie, passt schlecht zu
  Datei-Sync (siehe ADR 0016).
- **v2 mit optionalen Feldern (`serde(default)`):** v2-Clients würden Bearbeitungen und Tombstones beim Zurückschreiben
  stillschweigend verlieren.

## Folgen

- Breaking im Mischbetrieb: Geräte mit nunc < der M11-Version lesen die v3-Datei nicht mehr, bis sie aktualisiert sind.
  Hinweis in den Release-Notes (`docs/release.md`).
- Dateigröße wächst durch Register und Tombstones; die GC räumt nach 90 Tagen ab.
- Projektzeit anderer Geräte erscheint nach dem Sync, weil der Todo-Bezug jetzt mitgeführt wird.
- Tests: `@domain @property` (Session-Merge kommutativ/assoziativ/idempotent), `@integration` (Migration 008 auf
  Bestands-DB, Backup alter Version restorebar), `@integration @sync` (zwei Geräte, eines bearbeitet, eines löscht;
  v2-Datei wird gelesen), `@ui` für Timesheet, Timeline und Dialoge.
