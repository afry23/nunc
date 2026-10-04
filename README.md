# nunc

Fokus-Timer mit **Pomodoro** und **Flowtime**, Aufgaben im **todo.txt**-Format, Statistik aus den einzelnen Sessions,
automatischer Datensicherung und Synchronisierung. Tauri 2 (Rust + React/TypeScript) für macOS, Windows, Linux,
Android und iOS – die Neuschreibung der Go/Wails-App `nunc-go`; die Daten bleiben kompatibel
([Umstieg von der Go-App](./docs/migration-from-go.md)).

## Funktionen

- **Timer:** Pomodoro (Arbeit, kurze/lange Pause, Auto-Start) und Flowtime (Stoppuhr, Pause aus dem Verhältnis
  Arbeit : Pause). Der Timer läuft im Backend über Zeitstempel; er überlebt Hintergrund, Neustart und Kill der App.
- **Aufgaben:** todo.txt mit Priorität, `+Projekt`, `@Kontext`, 🍅-Zähler, Filter, Suche, Sortierung, Drag & Drop.
- **Statistik:** Tag/Woche/Monat/Jahr, Streaks, Verteilung nach Projekt/Kontext, Export als CSV/JSON.
- **Backup:** täglich beim Start, Rotation, Wiederherstellen aus der Liste oder einer `.db`-Datei.
- **Sync:** WebDAV (alle Plattformen), Google Drive und Todoist (Desktop).
- **Desktop:** Tray-Symbol, Mini-Timer, Tastenkürzel. **Mobil:** Benachrichtigung bei Timer-Ende, Wischgesten.
- **Sprache:** Deutsch und Englisch, „System“ folgt dem Betriebssystem (Einstellungen → Sprache).

## Installation

Fertige Installer stehen unter *Releases*: macOS (`.dmg`), Windows (`.exe`/`.msi`), Linux (`.deb`, `.rpm`, AppImage),
Android (Play Store, interner Test-Track bzw. APK), iOS (TestFlight). Selbst bauen:

```bash
npm ci
npm run tauri build            # Desktop-Installer in target/release/bundle/
npm run tauri dev              # Entwicklung (Hot Reload)
npm run tauri android build -- --debug --apk --target aarch64    # Android
npm run tauri ios dev          # iOS (braucht Xcode und xcodegen)
```

Voraussetzungen: Rust (stable), Node 22, Tauri-Voraussetzungen der Plattform; Android: JDK 21, SDK/NDK; iOS: Xcode.
Release und Signierung: [docs/release.md](./docs/release.md).

## Datenpfade

| | macOS | Windows | Linux |
|---|---|---|---|
| Datenbank `nunc.db` | `~/Library/Application Support/com.afry.nunc/` | `%AppData%\com.afry.nunc\` | `~/.config/com.afry.nunc/` |
| Sicherungen | `<Datenordner>/backups/` (oder eigener Ordner in den Einstellungen) | dito | dito |
| Protokoll `nunc.log` | `~/Library/Logs/com.afry.nunc/` | `%LocalAppData%\com.afry.nunc\logs\` | `~/.local/share/com.afry.nunc/logs/` |

Auf Android und iOS liegt alles im geschützten App-Speicher. Passwörter und Tokens (WebDAV, Google, Todoist) liegen
nicht in der Datenbank, sondern im Schlüsselspeicher des Systems (Dienst `nunc`).

## Backup und Wiederherstellung

- Beim Start entsteht höchstens einmal pro Tag eine Sicherung (`VACUUM INTO`, geprüft); ältere werden rotiert.
- *Einstellungen → Datensicherung:* „Jetzt sichern“, Ordner wählen (Desktop), Liste der Sicherungen, **Wiederherstellen**
  (mit Bestätigung) oder **Aus Datei wiederherstellen**. Vor jeder Wiederherstellung entsteht `before-restore-<Datum>.db`;
  schlägt sie fehl, bleiben die Daten unverändert.
- Mobil: Sicherung per „Speichern unter“ exportieren, Wiederherstellen über den Datei-Dialog.
- Sicherungen der Go-App lassen sich wiederherstellen.

## Synchronisierung

*Einstellungen → Synchronisierung* – genau ein Anbieter:

- **WebDAV:** Adresse, Anmeldung (Basic oder Digest), Remote-Pfad; „Verbindung testen“. Läuft beim Start, beim Beenden,
  nach Änderungen, manuell und beim Wechsel aus dem Hintergrund (Mobil).
- **Google Drive** (Desktop): Anmeldung im Browser, Datei `nunc/nunc-sync.json` im eigenen Drive-Ordner (Scope `drive.file`).
- **Todoist** (Desktop, zusätzlich zum Anbieter): Aufgaben, Projekte und Labels.

Zusammenführen: Aufgaben nach „letzte Änderung gewinnt“ (Löschen ist ein Merker), Sessions werden nur ergänzt.
Google und Todoist brauchen OAuth-Zugangsdaten beim Bauen (`NUNC_GOOGLE_CLIENT_ID` …, siehe `.env.example` und
[CLAUDE.md](./CLAUDE.md)); ohne sie ist „Verbinden“ deaktiviert.

## Entwicklung

BDD-first: die Gherkin-Szenarien in `features/` sind Spezifikation und Test.

```bash
cargo test --workspace         # @domain und @integration (cucumber-rs), Unit-Tests
npm run test                   # @ui (Vitest + quickpickle) und Unit-Tests
npm run typecheck && npm run lint
npm run docs:report            # Traceability + Living Documentation → target/docs/ (prüft: kein @wip)
tests/nfr/measure-desktop.sh   # Start und Speicher des Release-Builds
```

| Dokument | Inhalt |
|---|---|
| [CLAUDE.md](./CLAUDE.md) | Architekturregeln, Workflow, Pflicht-Checks |
| [PLAN.md](./PLAN.md) · [roadmap.md](./roadmap.md) | Architektur, Meilensteine M0–M8 |
| [docs/adr/](./docs/adr/) | Entscheidungen (Tauri, SQLite, Timer, Sync, i18n, Release …) |
| [features/README.md](./features/README.md) | Tags und Szenario-Konventionen |
| [docs/nfr.md](./docs/nfr.md) | Messung Start, Speicher, Drift |

**Living Documentation und Traceability:** `npm run docs:report` erzeugt `target/docs/living-docs.md` (alle Szenarien je
Feature) und `traceability.md` (Story/Anforderung → Szenario → Ebene → Status, mit Lauf-Ergebnis, wenn
`target/bdd/domain.json` vorliegt). Die CI hängt beide dem Artefakt `cucumber-report` an.

## Lizenz

GPL-3.0-or-later, siehe [LICENSE](./LICENSE).
