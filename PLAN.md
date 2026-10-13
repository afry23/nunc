# Plan: nunc als Tauri-2-App (Rust + React/TS) mit BDD, Desktop + Mobile

## Context

`nunc-go/` = Pomodoro/Flowtime-App in Go (Wails v3 alpha) + Svelte 3. Funktionen: Timer (Pomodoro/Flowtime), todo.txt-Todos, Statistiken + CSV/JSON-Export, SQLite-Backups/Restore, Sync (Google Drive / WebDAV, exklusiv), Todoist-Bidirektional-Sync, System-Tray, Notifications, Sounds, Mini-Timer.
Ziel: Neuschreiben als **Tauri 2** (Rust-Backend, React + TypeScript-Frontend), lauffähig auf **macOS/Windows/Linux + Android + iOS**, entwickelt **BDD-first** (Gherkin = Spezifikation + Tests).

Entscheidungen (User):
- Gherkin **englisch**
- Mobile v1: **WebDAV-Sync + Backup/Restore**. Google Drive + Todoist v1 nur Desktop (Mobile später, Deep-Link-OAuth).
- **Kein Go-Parallelbetrieb mehr** (Entscheidung 2026-10-07, ADR 0016): Schema 001–005 bleibt für Bestandsdaten, das Sync-Format ist ein eigenes CRDT-Format (`nunc-sync.v2.json`).
- Projektort: dieses Repository (`nunc/`, eigenes Git-Repo). `nunc-go/` bleibt Referenz, unverändert.

Quellen der Wahrheit für Fachlogik: `nunc-go/REQUIREMENTS.md`, `STORIES.md`, `nunc-userstories/*.md`, `DESIGN.md`, `screens/*` (Stitch-Mockups), Go-Tests in `nunc-go/internal/**/_test.go` (Table-Driven → Scenario Outlines). Achtung: `nunc-go/CLAUDE.md` teils veraltet (sagt Wails v2, kennt Sync nicht). Die Go-Quellen sind Referenz, nicht mehr bindend.

---

## 1. Zielarchitektur

### 1.1 Repo-Layout

```
nunc/
├── Cargo.toml                  # Workspace
├── crates/
│   ├── nunc-core/              # PURE Domain, kein I/O, kein tokio
│   │   ├── todotxt.rs          # Parser/Serializer  (← services/todos.go)
│   │   ├── timer.rs            # Zustandsautomat + Zeitberechnung (← services/timer.go)
│   │   ├── flowtime.rs         # calculate_break_duration (← STORIES US-FT-02)
│   │   ├── stats.rs            # Aggregation, Streaks (← services/stats.go)
│   │   ├── export.rs           # CSV/JSON
│   │   ├── sync_merge.rs       # LWW Todos, append-only Sessions (← sync_merge.go, sync_webdav_merge.go)
│   │   ├── todoist_mapping.rs  # MergeTodos, Feld-Mapping (← todoist/mapping.go)
│   │   └── backup_policy.rs    # Rotation (7), "heute schon gesichert?"
│   ├── nunc-store/             # rusqlite (bundled), Repos, Migrationen, VACUUM INTO, Restore-Validierung
│   │   └── migrations/001…005  # 1:1 aus nunc-go/internal/db/migrations kopiert
│   ├── nunc-sync/              # reqwest: Google Drive, WebDAV (PROPFIND/PUT/GET), Todoist REST
│   └── nunc-platform/          # Traits: Clock, SecretStore, Notifier, EventSink, FileSystem paths
├── src-tauri/                  # Tauri-App: Commands, Events, Plugins, Adapter, Tray (#[cfg(desktop)])
│   ├── capabilities/           # desktop.json, mobile.json
│   └── gen/android, gen/apple  # tauri android/ios init
├── src/                        # React-Frontend
├── features/                   # Gherkin – EINE Quelle für alle Runner
└── tests/                      # Rust cucumber runner(s)
```

Prinzip aus Go-Projekt beibehalten: **funktionaler Kern, Seiteneffekte an Rändern**. `nunc-core` = pure functions, dadurch trivial BDD-/unit-testbar. Dependencies per Konstruktor (Traits), kein globaler State.

### 1.2 Backend (Rust)

| Thema | Go heute | Rust/Tauri |
|---|---|---|
| DB | modernc sqlite, WAL, single conn | `rusqlite` (feature `bundled`), WAL, `Mutex<Connection>` in Tauri `State`; Migrationsrunner liest gleiche `schema_migrations`-Tabelle wie `nunc-go/internal/db/db.go` |
| Bindings | Wails + `wailsbridge.ts` | `#[tauri::command]` + **`tauri-specta`** → generiert `src/bindings.ts` (typsicher, Events inkl.) |
| Events | `timer:tick`, `timer:complete`, `timer:stopped`, `app:reload`, `sync:*`, `todoist:*` | gleiche Namen, typisierte specta-Events |
| Timer | Goroutine-Ticker zählt Ticks | **Zeitstempel-basiert** (s. 1.4) + tokio-Interval nur für Tick-Events |
| Notifications | osascript/beeep/notify-send | `tauri-plugin-notification` (Desktop + Mobile, Mobile mit **Schedule**) |
| Keychain | go-keyring | Trait `SecretStore` auf `keyring-core` mit je einem Store pro Plattform (macOS/iOS apple-native, Windows, Linux Secret Service, Android `android-native-keyring-store`, ADR 0005). Go-Einträge werden nicht übernommen |
| OAuth | Loopback 127.0.0.1:42813 + Browser | Desktop: Loopback bleibt (`tauri-plugin-opener` + tokio TCP-Listener, PKCE). Mobile: später `tauri-plugin-deep-link` |
| Dateidialoge | Wails Dialog | `tauri-plugin-dialog` (+ `tauri-plugin-fs`) |
| Tray, Mini-Timer-Fenster, Shortcuts | Wails | `tauri` tray-icon + 2. WebviewWindow, nur `#[cfg(desktop)]`; Shortcuts im Frontend (keydown) |
| Single instance / Autostart | – | `tauri-plugin-single-instance` (Desktop) |
| Pfade | `%APPDATA%/nunc`, `~/Library/Application Support/nunc`, XDG | `app.path().app_config_dir()` — **Identifier so wählen, dass Desktop-Pfad = `…/nunc/`** bzw. beim ersten Start Go-DB vom alten Pfad übernehmen (Migration-Check) |

### 1.3 Frontend (React)

- React 19 + TypeScript (strict, kein `any`) + Vite + Tailwind v4 (Tokens aus `DESIGN.md`: surface-Stufen, primary `#4de082`, Manrope/Inter, "Breath"-Animation).
- Routing: TanStack Router (Views: Timer, Todos, Stats, Settings, Mini-Timer).
- State: **Zustand**-Stores (timer, todos, settings, sync) — Store abonniert specta-Events; Frontend besitzt Timer-State NICHT, rendert nur Snapshot.
- Einziger Backend-Zugang: `src/lib/api.ts` (dünner Wrapper über generierte `bindings.ts`) — Komponenten rufen nie `invoke` direkt.
- Komponenten: `RingTimer` (reines SVG), `TodoList/TodoItem`, `StatsView` mit eigenen SVG-Charts (Bar, Line, Heatmap, Pie), `Settings` (Sections: Timer, Flowtime, Sound, Sync-Provider, WebDAV, Drive, Todoist, Backup/Restore).
- **Responsive/Mobile:** Desktop = Sidebar-Layout; Mobile = Bottom-Tab-Bar, `env(safe-area-inset-*)`, Touch-Targets ≥ 44px, Swipe-to-complete optional. Plattform-Features via `@tauri-apps/plugin-os` `platform()` ein-/ausblenden (Tray-Setting, Backup-Ordner-Wahl, Drive/Todoist auf Mobile ausgeblendet).
- Sounds: Web Audio, Assets `.mp3` (iOS-WebView kann kein `.ogg` zuverlässig → offener Punkt #2 aus REQUIREMENTS damit entschieden).
- Pure Utils (`format.ts`, `todotxt-display.ts`) mit 100 % Coverage.

### 1.4 Kritischer Punkt: Timer auf Mobile

Mobile-OS suspendiert App im Hintergrund → Ticker-Ansatz aus Go bricht.
Lösung:
1. `nunc-core::timer` modelliert Zustand als Daten: `{state, mode, session_type, started_at, run_segments_start, accumulated_ms, planned_sec, todo_id}`. `remaining = f(state, now)` — pure, mit injizierter `Clock`.
2. Aktiver Timer-State wird bei jeder Transition in DB persistiert (neue Migration `006_active_timer.sql`, additiv → Go-kompatibel bleibt, Go ignoriert Tabelle) → erfüllt NFR „Timer-Zustand überlebt App-Neustart“.
3. Bei Start/Resume: **geplante Notification** zum Endzeitpunkt (Mobile + Desktop); bei Pause/Stop canceln.
4. App-Resume (`RunEvent::Resumed` / visibilitychange) → Backend rechnet Zustand nach; abgelaufene Sessions werden mit korrektem `ended_at` abgeschlossen.
5. Tick-Events (1 s) nur solange App im Vordergrund.

---

## 2. BDD-Strategie

### 2.1 Ein Feature-Ordner, mehrere Runner (per Tags)

```
features/
├── timer/pomodoro.feature, flowtime.feature, persistence.feature
├── todos/todotxt_parsing.feature, crud.feature, filter_sort.feature
├── stats/aggregation.feature, streaks.feature, export.feature
├── backup/daily_backup.feature, rotation.feature, restore.feature
├── sync/provider.feature, webdav.feature, google_drive.feature, crdt.feature, crdt_store.feature, existing_database.feature
├── todoist/auth.feature, mapping.feature, sync.feature
└── ui/navigation.feature, shortcuts.feature, responsive.feature
```

Tags: `@domain` (Rust, pure core), `@integration` (Rust, echte SQLite in-memory + wiremock HTTP), `@ui` (Frontend mit gemocktem IPC), `@e2e` (echte App), `@desktop` / `@mobile` / `@wip`. Jede Szenario-Gruppe trägt Trace-Tag zur Story: `@US-B01`, `@US-FT-02`, `@US-Todoist-Sync` …

| Ebene | Runner | Was |
|---|---|---|
| Domain + Integration | **`cucumber` crate (cucumber-rs)** in `tests/bdd.rs`, `harness = false`; World enthält In-Memory-DB, `FakeClock`, `FakeSecretStore`, `RecordingEventSink`, `wiremock::MockServer` | Fachlogik: Timer-Automat, Break-Berechnung, Parser, Stats, Merge, Backup-Rotation, Restore-Validierung, Sync-Protokoll, Todoist-Mapping |
| UI | **Vitest + `quickpickle`** (Gherkin in Vitest) + Testing Library; IPC über `@tauri-apps/api/mocks` `mockIPC` | Komponenten-/Store-Verhalten, Formulare, Fehlermeldungen, Responsive-Varianten |
| E2E Desktop | **WebdriverIO + `tauri-driver` + `@wdio/cucumber-framework`** (Linux/Windows in CI; macOS nicht unterstützt) | kleine Smoke-Suite: Start → Todo → Pomodoro (Fake-Kurzdauer via Env) → Stats |
| E2E Mobile | **Maestro**-Flows (YAML) auf Android-Emulator/iOS-Simulator, je Flow auf `@mobile`-Szenario verlinkt | Smoke: Start, Timer, Hintergrund/Resume, WebDAV-Sync gegen Test-Server |

Rust-Unit-Tests (`#[test]`, `proptest` für Parser/Merge) ergänzen, wo Gherkin zu kleinteilig wäre.

### 2.2 Workflow pro Story (Outside-in)

1. Story + Akzeptanzkriterien → `.feature` (englisch), Review mit User → Tag `@wip`.
2. Step-Definitions schreiben → rot.
3. Domain-Code in `nunc-core` → `@domain` grün; dann Store/Sync → `@integration` grün.
4. Command + UI → `@ui` grün. Ggf. E2E-Smoke.
5. `@wip` entfernen. CI erzwingt: keine `@wip` auf `main`, keine undefined/pending Steps.

### 2.3 Erste Feature-Files aus Bestand ableiten

Go-Tests sind ausführbare Spezifikation → als Scenario Outlines übernehmen, z. B.:

```gherkin
@domain @US-FT-02
Feature: Flowtime break recommendation
  Scenario Outline: Break is a fixed ratio of worked time
    Given the flowtime ratio is <ratio>
    And the minimum flow state is 15 minutes
    When I finish a flowtime session after <worked> minutes
    Then the suggested break is <break> minutes

    Examples:
      | ratio | worked | break |
      | 0.2   | 50     | 10    |
      | 0.2   | 14     | 0     |
      | 0.5   | 15     | 7     |
      | 0.1   | 59     | 5     |
```

Quellen für Szenarien: `timer_test.go`, `timer_pure_test.go`, `todos_parse_test.go`, `todos_test.go`, `stats_test.go`, `stats_streaks_test.go`, `backup_*_test.go`, `sync_merge_test.go`, `sync_webdav_merge_test.go`, `settings_test.go`, `todoist/mapping_test.go`, `db_test.go`.

**Bestandsdaten-Szenarien** (`existing_database.feature`): Fixtures = echte DB aus der Go-App (`tests/fixtures/go-v5.db`) → „Given a backup created by nunc-go, When I restore it, Then all todos and sessions are present“. Die alte Sync-Datei `nunc-sync.json` wird nur einmal importiert (ADR 0016); der Roundtrip Go → Rust → Go entfällt.

---

## 3. Umsetzungsphasen

Jede Phase endet mit grünen Checks (s. 5) und lauffähigem Build.

### Phase 0 – Spezifikation konsolidieren (≈ 1 Woche)
- Anforderungen aus Docs + Go-Code abgleichen (veraltete Stellen in CLAUDE.md/REQUIREMENTS markieren; z. B. Long Break 25 min, „Mobile out of scope“ → jetzt in scope).
- Alle Features als `.feature` (englisch) mit Story-Tags anlegen, `@wip`. Mobile-Abweichungen als eigene Szenarien `@mobile` (z. B. Backup-Ordner nicht wählbar, Export via Share-Sheet).
- Go-Fixture-Dateien erzeugen (DB nach Migration 005, Sync-JSON, WebDAV-Datei).
- ADRs (kurz, `docs/adr/`): Tauri 2, rusqlite, specta, Timer-Zeitstempel-Modell, SecretStore-Strategie, BDD-Toolchain.

### Phase 1 – Skeleton & Toolchain (≈ 1 Woche)
- Bestehendes `create-tauri-app`-Gerüst (React-TS) übernehmen, Cargo-Workspace + Crates ergänzen, `tauri android init`, `tauri ios init`.
- tauri-specta Codegen, Tailwind-Tokens aus DESIGN.md, Router, Basis-Layout (Sidebar/Bottom-Tabs).
- cucumber-rs + quickpickle + Vitest lauffähig mit 1 Dummy-Szenario pro Runner.
- CI (Gitea Actions / GitHub Actions): `cargo fmt/clippy -D warnings`, `cargo test` (inkl. cucumber), `npm run lint/typecheck/test`, Build-Matrix macOS/Windows/Linux, Android-APK-Build; iOS-Build auf macOS-Runner.
- **Spike (Timebox 2 Tage):** Timer im Hintergrund auf Android + iOS (geplante Notification, Resume-Neuberechnung) und SecretStore Android → Risiko früh klären.

### Phase 2 – Domain-Kern `nunc-core` (≈ 2 Wochen)
- todo.txt-Parser/Serializer, Timer-Automat (start/pause/resume/stop/complete, 50 %-Regel aborted/completed, skipped Breaks, Long-Break-Zyklus, Auto-Start-Flags), Flowtime-Break, Stats/Streaks, Export, Sync-Merge (LWW + Soft-Delete + Sessions append-only per `device_id + started_at`), Todoist-Mapping, Backup-Rotation.
- Alle `@domain`-Szenarien grün; proptest für Parser-Roundtrip + Merge-Kommutativität.

### Phase 3 – Persistenz `nunc-store` (≈ 1,5 Wochen)
- Migrationsrunner kompatibel zu Go-`schema_migrations`; Migrationen 001–005 kopiert; 006 `active_timer`.
- Repos: settings (Key-Value mit typisierten Defaults = `models.DefaultSettings()`), todos, sessions, todoist_mappings, sync-Metadaten.
- Backup: `VACUUM INTO`, tägliches Auto-Backup (max. 1/Tag, Hintergrund), Rotation 7, `before-restore-YYYY-MM-DD.db`, Restore mit Schema-Validierung + atomarem Swap + Service-Reinit + `app:reload`-Event.
- Erstes Starten: alte Go-DB am Go-Pfad erkennen → übernehmen.
- `@integration`-Szenarien inkl. `existing_database.feature` grün.

### Phase 4 – Tauri-Commands & Timer-Runtime (≈ 1,5 Wochen)
- Commands 1:1 zu `nunc-go/app.go` (GetSettings, SaveSettings, GetTodos, AddTodo, UpdateTodo, DeleteTodo, SetTodoCompleted, ReorderTodos, StartTimer, StartBreak, PauseTimer, ResumeTimer, StopTimer, GetTimerTick, GetStats, ExportStats, SaveExportToFile, Backup*/Restore*, Sync*, Todoist*), snake_case in Rust, specta exportiert camelCase-Typen identisch zu Go-JSON-Tags.
- Timer-Runtime: tokio-Task für Ticks, Notification-Scheduling, Resume-Handling, Persistenz.
- Desktop: Tray (Icon idle/rot/grün, Menü Show/Hide, Start/Pause/Stop, Status, Quit), Mini-Timer-Fenster, Single-Instance.

### Phase 5 – Frontend Kernansichten (≈ 3 Wochen)
- Timer-View (RingTimer SVG, Farben je Modus, Pulsieren/„Breath“, Todo-Auswahl, Break-Vorschlag-Dialog), Todos (CRUD, Filter Projekt/Kontext/Prio/Freitext, Sortierung, 🍅-Zähler, Reorder per Drag & Drop/Touch), Stats (Tag/Woche/Monat/Jahr, Top-Todos, Projekt-/Kontext-Verteilung, Streaks, Export), Settings (Timer, Flowtime-Slider 0.1–0.5 in 0.05-Schritten + Anker + Info-Banner, Sound/Notifications).
- Keyboard-Shortcuts (Space, Esc, N, 1/2/3, `,`) Desktop.
- ARIA-Labels überall; `@ui`-Szenarien grün; Mockups in `nunc-go/screens/*` als visuelle Referenz.

### Phase 6 – Backup/Restore UI + WebDAV-Sync (Desktop + Mobile) (≈ 2 Wochen)
- Backup-Status (letztes Backup, Anzahl, Größe, Warnung > 2 Tage), „Jetzt sichern“, Liste, Restore mit Bestätigungsdialog, „Aus Datei wiederherstellen“ (nur `.db`).
  - Mobile: Backups in App-Sandbox; Export/Import über Datei-Picker/Share.
- Sync-Provider-Auswahl (none/google_drive/webdav, exklusiv, Bestätigung bei Wechsel, sofort gespeichert).
- WebDAV: Config (URL, basic/digest, User, Remote-Pfad; Passwort im SecretStore), Verbindungstest, Sync bei Start / Beenden (max. 5 s) / manuell / nach Änderungen (debounced), Statusleiste idle/syncing/error. Mobile: Sync bei App-Start + Resume + „in Hintergrund“ statt Beenden.
- WebDAV-Testserver in Tests: wiremock (integration) + Docker `rclone serve webdav` für E2E.

### Phase 7 – Google Drive + Todoist (Desktop) (≈ 2 Wochen)
- OAuth2 PKCE Loopback, Tokens im SecretStore (eigene Service-Namen; Konten werden nach dem Umstieg neu verbunden, Go-Keychain-Einträge werden nicht übernommen).
- Drive: Ordner `nunc/`, `nunc-sync.json`, `drive.file`-Scope, Import-Dialog auf neuem Gerät, Konflikt-Hinweise.
- Todoist: API v1 (geprüft, siehe ADR 0009), Mapping-Tabelle, serialisierter Sync, Close statt Delete.
- In Mobile-UI ausgeblendet (Szenarien `@desktop`).

### Phase 8 – Mobile-Feinschliff & Release (≈ 2 Wochen)
- Android: Notification-Channel, Exact-Alarm-Permission (Android 12+) bzw. Fallback, Back-Button-Handling, Icons/Splash.
- iOS: Notification-Permission-Flow, Haptics optional, App-Icons, Signing/Provisioning, TestFlight.
- Desktop: Signing/Notarisierung macOS, Windows-Installer (NSIS/MSI), Linux AppImage/deb; `tauri-plugin-updater` optional.
- Maestro-Smoke + WDIO-Smoke in CI; Performance-NFRs prüfen (Start < 2 s, < 100 MB RAM).

### Später (v1.x)
Google Drive + Todoist auf Mobile (Deep-Link-OAuth, Plattform-Client-IDs), Home-Screen-Widget / Live Activity (iOS) / Ongoing-Notification (Android) für laufenden Timer, konfigurierbare Shortcuts, Themes.

**Gesamtschätzung:** ≈ 16–17 Wochen für eine Person.

---

## 4. Risiken & Gegenmaßnahmen

| Risiko | Maßnahme |
|---|---|
| Timer im Hintergrund (Mobile) | Zeitstempel-Modell + geplante Notifications; Spike in Phase 1 |
| Secure Storage Android | `android-native-keyring-store` (Spike M0-10 bestätigt, kein eigenes Plugin nötig) |
| `tauri-driver` ohne macOS-Support | E2E-Desktop in CI auf Linux/Windows; macOS manuell + `@ui`-Ebene deckt Logik ab |
| Schema-Kompatibilität bricht | Nur additive Migrationen; `existing_database.feature` mit echten Go-Fixtures in CI |
| `.ogg` auf iOS | `.mp3`-Assets |
| OAuth-Loopback-Port belegt | Port-Fallback-Liste; Redirect-URIs bei Google/Todoist registrieren |

---

## 5. Definition of Done / Verifikation

Pflicht-Checks nach jeder Änderung (analog Go-`CLAUDE.md`, ins neue `nunc/CLAUDE.md` übernehmen):

```bash
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                       # inkl. cucumber-rs (@domain @integration)
npm run typecheck && npm run lint && npm run test   # Vitest + quickpickle (@ui)
npm run tauri build                           # Desktop-Build ohne Warnungen
```

Zusätzlich je Phase:
- `npm run tauri dev` → manueller Durchlauf der Phase-Szenarien.
- `npm run tauri android dev` / `npm run tauri ios dev` ab Phase 5 → Timer starten, App in Hintergrund, Notification kommt zur richtigen Zeit, Resume zeigt korrekten Stand.
- Kompatibilität: Backup aus laufender `nunc-go`-App in neue App restoren; WebDAV-Sync zwischen Go-App und Tauri-App parallel → identische Todos/Sessions.
- Coverage: `cargo llvm-cov` ≥ 80 % für `nunc-core`/`nunc-store`; Frontend-Utils 100 %.
- Living Documentation: cucumber-rs JSON/JUnit-Report + quickpickle-Report als CI-Artefakt; Traceability Story-Tag → Szenario → Status.

## 6. Erste konkrete Schritte nach Freigabe

1. Detaillierte Ticket-Roadmap: siehe [`roadmap.md`](./roadmap.md) (baut auf bestehendem Gerüst auf).
2. `nunc/CLAUDE.md` mit Architekturregeln, BDD-Workflow, Pflicht-Checks.
3. `features/` mit ersten 3 Features aus Go-Tests (Flowtime-Break, todo.txt-Parsing, Pomodoro-Zyklus) + cucumber-rs-Runner → rot → `nunc-core` → grün.
4. Spikes Hintergrund-Timer + Android-SecretStore.
