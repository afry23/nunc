# ADR 0028 – Habit-Tracker

- Status: ersetzt durch ADR 0033
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

**Store:** additive Migration `015_habits.sql`: `habits(sync_id PK, created, name, schedule, goal, archived, deleted)` mit
`habit_field_clock(sync_id, field, hlc…)` (wie beim Journal) und `habit_logs(sync_id PK, habit, day, done, hlc_ms,
hlc_counter, hlc_device)`; der einzige Stempel eines Hakens steht in der Zeile selbst. Haken haben keinen Fremdschlüssel:
der Sync-Lauf entfernt Haken, die der zusammengeführte Stand nicht mehr enthält, wenn sie entfernt sind oder ihre
Gewohnheit fehlt. Haken gibt es nur bei Ziel `check`; `set_habit_done` lehnt andere Gewohnheiten ab. Mehrschrittige
Schreibvorgänge laufen in einer Transaktion.

**Sync:** `SyncState.habits` und `SyncState.habit_logs`; in der Datei die Listen `habits` und `habit_logs`
(`serde(default)`). `schema_version` wird **8**; gelesen werden 2 bis 8, geschrieben wird 8 (Begründung wie ADR 0026:
ältere Clients lehnen v8 ab, statt die Gewohnheiten still zu verwerfen).

**Rückblick:** `Review.habits` listet je auswertbarer Gewohnheit, die bis zum Ende des Zeitraums angelegt war, die
**fälligen Tage**, die **erledigten Tage** und die **Quote** (gerundet, höchstens 100 %; `null`, wenn nichts fällig war).
Gezählt wird von `max(Beginn, Anlage)` bis `min(Ende, heute)`. Fällig sind bei `daily` alle Tage, bei `days` die
gewählten Wochentage, bei `weekly:N` aufgerundet N Tage je sieben Tage des Zeitraums. Archivierte Gewohnheiten erscheinen
nur, wenn sie im Zeitraum erledigt wurden. Der Markdown-Export hat einen Abschnitt „Gewohnheiten“ (`Name: erledigt / fällig (Quote)`).

## Umsetzung in Stufen

1. `nunc-core`: Typen, Kodierung, Validierung, CRDT-Merge, Auswertung, `SyncState`/`SyncDocument` (Format 8) – diese Stufe.
2. `nunc-store`: Migration 015, Repo, Anbindung an `local_state`/`apply_state` und damit an den Sync-Lauf – diese Stufe
   (es gibt noch keine Commands, also noch keine Möglichkeit, Gewohnheiten anzulegen).
3. Commands `get_habit_overview` (alle Gewohnheiten mit Stand, Serien und den zuletzt erledigten Tagen; ersetzt ein
   eigenes `list_habits`), `save_habit` (anlegen oder ändern), `archive_habit`, `delete_habit`, `set_habit_done` – diese
   Stufe. Die Übersicht rechnet `habit_overview` in `nunc-core`; ein Eintrag mit unbekannter Kodierung hat keinen
   `progress`.
4. UI in zwei Teilen: (a) Ansicht „Gewohnheiten“ (`/habits`: anlegen, ändern, abhaken, archivieren, löschen) und Abschnitt
   „Gewohnheiten heute“ in „Heute“ – diese Stufe; (b) Rückblick: Quote je Gewohnheit im Zeitraum (`Review.habits`, Markdown-Abschnitt, Karte im Rückblick).

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
