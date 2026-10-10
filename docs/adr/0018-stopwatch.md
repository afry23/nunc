# ADR 0018 – Stopwatch als dritter Timer-Modus

- Status: vorgeschlagen
- Datum: 2026-10-09
- Ergänzt: ADR 0004 (Timer), ADR 0017 (Sessions als CRDT)

## Kontext

Für die Zeiterfassung (M11) fehlt ein Timer ohne Ziel und ohne Pausenlogik: eine freie Stoppuhr. Flowtime zählt zwar
ebenfalls hoch, schlägt aber beim Stopp eine Pause vor und erhöht den Pomodoro-Zähler des Todos.

## Entscheidung

- **Neuer `TimerMode::Stopwatch`** (`snake_case`: `stopwatch`). Der Automat nutzt wie Flowtime `planned_sec == 0`.
  Beim Stopp ist die Session immer `Completed`, es gibt **keine** Pausenempfehlung und **kein** `increment_todo`.
  Zuordnung zu Todo/Projekt bleibt möglich.
- **Persistenz in `sessions`:** `mode='flowtime'` (der `CHECK` aus Migration 001 ist unantastbar, siehe ADR 0017).
  Unterschieden wird über die neue additive Spalte `session_meta.kind` (`regular` | `stopwatch`, Default `regular`)
  aus Migration `009_session_kind.sql`. Dieselbe Konstruktion wie `origin='manual'`.
- **Domäne:** `Session.mode` darf `Stopwatch` sein. Der Store bildet beim Speichern `Stopwatch` auf `mode=flowtime` +
  `kind=stopwatch` ab und setzt beim Lesen beides wieder zusammen.
- **Sync:** `kind` ist ein unveränderliches Feld von `SessionCrdt` (wie `origin`). `schema_version` wird 4, Gelesen
  werden 2 bis 4 (fehlendes `kind` = `regular`). Dateiname bleibt `nunc-sync.v2.json`. Ältere Clients lehnen v4 ab
  (strikter Versionscheck, ADR 0017).
- **Nachgetragene Einträge zählen als Stoppuhr:** `kind='stopwatch'` bei `origin='manual'` (Migration
  `010_manual_is_stopwatch.sql` für Bestand, `insert_manual` und Einspielen vom Sync für Neues).
- **Auswertung:** Stopwatch-Zeit zählt wie Flowtime-Arbeit in Arbeitszeit, Timesheet und Timeline. Sie ist kein
  Pomodoro und zählt nicht für Serien. `DailyStats` führt Anzahl und Minuten je Timer (Pomodoro, Flowtime,
  Stoppuhr); die Diagramme zeigen gestapelte Balken und eine Linie je Timer in der Timer-Farbe. Im CSV-Export
  steht `mode` = `stopwatch` für Stoppuhr- und nachgetragene Sessions, die Spalten bleiben gleich.

## Verworfene Alternativen

- **`sessions.mode='stopwatch'`:** bräuchte einen Tabellen-Neuaufbau (Migration 001 unantastbar) und bräche beim Sync
  mit älteren Clients am `CHECK`.
- **Stopwatch als Flowtime-Variante per Einstellung:** vermischt zwei fachlich verschiedene Abläufe (Pause, Todo-Zähler).

## Folgen

- Downgrade: Ein gespeicherter `active_timer` mit `mode: "stopwatch"` ist für ältere Builds nicht ladbar. Nach einem
  Downgrade während eine Stopwatch lief, geht dieser eine laufende Timer verloren (Sessions bleiben).
- Mischbetrieb: Geräte mit nunc < dieser Version lesen die v4-Datei nicht (Hinweis in `docs/release.md`).
- Nicht Teil dieses ADR: Kennzeichnung in der Timeline.
- Tests: `@domain` (Automat), `@integration` (Migration 009, Persistenz, Resume), `@sync` (kind-Merge, v3-Datei),
  `@ui` (Modus-Auswahl, Ring, kein Pausendialog).
