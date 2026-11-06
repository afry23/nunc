# Plan: UI-Umbau nunc – Heute · Aufgaben · Verlauf · Rückblick

## Context
Quelle: `docs/AppUmbau.pdf`. Heute zeigt die Navigation Werkzeuge (Timer, Todos, Times, Stats) statt Arbeitsablauf.
Today und Habits sind nur per Link erreichbar, der Timer ist Eingangstür, Times mischt Zeiten und Review, Stats
zeigt v. a. Timer-Nutzung. Ziel: Hierarchie ändern, nicht alles neu bauen. Gleiche Informationsarchitektur auf
Desktop und Android. Habit-Tracker wird komplett entfernt (Entscheidung User).

Ist-Stand (Erkundung): `src/router.tsx` (Routen `/`, `/todos`, `/today`, `/habits`, `/times`, `/stats`, `/settings`, `/help`, `/mini`),
`components/AppShell.tsx`, `components/Navigation.tsx` (`NAV_ITEMS`, Sidebar ≥768 px via `useIsWide`, sonst `BottomTabs`),
`views/{Timer,Todo,Today,Habits,Times,Stats,Settings}View`, `TimesView` = Segmented `TimesheetPanel`/`ReviewPanel`,
`SyncStatusBar` oben in jeder Route, kein globaler Timer-Indikator (nur `timer-breathing`, Mini-Fenster Desktop).
Backend hat schon fast alles: `get_day_plan`, `plan_todo`, `close_day`, `get_timesheet`, `get_day_timeline`, `create_manual_session`,
`get_review`, Journal (day_note/week_note/achievement), Sessions ohne Todo mit Tags (ADR 0023), `est:`/`plan:` in todo.txt (ADR 0027).

## Zielstruktur
Vier Hauptbereiche, Einstellungen/Hilfe sekundär:

| Route | Inhalt | Quelle (wiederverwenden) |
|---|---|---|
| `/` **Heute** (Startseite) | Tab Plan / Verlauf / Notizen; Tagesfokus; Aufgaben für heute; „Aufgaben auswählen“; „Freie Tätigkeit starten“; Tag abschließen | `TodayView`, `DayNote`, `CloseDayDialog`, `Timeline` |
| `/tasks` **Aufgaben** | ruhige Liste + Detailpane | `TodoView`, `TodoItem`, `FilterMenu` |
| `/log` **Verlauf** | Tag / Woche / Gruppierung; Zeiten prüfen, nachtragen, korrigieren | `TimesheetPanel`, `Timeline`, `SessionDialog` |
| `/review` **Rückblick** | Ergebnisse, Projekte, Aufgaben, Notizen, Reflexion, Folgeaufgabe; Timerstatistik als „Arbeitsweise“ | `ReviewPanel`, `ReviewJournal`, Teile von `StatsView` |
| `/focus` Fokusmodus | große Timeransicht (bestehende `TimerView`/`RingTimer`) | `TimerView` |
| `/settings`, `/help` | Seitenmenü / Überlaufmenü; Import/Export hierher | vorhanden |

Alte Routen `/today`, `/times`, `/stats`, `/todos`, `/habits` → Redirects (Deep Links, Tray, Mini-Fenster, Shortcuts prüfen).
Globale Aktiv-Leiste (`ActiveSessionBar`: Titel · Zeit · Pause/Stop, Tap → `/focus`) in `AppShell`, über Tabbar (Android) bzw. am unteren Rand (Desktop). Nur Snapshot aus `timerStore`, kein Frontend-Timer.
Sync-Status: Zeile aus `AppShell` entfernen, kleines Symbol im Header/Einstellungen, Fehler/Konflikt sichtbar hervorgehoben.
Adaptiv: `useIsWide` bleibt; neu Listen-/Detailmuster (breit nebeneinander, schmal Screen + Zurück mit erhaltenem Filter).

## Etappen (je Ticket/Branch/PR, BDD-first: Feature → rot → grün; Pflicht-Checks laut CLAUDE.md)

### E1 – Heute als Start, vier Bereiche, Review aus Times lösen
- `features/ui/navigation.feature`, `today.feature`, `timetracking.feature` anpassen/erweitern (Szenarien englisch).
- `Navigation.tsx` `NAV_ITEMS`: Heute, Aufgaben, Verlauf, Rückblick (+ Einstellungen/Hilfe unten); Router-Umbau mit Redirects; `/` = Heute.
- `TimesView` splitten: `TimesheetPanel` → `/log`, `ReviewPanel` → `/review`.
- Heute-Leerzustand: „Was möchtest du heute angehen?“ + [Aufgaben auswählen]/[Neue Aufgabe]/Hinweis freie Tätigkeit (i18n `today.*`, de+en).
- Budget-Anzeige trennen: „Geplanter Aufwand x / Budget“ und „Erfasste Arbeit“ (zwei Angaben, Quelle `get_day_plan` + Sessions des Tages). „Heute fällig“ als Hinweis, nicht automatisch im Plan.
- Prüfen: Android 411 px kein horizontaler Überlauf; `tests/steps/mobile.steps.tsx`, `e2e/`.

### E2 – Globale aktive Session, Fokusmodus
- `ActiveSessionBar` in `AppShell`; `/focus` aus `TimerView`; Start-Einstieg A (aus Aufgabe, Modus = zuletzt benutzter), B „Freie Tätigkeit starten“ (Dialog: Tätigkeit, Projekte, Kontexte, Modus) ersetzt `labels.heading` „Assignment without a task“ (`TimerView` ~191–215, `timerStore.start`, `NO_LABELS`).
- Modi Pomodoro/Flowtime/Stopwatch nur im Fokus/Startdialog; optional „Ergebnis / nächste Schritte“ beim Stoppen (Session-`note` existiert schon in `session_meta`).
- Mini-Timer (`/mini`) bleibt, Link „Hauptfenster“ → `/focus`.
- Kein Backend-Umbau nötig (`start_labelled`, Snapshot-`labels`).

### E3 – Taskzeilen vereinfachen, Details auslagern
- `TodoItem`: dauerhaft nur Erledigen, Titel, wichtige Metadaten (+Projekt @Kontext), Starten, „⋯“-Menü, Subtask-Aufklappindikator + Fortschritt „2/4 · Heute · 75 min erfasst“.
- Aktionsmenü: Für heute wählen/aus Plan nehmen, Unteraufgabe, Verschieben/Einrücken, Duplizieren, Löschen.
- Neue `TaskDetail`-Komponente (Beschreibung, lange Links, rohe todo.txt-Zeile, Zeiteinträge, Eigenschaften); Desktop Pane rechts, Android eigene Route `/tasks/:id`, Zurück behält Filter (Filterzustand in `todoStore`/`uiStore`).
- Toolbar: `Aufgaben [+] [⋯]`, Suche, [Eingang ▾] [Filter]; Import/Export ins Überlaufmenü; todo.txt-Schnelleingabe + Rohtexteditor bleiben.
- „Aufgaben auswählen“-Auswahlansicht für Heute (Mehrfachauswahl, setzt `plan_todo`, verschiebt nichts).

### E4 – Android: einspaltig, untere Navigation, Liste→Detail
- `BottomTabs` mit 4 Zielen; Heute auf Android: Tagesfokus + Segmente [Plan][Verlauf][Notizen], vertikal; Verlauf Tagesliste Standard (Wochenmatrix nur Desktop); Rückblick aufklappbare Abschnitte.
- `useIsWide`-Breakpoints prüfen (768 px), ggf. Medium-Rail (Icons) zwischen 600–1024.
- Maestro-Flows `e2e/mobile/` und `features/ui/responsive.feature` aktualisieren; Edge-to-edge/Keyboard-Inset (`MainActivity.kt`) testen.

### E5 – Rückblick auf Ergebnisse ausrichten, Timerstatistik sekundär
- `/review`: Zeitraum-Umschalter (Woche ▾, ‹ ›), Reihenfolge: Ergebnisse/Beiträge (Journal `achievement`), Arbeit nach Projekten (+ „Nicht zugeordnet“), Aufgaben (erledigt/bearbeitet-offen), Notizen aus den Tagen, Reflexion (3 Fragen, `week_note`), Nächster Zeitraum [Aufgabe erstellen].
- `StatsView`-Inhalte (Pomodoros, Streaks, Modi, Charts) → Abschnitt „Arbeitsweise“ im Rückblick (eingeklappt) oder Unterseite; `/stats` Redirect.
- Begriff: „Erfasste Arbeitszeit“ (Summe) vs. „Fokuszeit“ (nur Pomodoro/Flowtime-Teilmenge) – prüfen `lib/stats`, `review.rs`, i18n.
- Wochenrückblick beginnen aus Rückblick; „Tag abschließen“ im Tagesmenü der Heute-Ansicht (`close_day` bleibt, Notiz jederzeit editierbar, abgeschlossener Tag korrigierbar).

### E6 – Habit-Tracker komplett entfernen, Routinen integrieren
Frontend: `HabitsView`, `HabitForm`, `HabitRow`, `habitStore`, `lib/habits.ts`(+Test), i18n `habits.*` (46), Habit-Teile in `TodayView`/`ReviewPanel`, Route/Links, `tests/helpers/fakeBackend.ts`, `tests/steps/common.steps.tsx:573-600`.
Backend: `nunc-core/habit.rs`, Habit-Felder in `crdt.rs`/`sync_document.rs`, Review-Anteil in `review.rs`, `nunc-store/habit_repo.rs`, `crdt_repo.rs` (≈478, 611), `commands/habits.rs` + Registrierung `lib.rs:66-70`, `commands/review.rs`-Anteile, `src/bindings.ts` (`cargo test -p nunc export_bindings`).
Daten: **neue additive Migration 016** (`DROP habits, habit_field_clock, habit_logs`); 015 bleibt unverändert. Vorher einmaliger **Export** (JSON/Markdown im Backup-/Hilfebereich oder Hinweis auf Backup), damit „nicht kommentarlos entfernt“ (PDF §8). Backups mit Habits bleiben restorebar (Migrationen laufen nach Restore).
Sync: `SCHEMA_VERSION` 9 → 10 (`sync_document.rs:24`), Felder `habits`/`habit_logs` beim Lesen ≤9 ignorieren (`MIN_SCHEMA_VERSION` bleibt); ADR-Folge nötig (siehe unten). Ältere Clients lehnen Format 10 ab – in Release Notes.
Features löschen: `features/habits/*`, `features/sync/habit_*.feature`, `features/ui/habits.feature`, Rust-BDD `tests/bdd/habits.rs` (core+store).
Routinen: „Tag planen“ (Heute), „Tag abschließen“ (Tagesmenü), „Wochenrückblick beginnen“ (Rückblick); wiederkehrende konkrete Arbeit als Aufgabe mit Wiederholung (`rec:` in todo.txt) – **offen**: existiert Wiederholungs-Unterstützung? Falls nicht, nur Hinweis/Backlog, kein neues Subsystem.

## Docs/ADRs
- Neuer ADR `0031-ui-informationsarchitektur` (vier Bereiche, Fokusmodus, adaptives Muster); ADR `0032-habits-entfernt` (Supersedes 0028, Migration 016, Format 10).
- `docs/adr/0025`/`0027` Verweise anpassen; `roadmap.md` (Journal/Habits/Tagesplaner veraltet) aktualisieren; PLAN.md unverändert (historisch).
- i18n: nur `de.ts`/`en.ts`, ungenutzte Schlüssel je Etappe entfernen (`nav.*`, `habits.*`, `labels.heading`).

## Kritische Risiken
- Sync-Format-Bruch (E6): Reihenfolge – erst alle Geräte updaten; Test `features/sync` für Format 9 → 10 (Lesen alter Dokumente).
- Tray/Mini-Fenster/Shortcuts (`useShortcuts`) referenzieren alte Routen.
- Pull-to-Refresh/Keyboard-Inset in neuer Shell; Aktiv-Leiste + Tabbar überlappen auf Android.
- Fachlogik (Budget-Trennung, Fokus-/Gesamtzeit) gehört nach `nunc-core`, nicht in Komponenten; abgeleitete Zahlen nie speichern. Mutationstests für geänderte `nunc-core`/`nunc-store`.

## Verifikation (je Etappe)
```
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run typecheck && npm run lint && npm run test && npm run test:bdd
npm run docs:report
npm run tauri build -- --debug --no-bundle
cargo mutants --in-diff /tmp/pr.diff -p nunc-core -p nunc-store -p nunc-sync   # bei Fachlogik (E1, E5, E6)
```
Manuell: Desktop (breit + <768 px) und Android-Emulator 411 px (`npm run tauri android build -- --debug --apk --target aarch64`), Flow: Heute → Aufgaben auswählen → Start → Leiste → Fokus → Stop → Verlauf → Rückblick. E6: Restore eines Backups mit Habits, Sync Format 9-Datei einlesen.

## Reihenfolge der PRs
E1 → E2 → E3 → E4 → E5 → E6 (E6 kann unabhängig nach E1). Erster Prototyp (laut PDF): E1 + Aktiv-Leiste aus E2, je schmal (Android) und Desktop.
