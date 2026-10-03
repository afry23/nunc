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

**Exit-Kriterien M1:** `cargo test -p nunc-core` grün, Coverage ≥ 90 % (`cargo llvm-cov`).

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

**Exit-Kriterien M2:** Backup aus laufender Go-App lässt sich per Test restoren; Coverage `nunc-store` ≥ 80 %.

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

---

## M4 – Frontend Kern / Desktop-MVP (3 Wochen)

**Ziel:** Bedienbare Desktop-App, funktional gleichwertig zur Go-App ohne Sync. Visuelle Referenz: `../nunc-go/screens/*`.

| ID | Ticket | Details | Features |
|---|---|---|---|
| M4-01 | App-Shell & Layout | Responsive: ≥ 768 px Sidebar, darunter Bottom-Tab-Bar; Routen Timer/Todos/Stats/Settings; Safe-Areas; Dark-Theme-Tokens; „Breath“-Animation | `ui/navigation.feature`, `ui/responsive.feature` |
| M4-02 | Stores | Zustand: `timerStore` (nur Snapshot aus Events), `todoStore`, `settingsStore`, `uiStore`; Reload bei `app:reload` | – |
| M4-03 | `RingTimer` | reines SVG, Farben je Modus/Typ, Countdown/Stoppuhr, `MM:SS`, Zustände (Pulsieren), ARIA live region | `timer/pomodoro.feature` (@ui) |
| M4-04 | Timer-View | Todo-Auswahl, Start/Pause/Resume/Stop, Break-Vorschlag-Dialog, Flowtime-Pausenempfehlung | `timer/*.feature` (@ui) |
| M4-05 | Sound | Web Audio, `.mp3`-Assets (Glocke, Ding, Digital), Lautstärke, Trigger auf `timer:complete` | – |
| M4-06 | Todo-View | Liste mit Prio/Tags/🍅-Zähler, Inline-Add/Edit (todo.txt-Syntax), Abhaken, Löschen, Filter, Suche, Sortierung, Reorder (Drag & Drop, touch-fähig: `dnd-kit`) | `todos/*.feature` (@ui) |
| M4-07 | Stats-View | Tag/Woche/Monat/Jahr; eigene SVG-Charts: Bar, Line, Heatmap-Kalender, Pie; Top-Todos, Projekt-/Kontext-Verteilung, Durchschnitt 7/30 T., Streaks; Export-Button | `stats/*.feature` (@ui) |
| M4-08 | Settings-View | Timer, Flowtime (Slider 0.1–0.5 / 0.05, Anker STRICT/BALANCED/RELAXED, Badge, Info-Banner), Min-Flow-State 1–60, Notifications, Sound, Tray | `settings.feature` (@ui) |
| M4-09 | Keyboard-Shortcuts | Space, Esc, N, 1/2/3, `,` – nur Desktop, nicht in Eingabefeldern | `ui/shortcuts.feature` |
| M4-10 | Mini-Timer | eigene Route im 2. Fenster, kompakter Ring | – |
| M4-11 | Accessibility-Pass | ARIA-Labels, Fokus-Reihenfolge, Kontrast (DESIGN.md-Farben prüfen) | – |
| M4-12 | Desktop-E2E-Smoke | WebdriverIO + `tauri-driver` + Cucumber (Linux/Windows-CI): Start → Todo anlegen → Pomodoro (Test-Dauer via Env `NUNC_TEST_FAST_TIMER`) → Stats zeigt 1 | `@e2e @desktop` |

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
