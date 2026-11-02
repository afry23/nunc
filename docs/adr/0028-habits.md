# ADR 0028 – Habit-Tracker

- Status: vorgeschlagen
- Datum: 2026-11-02
- Ergänzt: ADR 0016 (CRDT-Sync), ADR 0017 (Sessions als CRDT), ADR 0023 (Projekte/Kontexte je Session), ADR 0026 (Journal), ADR 0027 (Tagesplaner)

## Kontext

nunc misst Fokuszeit und kennt Streaks nur für „irgendeine Session am Tag“. Wiederkehrende Gewohnheiten („täglich
Deutsch üben“, „dreimal pro Woche Sport“, „30 Minuten Deep Work an Werktagen“) lassen sich nicht abbilden. Sie sollen
entweder von Hand abgehakt oder **automatisch** aus den gemessenen Sessions abgeleitet werden (Regel über Projekt oder
Kontext), damit nichts doppelt erfasst wird. Keine Gamification, keine Erinnerungen in der ersten Fassung.

## Entscheidung

**Zwei CRDT-Typen** (rein in `nunc-core`, zustandsbasiert wie `JournalEntry`):

`Habit` – eine Gewohnheit, Identität `sync_id` (UUID). Felder als LWW-Register `(Wert, HLC)`:

| Feld | Inhalt |
|---|---|
| `name` | 1 bis 100 Zeichen (nach `trim`) |
| `schedule` | Rhythmus, als Text kodiert (siehe unten) |
| `goal` | Ziel, als Text kodiert |
| `archived` | ausgeblendet, Verlauf bleibt |
| `deleted` | gelöscht (Tombstone, 90 Tage) |

`created` (Tag der Anlage) ist unveränderlich und ordnet die Liste (`created`, dann `sync_id`).

`HabitLog` – ein Haken für (Gewohnheit, Tag), Identität `log:<habit-sync_id>:<YYYY-MM-DD>`, **deterministisch**, damit zwei
Geräte denselben Tag nicht doppelt anlegen. Einziges Register: `done` (Bool). `done = false` ist der Tombstone („Haken
entfernt“) und wird wie ein Todo-Tombstone nach 90 Tagen eingesammelt.

**Rhythmus** (`Schedule`):

- `daily` – jeden Tag fällig.
- `days:1,3,5` – nur an diesen Wochentagen (1 = Montag … 7 = Sonntag, sortiert, ohne Wiederholung, mindestens einer).
- `weekly:N` – N Tage pro Woche (1 bis 7), beliebige Tage; die Woche läuft Montag bis Sonntag (wie ADR 0025).

**Ziel** (`Goal`):

- `check` – von Hand abhaken; Grundlage sind die `HabitLog`-Einträge.
- `minutes:M:TAG` – automatisch: Der Tag zählt als erledigt, wenn die **Arbeitszeit** der Sessions mit dem Tag `TAG`
  (`+projekt` oder `@kontext`, wie in ADR 0023 über `effective_tags`) am lokalen Tag mindestens `M` Minuten (1 bis 1440)
  beträgt. Es gibt keine Logs für solche Gewohnheiten; nichts wird abgeleitet gespeichert. Gezählt werden
  Arbeitsintervalle, die nicht übersprungen wurden (wie in ADR 0027 `spent_min`).

Unbekannte oder kaputte Kodierungen (neuere App) lesen sich als **nicht auswertbar**: die Gewohnheit bleibt sichtbar,
zählt aber nichts, und der Wert wird beim Zurückschreiben nicht verändert.

**Auswertung** (`habit_progress`, pur; `today` und die Menge der erledigten Tage bringt der Aufrufer mit):

- `done_today`, `due_today` (laut Rhythmus fällig; bei `weekly:N` ist jeder Tag möglich, „fällig“ solange die Woche ihr
  Ziel nicht erreicht hat).
- **Serie** (`current_streak`, `best_streak`): bei `daily` und `days` zählen aufeinanderfolgende **fällige** Tage mit Haken;
  nicht fällige Tage unterbrechen nichts und zählen nicht. Ist der heutige fällige Tag noch offen, bricht die Serie nicht
  (sie zählt ab gestern). Bei `weekly:N` zählt die Serie in **Wochen**, in denen mindestens N Tage erledigt sind; die
  laufende Woche zählt mit, sobald sie ihr Ziel erreicht hat, bricht aber nicht, solange sie noch läuft. Die Einheit
  (`Days`/`Weeks`) liefert die Auswertung mit.
- `week_done` / `week_target`: erledigte Tage in der laufenden Woche und, bei `weekly:N`, das Ziel N.

**Store:** additive Migration `015_habits.sql`: `habits(id, sync_id UNIQUE, created, name, schedule, goal, archived,
deleted)`, `habit_field_clock(habit_id, field, hlc…)`, `habit_logs(sync_id PK, habit_sync_id, day, done)` mit
`habit_log_clock`. Mehrschrittige Schreibvorgänge in einer Transaktion.

**Sync:** `SyncState.habits` und `SyncState.habit_logs`; in der Datei die Listen `habits` und `habit_logs`
(`serde(default)`). `schema_version` wird **8**; gelesen werden 2 bis 8, geschrieben wird 8 (Begründung wie ADR 0026:
ältere Clients lehnen v8 ab, statt die Gewohnheiten still zu verwerfen).

**Rückblick:** `Review` bekommt später je Gewohnheit die erledigten Tage im Zeitraum und die Quote (Stufe 4).

## Umsetzung in Stufen

1. `nunc-core`: Typen, Kodierung, Validierung, CRDT-Merge, Auswertung, `SyncState`/`SyncDocument` (Format 8) – diese Stufe.
2. `nunc-store` und `nunc-sync`: Migration 015, Repo, Anbindung an den Sync-Lauf (bis dahin enthält der Zustand aus der
   Datenbank keine Gewohnheiten; es gibt auch noch keine Möglichkeit, welche anzulegen).
3. Commands (`list_habits`, `save_habit`, `archive_habit`, `delete_habit`, `set_habit_done`, `get_habit_overview`).
4. UI (Ansicht „Gewohnheiten“, Abschnitt in „Heute“) und Rückblick.

## Verworfene Alternativen

- **Haken als Menge je Gewohnheit in einem Register:** Zwei Geräte, die verschiedene Tage abhaken, würden sich
  gegenseitig überschreiben. Ein Log je Tag merged ohne Verlust.
- **Auto-Gewohnheit als Haken speichern:** doppelte Wahrheit; würde nach dem Löschen oder Nachtragen von Sessions
  veralten. Die Auswertung rechnet aus den Roh-Sessions (CLAUDE.md: keine abgeleiteten Statistiken speichern).
- **Serie immer in Tagen:** bei „dreimal pro Woche“ wäre jede Pause zwischen den Trainingstagen ein Bruch.

## Folgen

- Gelöschte Gewohnheiten lassen ihre Logs zurück; sie werden beim Einsammeln mit entfernt, sobald die Gewohnheit nicht
  mehr existiert.
- Eine Auto-Gewohnheit kann man rückwirkend ändern (anderes Tag, andere Minuten); die Auswertung folgt sofort.
