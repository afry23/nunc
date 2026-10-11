# CLAUDE.md – nunc (Tauri 2 · Rust · React/TS · BDD)

Run `cargo fmt` after you edit a rust file.
Run `cargo clippy --workspace --all-targets -- -D warnings` and `tsc --noEmit` before a commit.

## Überblick

nunc ist die Neuschreibung der Go/Wails-App `../nunc-go/` (Pomodoro-/Flowtime-Timer, todo.txt, Statistik,
Backup, Sync) als **Tauri-2-App** für macOS, Windows, Linux, Android und iOS. Entwicklung **BDD-first**:
Gherkin-Szenarien (englisch) sind Spezifikation und Test zugleich.

- Plan und Architektur: [`PLAN.md`](./PLAN.md) · Tickets und Reihenfolge: [`roadmap.md`](./roadmap.md)
- Entscheidungen: [`docs/adr/`](./docs/adr/) (vor größeren Änderungen lesen)
- Fachliche Referenz (nicht bindend): `../nunc-go/` (REQUIREMENTS.md, STORIES.md, DESIGN.md, `internal/**/_test.go`).
  Auf die Go-App wird **keine Rücksicht** mehr genommen (kein Parallelbetrieb, kein gemeinsames Sync-Format).
- **Bestandsdaten:** SQLite-Schema und Migrationen 001–005 bleiben unantastbar (bestehende nunc-Installationen,
  Backups älterer nunc-Versionen müssen restorebar bleiben). Das Sync-Format ist ein eigenes CRDT-Format
  (`nunc-sync.v2.json`, [ADR 0016](./docs/adr/0016-crdt-sync.md)). Secrets der Go-App werden nicht übernommen.

## Stack und Layout

| Schicht | Technik |
|---|---|
| Shell | Tauri 2 (`src-tauri/`), Plugins: opener (auch für die Google-Anmeldung), notification, dialog, fs, single-instance (Desktop) |
| Backend | Rust, Edition 2024, Cargo-Workspace |
| DB | SQLite über `rusqlite` (bundled), WAL, Foreign Keys |
| Typen Rust↔TS | `tauri-specta` → `src/bindings.ts` |
| Frontend | React 19, TypeScript strict, Vite, Tailwind v4, TanStack Router, Zustand |
| Tests | cucumber-rs (`@domain`, `@integration`), Vitest + quickpickle + Testing Library (`@ui`), proptest, cargo-mutants |

```
crates/nunc-core      pure Fachlogik, kein I/O, kein tokio
crates/nunc-store     rusqlite, Migrationen, Repos, Backup/Restore
crates/nunc-sync      WebDAV-, Google-Drive- und Todoist-Client (reqwest), OAuth für Google (PKCE, Loopback), Sync-Läufe (CRDT-Merge liegt pur in nunc-core), Steuerung (Koordinator)
crates/nunc-api       lokale HTTP-Schnittstelle (127.0.0.1, Token): pures Routing + hyper-Server, Desktop (ADR 0013)
crates/nunc-platform  Traits + Fakes: Clock, EventSink, Notifier, SecretStore (Keyring), UrlOpener
src-tauri/            Tauri-App: Commands, Events, Plugins, Tray (#[cfg(desktop)])
  src/state.rs        AppState (DB, TimerService, Tresor), Start, Restore, Shutdown-Hooks
  src/timer_service.rs  Timer-Laufzeit: Transitionen persistieren, Events, Benachrichtigung
  src/adapters.rs     Tauri-Adapter für EventSink/Notifier/UrlOpener (brauchen AppHandle)
  src/sync_service.rs Verdrahtung des Syncs: Store-Adapter, Koordinatoren (Sync-Anbieter, Todoist), Ereignisse
  src/api_service.rs  ApiService (Server an/aus, Token im Tresor) und AppBackend (Todos für nunc-api), nur Desktop
  src/credentials.rs  fester Google-Client nur in Debug-Builds: Umgebung (`NUNC_GOOGLE_*`) → `.env`
  src/commands/       dünne Commands je Bereich, Tests in commands/tests.rs (mock_builder)
src/                  React-Frontend (src/lib/api.ts = einziger Backend-Zugang)
  src/lib/            reine Hilfen (format, timer, todos, stats, shortcuts, sound), je mit Unit-Test
  src/stores/         Zustand: timer (nur Snapshot), todos, settings, ui; stores/backend.ts verdrahtet Events
  src/views/          Timer, Todos, Stats, Settings, MiniTimer; src/components/ Ring, Charts, Shell, Dialog
  tests/              steps/ (quickpickle), helpers/fakeBackend.ts; e2e/ WebdriverIO + tauri-driver, e2e/mobile/ Maestro
features/             Gherkin, EINE Quelle für alle Runner
docs/adr/             Architecture Decision Records
```

## Architekturregeln

**Backend**
- **Funktionaler Kern, Seiteneffekte am Rand.** `nunc-core` besteht aus puren Funktionen, Zeit kommt über eine
  injizierte `Clock`, Abhängigkeiten über Konstruktoren/Traits, kein globaler veränderlicher Zustand.
- Sperrreihenfolge: erst `TimerService` (Timer), dann `Db`. Commands ohne Timer sperren nur `Db`.
- Tauri-Commands in `src-tauri` sind dünn: Argumente prüfen, an Service/Core delegieren, Ergebnis abbilden.
  Keine Fachlogik in `lib.rs`/`commands.rs`.
- DB-Zugriff nur über `nunc-store`. Mehrschrittige Schreibvorgänge in einer **Transaktion**.
- Schema ändern nur über eine **neue, additive Migration** (`NNN_*.sql`). Migrationen 001–005 sind unantastbar.
- Fehler gehen als typisierter Fehler (`thiserror`) zum Frontend, nie als Panic. Kein `unwrap()`/`expect()` in
  Produktionscode außerhalb von Start-Invarianten.
- Commands und Typen mit `#[specta::specta]`/`specta::Type` (camelCase wie die Go-JSON-Tags) registrieren in
  `specta_builder()` in `src-tauri/src/lib.rs`.
- Plattformcode mit `#[cfg(desktop)]` / `#[cfg(mobile)]` bzw. `#[cfg(target_os = …)]` trennen; nichts, was nur auf
  dem Entwicklungsrechner existiert (Pfade!), auf Mobilgeräten ausführen.

**Frontend**
- Backend-Zugriff **nur** über `src/lib/api.ts`. `invoke` und `bindings` sind außerhalb von `src/lib/` per ESLint
  gesperrt.
- TypeScript strict, **kein `any`**. Pure Hilfsfunktionen in `src/lib/` (100 % Coverage), Zustand in Zustand-Stores.
- Das Frontend besitzt **keinen Timer-Zustand**, es rendert nur den Snapshot aus den Backend-Events.
- Design-Tokens aus `src/styles.css` (Quelle `nunc-go/DESIGN.md`): keine 1px-Trennlinien, kein reines Weiß, Breath-
  Animation bei aktivem Timer. Farben/Abstände über Tokens, nicht hartcodiert.

**Timer** (siehe ADR 0004): Zeit ist ein absoluter Zeitstempel, nie ein Zähler. Der Zustand wird bei jeder
Transition persistiert. Das Ende im Hintergrund meldet eine geplante lokale Notification; bei Resume wird aus dem
Zeitstempel neu berechnet. Tick-Events (`timer:tick`) sind nur Anzeige im Vordergrund.

## BDD-Workflow

1. **Feature zuerst:** Szenarien in `features/**/*.feature` (englisch), Tags `@wip` + Story-Tag (z. B. `@US-B01`).
2. **Rot:** Step-Definitions schreiben, Runner schlägt fehl.
3. **Grün von innen nach außen:** `nunc-core` → `nunc-store`/`nunc-sync` → Command → UI.
4. `@wip` entfernen, Pflicht-Checks ausführen, committen.

Tags: `@domain` (Rust, pure), `@integration` (Rust, In-Memory-SQLite + wiremock), `@ui` (Vitest, `mockIPC`),
`@sync` (Integrationsszenario für den Sync, läuft in `nunc-sync`), `@api` (lokale HTTP-Schnittstelle, läuft nur in `nunc-api`), `@e2e`, `@desktop`/`@mobile`, `@property` (Eigenschaftstest über zufällige Eingaben, ADR 0006), `@wip`. `@wip`-Szenarien werden von den Runnern übersprungen. Keine
undefined/pending Steps (der Rust-Runner bricht bei übersprungenen Steps ab).

```bash
cargo test -p nunc-core --test bdd      # @domain (NUNC_BDD_REPORT=/abs/pfad.json schreibt zusätzlich JSON)
cargo test -p nunc-store --test bdd     # @integration (Temp-Ordner, echte SQLite-Dateien)
cargo test -p nunc-sync --test bdd      # @integration @sync (WebDAV-/Google-/Todoist-Server mit Zustand per wiremock, Sync-Steuerung)
cargo test -p nunc-api --test bdd       # @api (pures Routing und Server über echten Socket)
npm run test:bdd                        # @ui (features/ui/*.feature, features/sync, features/todoist)
npm run test:e2e                        # @e2e @desktop, braucht tauri-driver (Linux/Windows)
npm run test:mobile                     # @e2e @mobile, Maestro-Flows gegen Emulator/Gerät (e2e/mobile/README.md)
```

Step-Definitions: Rust in `crates/*/tests/bdd/` (`main.rs` mit gemeinsamer `DomainWorld`, ein Modul pro Themenbereich), TypeScript in `tests/steps/` (`tests/helpers/ipc.ts` →
`mockBackend({ app_info: () => … })`). Details: ADR 0006.

## Texte, Fehler, Protokoll

- **Keine Klartexte in der UI:** sichtbare Texte stehen in `src/i18n/de.ts` (Quelle) und `en.ts` (gleiche Schlüssel, vom Typ
  erzwungen). Komponenten: `const t = useT()`, sonst `t()` aus `src/i18n`. Neue Sprache: ADR 0011.
- Fehler an die UI nur als `AppError` (`code` stabil, `message` deutsch); für neue Codes `error.<code>` in beiden Katalogen ergänzen.
  Jeder `AppError` landet im Protokoll (`nunc.log`, ADR 0010) – keine Secrets in Meldungen.
- Abnahme: `npm run docs:report` muss grün sein (kein `@wip`, jedes Szenario mit Ebenen-Tag und Story/Anforderung).
  Release: [docs/release.md](./docs/release.md).

## Pflicht-Checks (nach jeder Änderung, alle müssen sauber durchlaufen)

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace            # inkl. cucumber (@domain/@integration)
npm run typecheck && npm run lint && npm run test
npm run docs:report                # Abnahme: kein @wip, Traceability
npm run tauri build -- --debug --no-bundle   # ab M3: ohne Warnungen
```

Zusätzlich bei Änderungen an **Fachlogik** (`nunc-core`, `nunc-store`, `nunc-sync`), vor dem PR:

```bash
git diff main...HEAD > /tmp/pr.diff
cargo mutants --in-diff /tmp/pr.diff -p nunc-core -p nunc-store -p nunc-sync
```

Überlebende Mutanten heißen: Szenario oder `Examples`-Zeile fehlt. Ergänzen statt ausschließen; nur äquivalente
Mutanten begründet ausnehmen (ADR 0007). Ziel: Mutation-Score `nunc-core` ≥ 80 %, `nunc-store` ≥ 70 %.

Warnungen nicht per Ignore-Kommentar unterdrücken, außer mit dokumentiertem Grund. Nach Änderungen an Commands
oder specta-Typen `cargo test -p nunc export_bindings` ausführen und `src/bindings.ts` mit committen (CI prüft das).

## Git und PRs

- `main` ist geschützt: **nur per Pull Request**, Pflicht-Check `checks`. Pro Ticket ein Branch
  (`feat/m1-02-todotxt`, `docs/m0-12-project-docs`, …).
- Conventional Commits (`feat:`, `fix:`, `test:`, `docs:`, `build:`, `ci:`, `chore:`), ein Ticket = ein bis wenige
  Commits. Nicht ohne Auftrag pushen oder mergen.
- Generierte Plattformprojekte (`src-tauri/gen/android`, `gen/apple`) sind eingecheckt; rein kosmetische Diffs nach
  `tauri ios/android build` (z. B. `project.pbxproj`, `Info.plist`) verwerfen.

## Stolpersteine

- `tauri ios dev` / `tauri android dev` starten Vite auf **Port 1420 (strictPort)**. Verwaiste `vite`-Prozesse
  (`lsof -nP -iTCP:1420 -sTCP:LISTEN`) lassen den nächsten Start mit „Connection refused" scheitern. iOS-Builds aus
  Xcode heraus gehen nur, solange `tauri ios dev` läuft.
- `tauri-plugin-notification` (iOS): `Schedule.at` parst das UTC-Datum als Ortszeit und lehnt es still ab,
  `sendNotification` verschluckt Fehler (ADR 0004). Nicht unverändert verwenden.
- Android: `USE_EXACT_ALARM` ist nötig, sonst feuern Timer-Notifications Minuten zu spät (ADR 0004).
- Zugangsdaten: nunc baut **keine** OAuth-Clients ein. Google Drive nutzt den eigenen „Desktop-App“-Client der Nutzer
  (Einstellungen, Tresor-Konto `google-drive-client`, ADR 0019), Todoist das persönliche API-Token (ADR 0020). Für die
  Entwicklung liest ein Debug-Build einen festen Google-Client aus Umgebung/`.env` (`.env.example`); er hat Vorrang vor
  dem eingetragenen. Tokens liegen nur im Tresor (Dienst `nunc`, Konten `google-drive`, `google-drive-client`,
  `todoist`, `webdav`). Kein Workflow braucht OAuth-Secrets.
- Keychain-Tests berühren die Login-Keychain und können einen Dialog auslösen; sie sind `#[ignore]` und nutzen nur
  eigene Wegwerf-Einträge. Secrets der Go-App werden nicht übernommen (ADR 0005).
- Headless-iOS-Simulator: kein `Simulator.app`, `simctl` kann nicht tippen; UI-Interaktion braucht XCUITest oder ein
  Gerät.
- Mobil: „mobil“ entscheidet `isMobile()` (`src/lib/platform.ts`, `plugin-os`), nicht der User-Agent. Android-Funktionen ohne Plugin
  laufen über die Brücke `window.NuncAndroid` (`MainActivity.kt`, nur lesen/Einstellungen öffnen). Die Android-App ist Edge-to-edge:
  Tastaturabstand setzt `MainActivity.kt`. Nach Layout-Änderungen auf schmalen Screens (≈ 411 px) auf horizontalen Überlauf prüfen:
  der macht die WebView kleiner und schiebt die Tab-Leiste weg.
- Android-Build/Emulator: `npm run tauri android build -- --debug --apk --target aarch64`, `adb install -r …/app-universal-debug.apk`;
  WebView per `adb forward tcp:9333 localabstract:webview_devtools_remote_<pid>` inspizierbar (CDP).

## Do Not

- Keine Fachlogik in `src-tauri` (Commands) oder React-Komponenten – sie gehört nach `nunc-core`.
- Kein `invoke`/`bindings`-Import außerhalb von `src/lib/`; kein `any` in TypeScript.
- Keinen Timer im Frontend laufen lassen oder Zeit über Tick-Zähler messen.
- Keine abgeleiteten Statistiken speichern – immer aus den Roh-`sessions` berechnen.
- Kein Schema-Eingriff ohne neue, additive Migration; Migrationen 001–005 nicht ändern.
- Keine `Date.now()`/`SystemTime::now()` in Fachlogik – `Clock` injizieren.
- Nicht direkt auf `main` committen; keine `@wip`-Szenarien auf `main` lassen, wenn das Ticket abgeschlossen ist.
- Das Token der lokalen Schnittstelle nie loggen, in Fehlermeldungen oder Antworten aufnehmen; die Schnittstelle bindet nie an etwas anderes als `127.0.0.1`.
- Keine Secrets (Tokens, Passwörter) in Logs, DB, Settings oder Testausgaben; nur über `SecretStore`.
