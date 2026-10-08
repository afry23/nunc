# Plan: Wear-OS-Companion (Galaxy Watch)

Status: Entwurf · Datum: 2026-10-06

## Kontext

Tauri kann nicht auf Wear OS laufen. Das Ziel ist eine native Kotlin-Watch-App (Compose for Wear OS), die den Timer der Phone-App anzeigt und steuert (Wearable Data Layer). Logik, Datenbank und Sync bleiben auf dem Phone (Option A, Companion). Dieser Plan beschreibt zuerst die Spezifikation (BDD-first: ADR, Feature-Szenarien, Roadmap). Der Code folgt in Folgetickets.

Verworfene Alternativen:
- **Standalone (B):** Rust-Core per UniFFI/JNI auf der Watch, eigene SQLite, eigener Sync. Das ist deutlich größer.
- **Tauri auf der Watch (C):** nicht machbar. Es gibt keinen sinnvollen WebView-UI-Stack für runde Wear-Screens, Tiles und Complications.

## Schritt 1 – Branch

`docs/m9-01-wear-companion`. Kein Commit auf `main`.

## Schritt 2 – ADR `docs/adr/0015-wear-os-companion.md`

Format wie ADR 0013 (Status, Datum, Kontext, Entscheidung, Verworfene Alternativen, Folgen). Inhalt:

- **Entscheidung:** Companion statt Standalone. Natives Gradle-Modul `:wear`, gleiche `applicationId` `com.afry.nunc`.
- **Protokoll (Data Layer):**
  - Phone → Watch: `DataClient`-Item `/timer` mit `WearTimerState`. Es enthält die Felder des `TimerSnapshot`, ein absolutes `endsAtMs` (Countdown) bzw. `startedAtMs` (Flowtime) und `todoTitle: String?`. Die Watch zählt lokal aus dem Zeitstempel (ADR 0004).
  - Watch → Phone: `MessageClient` `/cmd/{start|pause|resume|stop|start_break}`. `start` trägt `mode` und optional `todoId`.
  - Das Wear-Payload kombiniert Snapshot und Titel. `TimerSnapshot` in `nunc-core` bleibt unverändert (Go-Kompatibilität, keine Specta-Änderung).
- **Umfang Watch:** Timer-Screen, Ongoing Activity, Tile, Complication, Haptik bei Phasenende. Keine Statistik, keine Einstellungen, keine Todo-Verwaltung.
- **Offline-Verhalten:** Die Watch zählt aus dem Zeitstempel weiter. Befehle ohne Verbindung liefern einen Fehlerhinweis, es gibt kein Queueing. Ist der Todo gelöscht, entfällt der Titel.
- **Sicherheit:** Befehle nur von der gepaarten Watch derselben App (Node-Capability). Keine Secrets über den Data Layer.
- **Phone-Seite:**
  - Kotlin-Klassen im Modul `:app` (`WearableListenerService`).
  - Brücke zu Rust über Tauri-Plugin-Command bzw. JNI.
  - Rust: `src-tauri/src/wear_service.rs` (`#[cfg(target_os = "android")]`), analog zu `api_service.rs`.
  - Fan-out-`EventSink` neben `TauriEventSink` in `src-tauri/src/adapters.rs`.
  - Befehle rufen `TimerService` auf. Die Sperrreihenfolge bleibt Timer, dann Db.
- **Folgen:** zweite UI-Codebasis, Play-Wear-Deklaration, zusätzlicher CI-Schritt `./gradlew :wear:assembleDebug`.
- **Bezug:** ADR 0004 (Foreground-Service/Live Activity im Backlog), ADR 0013.

## Schritt 3 – Feature-Szenarien `features/wear/`

Szenarien auf Englisch, mit Tags `@REQ-WEAR` und `@wip`.

- `wear_sync.feature` (`@integration`): Der Snapshot wird bei Start, Pause, Resume, Stop und Ende veröffentlicht. Das Payload enthält Endzeitstempel und Todo-Titel. Bei gelöschtem Todo fehlt der Titel. Es gibt keine Tick-Flut, nur Transitionen.
- `wear_commands.feature` (`@integration`): Start (Pomodoro/Flowtime, mit Todo), Pause, Resume, Stop und Start Break wirken wie am Phone. Ein im Zustand ungültiger Befehl liefert eine Fehlerantwort, der Timer bleibt unverändert. Befehle von unbekannten Nodes werden abgelehnt.
- `wear_watch.feature` (`@e2e @mobile`, nur Dokumentation wie `features/e2e/mobile.feature`): Die Watch zählt bei Verbindungsverlust lokal weiter. Haptik bei Phasenende. Tile und Complication zeigen die Restzeit.

Die Step-Definitions folgen im Folgeticket (Rot-Phase). `@wip` bleibt bis dahin, deshalb schlägt `npm run docs:report` im Branch erwartungsgemäß an.

## Schritt 4 – Roadmap

Neuer Abschnitt `## M9 – Wear-OS-Companion` in `roadmap.md` nach M8:

| ID | Ticket |
|---|---|
| M9-01 | ADR 0015 und Feature-Szenarien |
| M9-02 | Rust `wear_service` und Fan-out-Sink (`@integration`) |
| M9-03 | Kotlin-Phone-Bridge (Data Layer, Listener-Service) |
| M9-04 | `:wear`-Modul: Timer-Screen und Ongoing Activity |
| M9-05 | Tile, Complication, Haptik |
| M9-06 | CI, Release, Play-Wear-Deklaration (`ci.yml`, `release.yml`, `docs/release.md`) |
| M9-07 | Docs, Hilfe-View, i18n (Wear-Strings in Android-Ressourcen de/en) |

Die Backlog-Zeile „Laufender Timer außerhalb der App“ wird auf M9 verlinkt.

## Kritische Dateien

- Neu: `docs/adr/0015-wear-os-companion.md`, `features/wear/*.feature`
- Ändern: `roadmap.md`
- Referenz für die Folgetickets:
  - `src-tauri/src/timer_service.rs`
  - `src-tauri/src/adapters.rs`
  - `crates/nunc-platform/src/events.rs`
  - `crates/nunc-core/src/timer.rs` (`TimerSnapshot`, Zeile ~154)
  - `src-tauri/gen/android/settings.gradle` (`:wear` hier eintragen, nicht in der autogenerierten `tauri.settings.gradle`)
  - `.github/workflows/ci.yml`, `.github/workflows/release.yml`
  - `docs/release.md`

## Risiken

- Pairing und Verbindungsabbruch: Der lokale Fallback über den Zeitstempel ist Pflicht.
- Zwei UI-Codebasen (React und Compose). Geteilt ist nur das Datenmodell des Payloads (im ADR dokumentiert).
- Samsung-Akkuoptimierung kann Hintergrundprozesse beenden. Die Ongoing Activity mildert das.
- Die Brücke Kotlin zu Rust existiert noch nicht (bisher nur JS-Interface `window.NuncAndroid`). Sie ist der größte technische Unsicherheitsfaktor in M9-02/M9-03.

## Verifikation

- ADR und Szenarien per Sichtprüfung gegen ADR 0013 und `features/timer/pomodoro.feature`.
- `npm run docs:report` im Branch: Traceability-Tags (`@REQ-WEAR`, Ebenen-Tag) prüfen. Der `@wip`-Fehler ist erwartet.
- Keine Code-Änderung: `cargo fmt --check` zur Sicherheit.
- Commit mit Conventional Commits (`docs: ADR 0015 wear os companion and scenarios`). Kein Push ohne Auftrag.
