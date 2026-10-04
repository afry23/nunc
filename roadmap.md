# Roadmap – nunc (Tauri 2 · Rust · React/TS · BDD · Desktop + Mobile)

> Grundlage: [`PLAN.md`](./PLAN.md) (Architektur, BDD-Strategie, Risiken).
> Referenz-Implementierung: `../nunc-go/` (Go/Wails) – Fachlogik, Tests und Migrationen werden daraus übernommen.
> Stand: 2026-10-03

---

## Ausgangslage (bestehendes Gerüst)

`nunc/` wurde mit `create-tauri-app` (React + TypeScript) erzeugt:

| Vorhanden | Stand | Anpassung nötig |
|---|---|---|
| `package.json` | React 19, Vite 8, TypeScript 6, `@tauri-apps/api` 2, `plugin-opener` | Tests, Lint, Tailwind, Router, State ergänzen |
| `src/` | Template (`App.tsx` mit `greet`) | durch App-Shell ersetzen |
| `src-tauri/` | Einzel-Crate `nunc` / Lib `nunc_lib`, `greet`-Command, Capability `default.json` | in Cargo-Workspace einhängen, Template-Code entfernen |
| `tauri.conf.json` | `identifier: com.afry.nunc`, Fenster 800×600, `csp: null` | Fenstergröße, CSP, Mini-Timer-Fenster, Bundle-Metadaten |
| Mobile | **nicht initialisiert** (kein `gen/android`, `gen/apple`) | `tauri android init`, `tauri ios init` |
| Git | `Initial commit` + untracked Template | Gerüst als erster echter Commit |

**Wichtig – Datenpfad:** Tauri leitet `app_config_dir()` vom Identifier ab (`~/Library/Application Support/com.afry.nunc`), die Go-App nutzt `…/nunc/`. → Ticket **M2-07** übernimmt beim ersten Start die bestehende `nunc.db` vom alten Pfad (Kopie, Original bleibt).

---

## Überblick

| Meilenstein | Inhalt | Dauer | Ergebnis |
|---|---|---|---|
| **M0** | Fundament & Toolchain | 1,5 Wo. | Workspace, BDD-Runner, CI, Android/iOS bauen |
| **M1** | Domain-Kern (`nunc-core`) | 2 Wo. | gesamte Fachlogik pure + `@domain` grün |
| **M2** | Persistenz (`nunc-store`) | 1,5 Wo. | Go-kompatible DB, Backup/Restore, `@integration` grün |
| **M3** | Tauri-Runtime & Commands | 1,5 Wo. | Timer läuft im Backend, alle Commands typisiert |
| **M4** | Frontend Kern (MVP) | 3 Wo. | **Desktop-MVP:** Timer, Todos, Stats, Settings |
| **M5** | Mobile-Basis | 1,5 Wo. | **Mobile-MVP** auf Android + iOS |
| **M6** | Backup/Restore-UI + WebDAV-Sync | 2 Wo. | Daten-Sicherheit + Sync auf allen Plattformen |
| **M7** | Google Drive + Todoist (Desktop) | 2 Wo. | Feature-Parität zur Go-App |
| **M8** | Härtung & Release 1.0 | 2 Wo. | signierte Builds, Stores/TestFlight |

Gesamt ≈ **17,5 Wochen** (1 Person). Meilensteine sind sequenziell; innerhalb eines Meilensteins sind Tickets ohne Abhängigkeit parallelisierbar.

```
M0 ──► M1 ──► M2 ──► M3 ──► M4 ──► M5 ──► M6 ──► M7 ──► M8
        │                    ▲
        └── Spikes (M0-09/10) beeinflussen M3 (Timer) und M6 (SecretStore)
```

---

## Arbeitsweise (gilt für jedes Ticket)

1. **Feature zuerst:** Szenarien in `features/**.feature` (englisch, Tag `@wip` + Story-Tag, z. B. `@US-B01`).
2. **Rot:** Step-Definitions schreiben, Runner schlägt fehl.
3. **Grün von innen nach außen:** `nunc-core` → `nunc-store`/`nunc-sync` → Command → UI.
4. `@wip` entfernen, Pflicht-Checks ausführen, Commit (Conventional Commits).

**Definition of Done (jedes Ticket):**
- [ ] zugehörige Szenarien grün, keine `undefined`/`pending` Steps
- [ ] `cargo fmt --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace`
- [ ] `npm run typecheck` · `npm run lint` · `npm run test`
- [ ] `npm run tauri build` ohne Warnungen (ab M3)
- [ ] keine Schema-Änderung ohne neue, additive Migration
- [ ] Fachlogik-Tickets (M1–M2, M6–M7): `cargo mutants --in-diff` ohne unbegründet überlebende Mutanten (ADR 0007)

---

## M0 – Fundament & Toolchain (1,5 Wochen)

**Ziel:** Leeres, aber vollständig getestetes und baubares Projekt auf allen fünf Plattformen.

| ID | Ticket | Details | Abh. |
|---|---|---|---|
| M0-01 | Gerüst committen | Template-Stand + `PLAN.md` + `roadmap.md` als Basis-Commit; `.gitignore` um `src-tauri/gen/android/**/build`, `*.db`, Coverage-Ordner ergänzen | – |
| M0-02 | Cargo-Workspace | Root-`Cargo.toml` mit `members = ["src-tauri", "crates/*"]`, `[workspace.dependencies]` (serde, thiserror, chrono/time, rusqlite, reqwest, tokio); Crates anlegen: `crates/nunc-core`, `nunc-store`, `nunc-sync`, `nunc-platform`; `src-tauri/Cargo.toml` auf Workspace-Deps umstellen, Metadaten (`description`, `authors`, `edition = "2024"`) | M0-01 |
| M0-03 | Template aufräumen | `greet`, `App.css`, `react.svg`, `vite.svg`/`tauri.svg` entfernen; `index.html`-Titel `nunc`, `lang` | M0-01 |
| M0-04 | Frontend-Toolchain | Tailwind v4 (`@tailwindcss/vite`), Fonts Manrope/Inter lokal, Design-Tokens aus `../nunc-go/DESIGN.md` als CSS-Variablen; TanStack Router; Zustand; ESLint (typescript-eslint strict, react-hooks) + Prettier; `@types/node` (entfernt `@ts-expect-error` in `vite.config.ts`); Scripts `typecheck`, `lint`, `test`, `test:bdd` | M0-03 |
| M0-05 | Typ-Bridge | `tauri-specta` + `specta` einrichten, Export nach `src/bindings.ts` im Debug-Build; Wrapper `src/lib/api.ts`; Lint-Regel: kein direktes `invoke` außerhalb `src/lib/` | M0-02, M0-04 |
| M0-06 | BDD-Runner Rust | `cucumber` crate in `src-tauri/tests/bdd.rs` bzw. `crates/nunc-core/tests/bdd.rs` (`harness = false`), liest `../../features`, Tag-Filter `@domain`/`@integration`; Dummy-Feature grün | M0-02 |
| M0-07 | BDD-Runner Frontend | Vitest + `@testing-library/react` + `quickpickle`, Tag-Filter `@ui`; `mockIPC` aus `@tauri-apps/api/mocks` als Test-Helper; Dummy-Feature grün | M0-04 |
| M0-08 | Mobile initialisieren | `npm run tauri android init`, `npm run tauri ios init`; `TAURI_DEV_HOST`-Setup prüfen (bereits in `vite.config.ts`); Capabilities splitten: `desktop.json` (`platforms: [macOS, windows, linux]`), `mobile.json` (`[android, iOS]`); Template-App läuft im Emulator/Simulator | M0-03 |
| M0-09 | **Spike: Hintergrund-Timer** (Timebox 2 T.) | Prototyp: Zeitstempel-Timer + `tauri-plugin-notification` geplante Notification; App in Hintergrund → Notification pünktlich? Resume → korrekter Stand? Android Exact-Alarm-Permission prüfen. Ergebnis als ADR | M0-08 |
| M0-10 | **Spike: SecretStore** (Timebox 2 T.) | Desktop `keyring`; iOS `keyring` (apple-native); Android: `android-native-keyring-store` (statt eigenem Kotlin-Plugin). Go-Keychain-Einträge müssen **nicht** gelesen werden (Neustart der Konfiguration, siehe ADR 0005). Ergebnis als ADR | M0-08 |
| M0-11 | CI | Workflow (Gitea/GitHub Actions): Lint + Tests (Linux), Build-Matrix macOS/Windows/Linux, Android-APK (Debug), iOS-Build (macOS-Runner, unsigned); Artefakt: Cucumber-Report | M0-06, M0-07 |
| M0-12 | Projekt-Doku | `CLAUDE.md` (Architekturregeln, BDD-Workflow, Pflicht-Checks, „Do Not“-Liste aus Go-Projekt adaptiert), `docs/adr/0001…` (Tauri 2, rusqlite, specta, Timer-Modell, SecretStore, BDD-Toolchain) | M0-09, M0-10 |
| M0-13 | Feature-Katalog | Alle Features aus `PLAN.md §2.1` als Dateien mit Szenario-Titeln anlegen (`@wip`), Story-Tags; Go-Fixtures erzeugen: `tests/fixtures/go-v5.db`, `nunc-sync.json`, WebDAV-Sync-Datei | M0-06 |
| M0-14 | Mutation-Tests (Rust) | `cargo-mutants` für `nunc-core`/`nunc-store`/`nunc-sync` (`.cargo/mutants.toml`), CI-Workflow `mutation.yml` (PR: `--in-diff`, wöchentlich voll), `npm run mutate:rust`, ADR 0007. TypeScript/Stryker zurückgestellt: nicht kompatibel mit Vitest 5 | M0-06 |

**Exit-Kriterien M0:** CI grün; `tauri dev` auf macOS, Android-Emulator, iOS-Simulator startet leere Shell; beide Spikes entschieden.

---

## M1 – Domain-Kern `nunc-core` (2 Wochen)

**Ziel:** Gesamte Fachlogik als pure Rust-Funktionen, injizierte `Clock`, keine I/O. Alle `@domain`-Szenarien grün.

| ID | Ticket | Go-Referenz (`../nunc-go/internal/…`) | Features |
|---|---|---|---|
| M1-01 | Domain-Typen | `models/*.go` (Todo, Session, Settings + `DefaultSettings`, TimerMode/State/SessionType, Stats) | – |
| M1-02 | todo.txt Parser/Serializer | `services/todos.go`, `todos_parse_test.go` | `todos/todotxt_parsing.feature` |
| M1-03 | Filter & Sortierung | `services/todos.go` | `todos/filter_sort.feature` |
| M1-04 | Timer-Zustandsautomat | `services/timer.go`, `timer_test.go`, `timer_pure_test.go` | `timer/pomodoro.feature` |
| | – Start/Pause/Resume/Stop, Zeit über Zeitstempel-Segmente | | |
| | – 50 %-Regel (`aborted`/`completed`), Break vorzeitig → `skipped` | | |
| | – Long-Break nach N Pomodoros, Auto-Start-Flags | | |
| M1-05 | Flowtime-Pausenberechnung | `timer.go` `CalculateBreakDuration`, `STORIES.md US-FT-02` | `timer/flowtime.feature` |
| M1-06 | Statistik & Streaks | `services/stats.go`, `stats_test.go`, `stats_streaks_test.go` | `stats/aggregation.feature`, `stats/streaks.feature` |
| M1-07 | Export CSV/JSON | `app.go ExportStats`, REQUIREMENTS §11 | `stats/export.feature` |
| M1-08 | Sync-Merge | `services/sync_merge.go`, `sync_webdav_merge.go` + Tests | `sync/merge.feature` |
| | – Todos LWW über `updated_at`, Soft-Delete; Sessions append-only (`device_id + started_at`) | | |
| M1-09 | Todoist-Mapping & MergeTodos | `services/todoist/mapping.go`, `mapping_test.go` | `todoist/mapping.feature` |
| M1-10 | Backup-Policy | `services/backup.go`, `backup_rotate_test.go`, `backup_status_test.go` | `backup/rotation.feature` |
| M1-11 | Property-Tests | `proptest`: Parser-Roundtrip, Merge idempotent/kommutativ | – |

**Exit-Kriterien M1:** `cargo test -p nunc-core` grün, Coverage ≥ 90 % (`cargo llvm-cov`), Mutation-Score ≥ 80 % (`cargo mutants -p nunc-core`).

---

## M2 – Persistenz `nunc-store` (1,5 Wochen)

**Ziel:** SQLite 100 % kompatibel zur Go-App. `@integration`-Szenarien (In-Memory-DB) grün.

| ID | Ticket | Details | Features |
|---|---|---|---|
| M2-01 | Verbindung & Migrationsrunner | `rusqlite` bundled, WAL, Foreign Keys; Runner kompatibel zu `schema_migrations` aus `nunc-go/internal/db/db.go`; Migrationen `001`–`005` **unverändert** kopieren (`include_str!`) | `compat_go.feature` |
| M2-02 | Migration 006 `active_timer` | additive Tabelle für persistierten Timer-State | `timer/persistence.feature` |
| M2-03 | Settings-Repo | Key-Value, typisierte Defaults, gleiche Keys wie Go (`services/settings.go`, `settings_test.go`) | – |
| M2-04 | Todo- & Session-Repo | CRUD, Reorder (`position`), `pomodoro_count`, `sync_id`, `updated_at`, `deleted`, `sync_device_id`; Transaktionen bei Mehrschritt-Writes | `todos/crud.feature` |
| M2-05 | Stats-Queries | Rohdaten aus `sessions` → `nunc-core::stats` | – |
| M2-06 | Backup & Restore | `VACUUM INTO`, 1×/Tag, Rotation 7, `before-restore-YYYY-MM-DD.db`, Validierung (SQLite + Schema-Version), atomarer Swap, Rollback bei Fehler | `backup/daily_backup.feature`, `backup/restore.feature` |
| M2-07 | Legacy-Pfad-Übernahme | erster Start: `…/nunc/nunc.db` (Go) erkennen → in `app_config_dir` kopieren, Backup-Ordner-Setting übernehmen | `compat_go.feature` |
| M2-08 | Kompatibilitäts-Szenarien | Go-Fixture-DB öffnen/restoren; Go-Backups restorebar; Rust-DB von Go lesbar (manueller Check mit `nunc-go`) | `compat_go.feature` |

**Exit-Kriterien M2:** Backup aus laufender Go-App lässt sich per Test restoren; Coverage `nunc-store` ≥ 80 %, Mutation-Score ≥ 70 %.

---

## M3 – Tauri-Runtime & Commands (1,5 Wochen)

**Ziel:** Backend vollständig über typisierte Commands/Events erreichbar; Timer läuft serverseitig.

| ID | Ticket | Details |
|---|---|---|
| M3-01 | App-State & Lifecycle | `AppState` (DB, Services) via `manage()`; Start: Migrationen → Legacy-Übernahme → Auto-Backup (Hintergrund) → Timer-Recovery; Shutdown-Hook für späteren Sync |
| M3-02 | Plattform-Adapter | `nunc-platform`-Traits implementieren: `SystemClock`, `TauriEventSink`, `TauriNotifier` (`tauri-plugin-notification`), `SecretStore` (gemäß ADR aus M0-10) |
| M3-03 | Commands Settings/Todos | `get_settings`, `save_settings`, `get_todos`, `add_todo`, `update_todo`, `delete_todo`, `set_todo_completed`, `reorder_todos` – Typen camelCase wie Go-JSON-Tags |
| M3-04 | Timer-Runtime | `start_timer`, `start_break`, `pause_timer`, `resume_timer`, `stop_timer`, `get_timer_tick`; tokio-Interval → Event `timer:tick` (nur Vordergrund); `timer:complete`, `timer:stopped`; geplante Notification bei Start/Resume, Cancel bei Pause/Stop; Resume-Neuberechnung (`RunEvent::Resumed`) |
| M3-05 | Commands Stats/Export | `get_stats`, `export_stats`, `save_export_to_file` (`tauri-plugin-dialog` + `fs`) |
| M3-06 | Commands Backup | `get_backup_status`, `trigger_backup`, `list_backups`, `restore_from_backup`, `select_backup_dir` (Desktop), `select_restore_file`; Event `app:reload` |
| M3-07 | Desktop-Tray | `#[cfg(desktop)]`: Icon idle/rot/grün, Menü Anzeigen/Verbergen, Start/Pause/Stop, Status-Zeile, Beenden; Fenster schließen → in Tray |
| M3-08 | Fenster & Instanz | Hauptfenster-Größe/Min-Größe, Mini-Timer-WebviewWindow (always-on-top), `tauri-plugin-single-instance`, CSP setzen |
| M3-09 | Command-Integrationstests | Rust-Tests mit `tauri::test::mock_builder`; Event-Reihenfolge Timer verifiziert |

**Exit-Kriterien M3:** `src/bindings.ts` generiert, alle Commands aus `nunc-go/app.go` (außer Sync/Todoist) abgedeckt.

**Stand:** umgesetzt. Abweichungen und Ergänzungen:

- Die Tauri-Adapter (`TauriEventSink`, `TauriNotifier`) liegen in `src-tauri/src/adapters.rs`, nicht in `nunc-platform`
  (sie brauchen `AppHandle`); `nunc-platform` enthält die Traits, `SystemClock`, den Keyring-`SecretStore` und Fakes.
- Zusätzliche Commands: `get_suggested_break`, `show_main_window`. `save_export_to_file` und `select_*` öffnen
  native Dialoge und sind nicht per Mock-Runtime testbar (manuell prüfen).
- Nicht übernommen: `GetFlowtimeRules`/`SaveFlowtimeRules` (Bucket-Modus, siehe Backlog), `OpenURL` (opener-Plugin).
- `save_settings` behält `lastBackupDate` und `backupDir` aus der Datenbank (Besitzer: Backup-Dienst bzw.
  `select_backup_dir`); ein leeres `backupDir` setzt den Standardordner zurück.
- Der Mini-Timer lädt die Route `/mini` und entsteht beim ersten Minimieren während eines Timers (M4-10 liefert die Route).
- Backups liegen standardmäßig in `<app_config_dir>/backups` (die Go-App nutzte `<config>/nunc/backups`, alte Backups
  werden nicht verschoben).

---

## M4 – Frontend Kern / Desktop-MVP (3 Wochen)

**Ziel:** Bedienbare Desktop-App, funktional gleichwertig zur Go-App ohne Sync. Visuelle Referenz: `../nunc-go/screens/*`.

| ID | Ticket | Details | Features |
|---|---|---|---|
| M4-01 | App-Shell & Layout | Responsive: ≥ 768 px Sidebar, darunter Bottom-Tab-Bar; Routen Timer/Todos/Stats/Settings; Safe-Areas; Dark-Theme-Tokens; „Breath“-Animation | `ui/navigation.feature`, `ui/responsive.feature` |
| M4-02 | Stores | Zustand: `timerStore` (nur Snapshot aus Events), `todoStore`, `settingsStore`, `uiStore`; Reload bei `app:reload` | – |
| M4-03 | `RingTimer` | reines SVG, Farben je Modus/Typ, Countdown/Stoppuhr, `MM:SS`, Zustände (Pulsieren), ARIA live region | `ui/timer.feature` |
| M4-04 | Timer-View | Todo-Auswahl, Start/Pause/Resume/Stop, Break-Vorschlag-Dialog, Flowtime-Pausenempfehlung | `ui/timer.feature`, `ui/flowtime.feature` |
| M4-05 | Sound | Web Audio, `.mp3`-Assets (Glocke, Ding, Digital), Lautstärke, Trigger auf `timer:complete` | – |
| M4-06 | Todo-View | Liste mit Prio/Tags/🍅-Zähler, Inline-Add/Edit (todo.txt-Syntax), Abhaken, Löschen, Filter, Suche, Sortierung, Reorder (Drag & Drop, touch-fähig: `dnd-kit`) | `ui/todos.feature` |
| M4-07 | Stats-View | Tag/Woche/Monat/Jahr; eigene SVG-Charts: Bar, Line, Heatmap-Kalender, Pie; Top-Todos, Projekt-/Kontext-Verteilung, Durchschnitt 7/30 T., Streaks; Export-Button | `ui/stats.feature` |
| M4-08 | Settings-View | Timer, Flowtime (Slider 0.1–0.5 / 0.05, Anker STRICT/BALANCED/RELAXED, Badge, Info-Banner), Min-Flow-State 1–60, Notifications, Sound, Tray | `ui/settings.feature` |
| M4-09 | Keyboard-Shortcuts | Space, Esc, N, 1/2/3, `,` – nur Desktop, nicht in Eingabefeldern | `ui/shortcuts.feature` |
| M4-10 | Mini-Timer | eigene Route im 2. Fenster, kompakter Ring | – |
| M4-11 | Accessibility-Pass | ARIA-Labels, Fokus-Reihenfolge, Kontrast (DESIGN.md-Farben prüfen) | – |
| M4-12 | Desktop-E2E-Smoke | WebdriverIO + `tauri-driver` + Cucumber (Linux/Windows-CI): Start → Todo anlegen → Pomodoro (Test-Dauer via Env `NUNC_TEST_FAST_TIMER`) → Stats zeigt 1 | `@e2e @desktop` |

**Stand:** umgesetzt (Desktop-E2E läuft auf Linux-CI, Windows folgt). Abweichungen und Ergänzungen:

- Die `@ui`-Szenarien liegen gesammelt in `features/ui/*.feature` (`timer`, `flowtime`, `todos`, `stats`, `settings`,
  `navigation`, `responsive`, `shortcuts`), nicht mehr in den Domänen-Dateien: deren `Background`-Schritte gehören zu den
  Rust-Runnern. `@mobile`-Szenarien bleiben `@wip` (M5).
- Töne sind synthetisiert (Web Audio: Glocke, Ding, Digital), es gibt keine `.mp3`-Assets.
- Drag & Drop mit `dnd-kit` (Maus, Touch, Tastatur); zusätzlich „Nach oben/unten“-Schaltflächen als barrierefreie Alternative.
- Das Filtern und Sortieren der Todos läuft über `get_todos` (Kern), nicht im React-Code.
- `npm run dev` im Browser startet ein Fake-Backend mit Beispieldaten (`src/dev/preview.ts`, nur Dev-Build).
- Der Mini-Timer ist die Route `/mini`; „Zur App“ ruft `show_main_window`.
- E2E: `npm run test:e2e` (WebdriverIO + `tauri-driver`, Linux/Windows; Linux-CI-Job `e2e-desktop`). `NUNC_TEST_FAST_TIMER=<Sekunden>` verkürzt den Pomodoro, wirkt nur in Debug-Builds.
- Nicht in M4: Backup-/Restore-UI und Sync-Einstellungen (M6), Kontrast-/Screenreader-Prüfung mit echten Tools (M8).

**Exit-Kriterien M4 = Desktop-MVP:** Alltagsnutzung auf Desktop möglich (Dogfooding startet, Daten parallel zur Go-App über gemeinsame DB-Kopie).

---

## M5 – Mobile-Basis / Mobile-MVP (1,5 Wochen)

**Ziel:** Gleicher Funktionsumfang wie M4 auf Android + iOS.

| ID | Ticket | Details |
|---|---|---|
| M5-01 | Plattform-Gating | `@tauri-apps/plugin-os`: Tray-/Shortcut-/Mini-Timer-/Backup-Ordner-Settings auf Mobile ausblenden; Rust-Seite `#[cfg(mobile)]` |
| M5-02 | Touch-UX | Touch-Targets ≥ 44 px, Swipe-to-complete, Pull-to-refresh (später für Sync), Tastatur-Overlap bei Todo-Eingabe |
| M5-03 | Notifications Mobile | Permission-Flow (iOS + Android 13+), Notification-Channel Android, Exact-Alarm-Handling (Android 12+) mit Fallback-Hinweis |
| M5-04 | Lifecycle | Resume → Timer-Neuberechnung + Store-Refresh; Android Back-Button (Navigation statt App-Ende) |
| M5-05 | Assets | App-Icons (`tauri icon` aus `../nunc-go/appicon.png`), Splash, Statusbar-Farbe |
| M5-06 | Mobile-E2E-Smoke | Maestro-Flows: Start, Todo, Timer starten, App in Hintergrund (`maestro` background/foreground), Notification, Resume-Stand |

**Stand:** umgesetzt (auf dem Android-Emulator geprüft; echte Geräte und iOS stehen aus). Abweichungen und Ergänzungen:

- M5-01: `plugin-os` entscheidet über „mobil“ (`src/lib/platform.ts`), der User-Agent ist nur Fallback (Browser, Tests). Tray, Tastenkürzel und Mini-Timer
  gab es mobil schon nicht (`#[cfg(desktop)]`); die Backup-Ordner-Einstellung kommt mit M6 und ist dort zu gaten.
- M5-02: Touch-Ziele ≥ 44 px per Klassen und globaler `pointer: coarse`-Regel; Swipe nach rechts erledigt eine Aufgabe, Pull-to-refresh lädt
  alle Stores neu (Grundlage für Sync), Tastatur über Visual Viewport (iOS) und IME-Insets in `MainActivity.kt` (Android, Edge-to-edge).
  Reorder per Touch: Griff (dnd-kit `TouchSensor`) und „Nach oben/unten“-Schaltflächen; das Ziehen selbst ist in jsdom nicht testbar.
- M5-03: Commands `get_notification_status` / `request_notification_permission`; Anfrage vor dem ersten Timer-Start (blockiert ihn nie),
  Hinweis bei Ablehnung und, auf Android, wenn Exact-Alarms fehlen (`NuncAndroid`-Brücke in `MainActivity.kt`). Manifest: `USE_EXACT_ALARM`
  (ab 13) und `SCHEDULE_EXACT_ALARM` (nur ≤ 12). Kanal `timer` (hohe Wichtigkeit). **Offen:** iOS-Zeitzonen-Bug des Plugins (ADR 0004) bleibt beim Offset-Workaround.
- M5-04: Rückkehr aus dem Hintergrund (`visibilitychange`) lädt Timer, Todos, Einstellungen und Berechtigung neu; Rust rechnet bei `RunEvent::Resumed` neu.
  Zurück-Taste: WebView-Verlauf (Wry) plus Zurück-Stapel mit dem Timer als Startseite (Tabs ersetzen sich, „Timer“ geht zurück).
- M5-05: Icons aus `nunc-go/appicon.png` (`tauri icon`); Android-Vordergrund ohne Schriftzug (Sicherheitszone), dunkles Splash/Theme, helle
  Statusleisten-Symbole, iOS: dunkler Launch-Screen, `Info.ios.plist` (Dark, helle Statusleiste), Fensterfarbe `#131313`.
- M5-06: Maestro-Flows in `e2e/mobile/` (`npm run test:mobile`), Szenarien in `features/e2e/mobile.feature`. **Nicht ausgeführt** (Maestro nicht installiert).

**Exit-Kriterien M5 = Mobile-MVP:** Debug-Builds laufen stabil auf echtem Android- und iOS-Gerät; Timer überlebt Hintergrund + App-Kill.

---

## M6 – Backup/Restore-UI + WebDAV-Sync (2 Wochen, alle Plattformen)

| ID | Ticket | Details | Features |
|---|---|---|---|
| M6-01 | Backup-UI | Status (letztes Backup, Anzahl, Größe, Warnung > 2 Tage), „Jetzt sichern“ (Spinner, Pfad + Größe), Backup-Ordner wählen (Desktop) | `backup/*.feature` (@ui) |
| M6-02 | Restore-UI | Liste (neueste zuerst), Bestätigungsdialog mit Warntext, „Aus Datei wiederherstellen“ (`.db`), Fehleranzeige | `backup/restore.feature` |
| M6-03 | Mobile Backup-Export/Import | Backups in App-Sandbox; Export über Share-Sheet/Datei-Picker; Import per Datei-Picker | `@mobile` |
| M6-04 | Sync-Provider-Auswahl | `none` / `google_drive` / `webdav`, exklusiv, Bestätigung bei Wechsel, sofort persistiert; auf Mobile nur `none`/`webdav` | `sync/provider.feature` |
| M6-05 | WebDAV-Client | `nunc-sync`: PROPFIND/GET/PUT via `reqwest` (Basic + Digest), ETag/If-Match gegen Lost-Update; Format wie `sync_webdav.go` | `sync/webdav.feature` (@integration, wiremock) |
| M6-06 | WebDAV-Konfiguration | URL, Auth-Methode, User, Remote-Pfad; Passwort im SecretStore; Verbindungstest | `sync/webdav.feature` (@ui) |
| M6-07 | Sync-Orchestrierung | serialisiert (Mutex), Trigger: Start, Beenden (max. 5 s), manuell, nach Todo-Änderung (debounced); Mobile: Start/Resume/Hintergrund; Events `sync:start`/`sync:complete`/`sync:status` | `sync/webdav.feature` |
| M6-08 | Sync-Status-Anzeige | Statusleiste idle/syncing/error, Ergebnis-Hinweis „X Sessions, Y Aufgaben importiert“, Konflikt-Hinweis | – |
| M6-09 | Interop-Test Go ↔ Tauri | WebDAV-Testserver (Docker `rclone serve webdav`): Go-App und Tauri-App syncen abwechselnd → identische Daten | `sync/compat_go.feature` (@e2e) |

**Stand:** umgesetzt (Desktop; Mobil-Export/-Import, Mobil-Sync und die Builds für Android/iOS sind nicht auf einem Gerät geprüft).
Entscheidungen: [ADR 0008](./docs/adr/0008-webdav-sync.md). Abweichungen und Ergänzungen:

- M6-01/02: Backup und Wiederherstellung stehen als Abschnitt „Datensicherung“ in den Einstellungen (`BackupSection`),
  getrennt vom Einstellungsformular (eigene Aktionen, kein „Speichern“). Der Datei-Dialog filtert auf `.db`
  (`RESTORE_EXTENSIONS`, Rust-Test); mobil gibt es keinen Filter, weil iOS/Android `.db` nicht als Dateityp kennen.
- M6-03: `export_backup` kopiert ein Backup über den „Speichern unter“-Dialog (auch `content://`-Ziele); kein Share-Sheet-Plugin.
  `select_restore_file` kopiert Auswahlen ohne Dateipfad in den Cache-Ordner (`import/`). Nur die Rust-Teile sind getestet,
  die Dialoge nicht (manuell auf Android/iOS prüfen).
- M6-04: Google Drive erscheint deaktiviert („folgt in einer späteren Version“) bis M7. Die Wahl wirkt sofort
  (`set_sync_provider`); `save_settings` überschreibt `syncProvider` nicht.
- M6-05/06: Passwort im Tresor unter `nunc` / `webdav`; die Oberfläche bekommt nur `hasPassword`. Eine leere Adresse darf
  gespeichert werden, sonst muss sie `http(s)://host` sein.
- M6-07: Auslöser, Entprellen (3 s), Beenden (5 s), Mobil-Hintergrund über `sync_in_background`; Pull-to-refresh synchronisiert mit.
  Ein pausierter Sync (`syncEnabled = false`) oder kein Anbieter ist ein stilles „nichts zu tun“, ein unvollständig
  eingerichtetes WebDAV ein Fehler im Status.
- M6-08: Statusleiste über dem Inhalt (nur bei gewähltem Anbieter), Ergebnis als Hinweis („X Sessions und Y Aufgaben importiert.“),
  Fehler zusätzlich als Hinweis ohne Dialog. Der Konfliktzähler nennt die ersetzten Todos nicht.
- M6-09: `tests/interop/webdav-go-nunc.sh` (Go-App und nunc abwechselnd, Server `x/net/webdav`, kein Docker). Lief grün;
  die CI führt es nicht aus (braucht `../nunc-go`).
- Mutationstest (`cargo mutants --in-diff`, 152 Mutanten): alle gefangen oder nicht kompilierbar (die CI hatte zwei Überlebende in
  `WebDavClient::send` und `ensure_folder`; dafür gibt es jetzt Szenarien: falscher Auth-Typ = genau eine Anfrage,
  Ordner gleichzeitig angelegt = 405 wird akzeptiert).
- Neu: Tag `@sync` für `@integration`-Szenarien, die im Runner von `nunc-sync` laufen.

**Exit-Kriterien M6:** Zwei Geräte (Desktop + Handy) synchronisieren über WebDAV; Restore auf Mobile funktioniert.

---

## M7 – Google Drive + Todoist (Desktop) (2 Wochen)

| ID | Ticket | Details | Features |
|---|---|---|---|
| M7-01 | OAuth2-PKCE-Loopback | Gemeinsames Modul: lokaler Callback-Server (tokio, 120 s Timeout, Port-Fallback), Browser via `plugin-opener`, Token-Refresh | – |
| M7-02 | Google Drive Client | Scope `drive.file`, Ordner `nunc/`, Datei `nunc-sync.json` (Format = `models/sync.go SyncFile`) | `sync/google_drive.feature` |
| M7-03 | Drive-UI & Flows | Verbinden („Verbunden als …“), Trennen, Pausieren (`syncEnabled`), Import-Dialog auf leerem Gerät, Link „In Drive öffnen“ | `sync/google_drive.feature` (@ui) |
| M7-04 | Todoist-API verifizieren | Prüfen, ob REST v2 noch gültig ist oder auf aktuelle API migriert werden muss; Ergebnis in ADR | – |
| M7-05 | Todoist-Client & Sync | Tasks/Projekte/Labels, Mapping-Tabelle `todoist_mappings`, Close statt Delete, serialisiert, Trigger Start + nach Todo-Änderung | `todoist/sync.feature` |
| M7-06 | Todoist-UI | Verbinden/Trennen (Bestätigung), Konto-Anzeige (E-Mail, Avatar), Status `✓ Todoist: HH:MM` / `⟳` / `⚠` | `todoist/auth.feature` |
| M7-07 | ~~Keychain-Kontinuität~~ | **Entfällt:** Die App ersetzt die Go-App, Konten (Drive, Todoist, WebDAV) werden neu verbunden; alte Go-Keychain-Einträge werden nicht übernommen | – |

**Stand:** umgesetzt (Desktop; Anmeldung mit echten Konten bei Google und Todoist ist nicht getestet – dafür braucht es eine
App-Registrierung, siehe unten). Entscheidungen: [ADR 0009](./docs/adr/0009-drive-todoist.md). Abweichungen und Ergänzungen:

- M7-01: gemeinsames Modul `nunc-sync::oauth` (PKCE, Loopback-Server, Token-Austausch und -Erneuerung, Tresor-Eintrag).
  Rückruf-Ports: Google beliebig, Todoist `42813`–`42817` (alle in der Todoist-App als Redirect-URI eintragen).
  Neu in `nunc-platform`: Trait `UrlOpener` (Adapter auf `tauri-plugin-opener`).
- M7-02/03: Scope nur `drive.file` (die E-Mail kommt über `about.get`, die Go-App brauchte `userinfo.email` zusätzlich).
  Drive kennt kein bedingtes Schreiben: der Lauf prüft vor dem Überschreiben die Änderungszeit und setzt neu an.
  Pausieren ist der Schalter „Synchronisierung aktiv“ (`syncEnabled`), „Leer starten“ schließt nur den Import-Dialog
  (wie in Go; die Daten kommen beim nächsten Abgleich).
- M7-04: Todoist API v1 geprüft (die Go-App nutzte sie schon; REST v2 ist nicht mehr der Stand). **Refresh-Tokens gibt es
  jetzt**, die Annahme der Go-App stimmte nicht mehr; ältere Apps haben unbefristete Tokens (ohne `expires_in`).
- M7-05: Todoist hat einen eigenen Koordinator (Events `todoist:sync_status`, `todoist:sync_complete`), unabhängig vom
  Sync-Anbieter; Auslöser Start, Todo-Änderung, Verbinden. Übernahmen aus Todoist tragen die Änderungszeit der Aufgabe
  (kein Rück-Push); sagt die Todoist-Version dasselbe wie das Todo, wird nur die Zeit angeglichen (kein
  „ersetzt“-Hinweis wegen des Erstellungsdatums). Mappings entstehen sofort nach dem Anlegen (kein Duplikat nach einem
  abgebrochenen Lauf), gelöschte Todos verlieren sie nach dem Schließen.
- M7-06: Abschnitt „Todoist“ in den Einstellungen, Statuszeile `✓ Todoist: HH:MM` / `⟳ Todoist synchronisiert…` /
  `⚠ Todoist-Sync fehlgeschlagen` (Tooltip: Meldung) über dem Inhalt. Fehler erscheinen zusätzlich als Hinweis.
- Zugangsdaten: `NUNC_GOOGLE_CLIENT_ID`, `NUNC_GOOGLE_CLIENT_SECRET`, `NUNC_TODOIST_CLIENT_ID`,
  `NUNC_TODOIST_CLIENT_SECRET` beim Bauen (`option_env!`) oder zur Laufzeit als Umgebungsvariablen; Debug-Builds lesen
  zusätzlich eine `.env` (`.env.example`, auch mit den Go-Namen `GOOGLE_CLIENT_ID` …). Ohne Zugangsdaten ist
  „Verbinden“ deaktiviert.
- **Manuell zu prüfen:** Anmeldung und Abgleich mit echten Konten; Interop mit der Go-App über dasselbe Google-Konto
  (`nunc/nunc-sync.json`).

**Exit-Kriterien M7:** Feature-Parität mit `nunc-go` auf Desktop → Go-App kann abgelöst werden.

---

## M8 – Härtung & Release 1.0 (2 Wochen)

| ID | Ticket | Details |
|---|---|---|
| M8-01 | NFR-Messung | Start < 2 s, RAM < 100 MB (Desktop), Timer-Drift < 1 s/h, kein Datenverlust bei Kill während Timer |
| M8-02 | Fehlerbehandlung | Einheitlicher Fehler-Typ (Rust `thiserror` → serialisiert `{ code, message }`), deutsche UI-Texte, Logging (`tauri-plugin-log`, Rotation) |
| M8-03 | i18n-Grundlage | Texte in Ressourcen (de default, en vorbereitet) – „Sprache: System“ aus REQUIREMENTS §5.4 |
| M8-04 | Desktop-Release | macOS Signing + Notarisierung, Windows MSI/NSIS (Icon), Linux AppImage/deb; optional `tauri-plugin-updater` |
| M8-05 | Android-Release | Signing-Key, AAB, Play-Console interner Test-Track |
| M8-06 | iOS-Release | Provisioning, Push-/Notification-Entitlements, TestFlight |
| M8-07 | Doku | README (Installation, Datenpfade, Backup, Sync), Living Documentation aus Cucumber-Report, Umstiegsanleitung Go → Tauri |
| M8-08 | Abnahme | Alle Features ohne `@wip`; Traceability-Report Story → Szenario → Status vollständig |

**Stand:** umgesetzt, soweit ohne Zertifikate, Konten und Geräte möglich. **Nicht ausgeführt:** Signierung/Notarisierung,
Play-Console- und TestFlight-Upload, Windows-/Mobil-Messung, Maestro. Entscheidungen: ADR [0010](./docs/adr/0010-fehler-und-protokoll.md)
(Fehler, Protokoll), [0011](./docs/adr/0011-i18n.md) (i18n), [0012](./docs/adr/0012-release.md) (Release). Abweichungen und Ergänzungen:

- M8-01: `tests/nfr/measure-desktop.sh` misst Start („Oberfläche geladen“ aus dem Protokoll) und Speicher (macOS `footprint`, Linux PSS)
  des Release-Builds; Ergebnisse und Grenzen in [docs/nfr.md](./docs/nfr.md) (macOS: Start ≈ 0,5 s, Speicher 88–93 MB, knapp unter
  dem Ziel). Neue Szenarien: Timer-Drift über eine Stunde, Timer überlebt Kill ohne sauberes Beenden (`@integration`).
- M8-02: der Fehlertyp `AppError { code, message }` bestand seit M3. Neu: `tauri-plugin-log` (Datei `nunc.log`, 1 MB, fünf
  Dateien), jeder `AppError` wird protokolliert. Fehlertexte in der UI folgen über den `code` der Sprache.
- M8-03: Einstellung `language` (`system`/`de`/`en`, nur nunc, die Go-App ignoriert den Schlüssel), Kataloge `src/i18n/` mit
  331 Schlüsseln, Englisch vollständig. **Offen:** Texte des Backends (Tray, Benachrichtigung, Server-Meldungen) sind deutsch.
- M8-04…06: Bundle-Metadaten, Release-Workflow (`release.yml`), Android-Signierung in `build.gradle.kts`, Versions-Skript
  (`npm run version:set`), Anleitung [docs/release.md](./docs/release.md). Kein Auto-Updater in 1.0. Die Version steht noch auf
  0.1.0; sie wird beim Release gesetzt.
- M8-07: README, [Umstieg Go → nunc](./docs/migration-from-go.md), `npm run docs:report` (Living Documentation + Traceability
  nach `target/docs/`, in der CI als Artefakt).
- M8-08: die 18 übrig gebliebenen `@wip`-Titel waren Dubletten umgesetzter Szenarien (bzw. das Share-Sheet, das es laut M6 nicht
  gibt) und sind entfernt. `npm run docs:report` bricht bei `@wip`, fehlender Ebene oder fehlender Story ab; CI und Release rufen es auf.

**Exit-Kriterien M8 = Release 1.0** auf macOS, Windows, Linux, Android, iOS.

---

## Nach 1.0 (Backlog)

| Thema | Notiz |
|---|---|
| Google Drive + Todoist auf Mobile | `tauri-plugin-deep-link`, Plattform-Client-IDs, Redirect-URIs registrieren |
| Laufender Timer außerhalb der App | Android Ongoing-Notification / Foreground-Service, iOS Live Activity, Home-Screen-Widgets |
| Konfigurierbare Shortcuts, Themes | REQUIREMENTS §9, §14 (v2) |
| `flowtime_rules`-Bucket-Modus | STORIES US-FT-02 „Out of Scope“ |
| Kalender-Integration | REQUIREMENTS §14 |

---

## Nächste konkrete Schritte

1. **M0-01** Gerüst + `PLAN.md` + `roadmap.md` committen.
2. **M0-02/03** Cargo-Workspace + Crates anlegen, Template-Code entfernen.
3. **M0-06** cucumber-rs-Runner mit erstem echten Feature `timer/flowtime.feature` (aus `STORIES.md US-FT-02`) → rot.
4. **M0-08** `tauri android init` / `tauri ios init`, danach Spikes **M0-09** und **M0-10** starten.
