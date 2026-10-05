# nunc

**nunc** is a focus timer with **Pomodoro** and **Flowtime** modes, tasks in the **todo.txt** format, statistics derived
from individual sessions, automatic backups, and sync. Built with Tauri 2 (Rust + React/TypeScript) for macOS, Windows,
Linux, Android and iOS.

## About the name

*Nunc* is Latin for **"now"**. The name reflects the core idea of the app: bring you into the present moment and help
you focus on what you are doing right now.

There is also a link to *kairos*, the Greek idea of the right, meaningful moment as opposed to *chronos*, mere clock
time. nunc does not just measure time; it supports a moment that is lived consciously.

## Features

- **Timer:** Pomodoro (work, short/long break, auto-start) and Flowtime (stopwatch, break derived from the work : break
  ratio). The timer runs in the backend on timestamps, so it survives backgrounding, restarts and app kills.
- **Tasks:** todo.txt with priority, `+project`, `@context`, 🍅 counter, filters, search, sorting, drag & drop.
  **Import/export** of a `todo.txt` file (Tasks view, "Import"/"Export"): import only adds new lines (duplicates and
  blank lines are skipped); export writes all active tasks, one line per task. The format has no pomodoro counter, so
  counters are lost on export.
- **Statistics:** day/week/month/year, streaks, breakdown by project/context, export as CSV/JSON.
- **Backup:** daily on start, rotation, restore from the list or from a `.db` file.
- **Sync:** WebDAV (all platforms), Google Drive and Todoist (desktop).
- **Local interface** (desktop, off by default): other programs exchange todo.txt lines and query status, see below.
- **Desktop:** tray icon, mini timer, keyboard shortcuts. **Mobile:** notification when the timer ends, swipe gestures.
- **Language:** German and English; "System" follows the operating system (Settings → Language).

## Installation

Prebuilt installers are available under *Releases*: macOS (`.dmg`), Windows (`.exe`/`.msi`), Linux (`.deb`, `.rpm`,
AppImage), Android (Play Store, internal test track or APK), iOS (TestFlight).

## Building locally

### Prerequisites (all platforms)

- [Rust](https://rustup.rs) (stable)
- Node.js 22 and npm
- The [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

Then install the dependencies once:

```bash
npm ci
```

Optional: Google Drive and Todoist sync need OAuth credentials at build time. Copy `.env.example` to `.env` and fill it
in (debug builds read it; release builds need `NUNC_*` set as environment variables). Without them, "Connect" is
disabled. See [CLAUDE.md](./CLAUDE.md).

### Desktop (macOS, Windows, Linux)

Extra requirements: Xcode Command Line Tools (macOS), Microsoft C++ Build Tools and WebView2 (Windows), WebKitGTK and
related packages (Linux), as listed in the Tauri prerequisites.

```bash
npm run tauri dev                      # development with hot reload
npm run tauri build                    # installers in target/release/bundle/
npm run tauri build -- --no-bundle     # binary only
```

### Android

Extra requirements: JDK 21, Android SDK and NDK (set `ANDROID_HOME`, `NDK_HOME` and `JAVA_HOME`), and the Android Rust
targets:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

```bash
npm run tauri android dev                                         # run on device/emulator
npm run tauri android build -- --debug --apk --target aarch64     # debug APK
npm run tauri android build -- --aab --apk                        # release (needs a signing key)
```

Release signing: [docs/release.md](./docs/release.md).

### iOS

Requires macOS with Xcode, [xcodegen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`), and the iOS Rust
targets:

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

```bash
npm run tauri ios dev                                          # run on simulator/device
npm run tauri ios build -- --debug --target aarch64-sim        # simulator build
APPLE_DEVELOPMENT_TEAM=<TEAMID> npm run tauri ios build -- --export-method app-store-connect   # signed .ipa
```

A signed build or a real device needs an Apple Developer account. Release and signing: [docs/release.md](./docs/release.md).

## Data locations

| | macOS | Windows | Linux |
|---|---|---|---|
| Database `nunc.db` | `~/Library/Application Support/com.afry.nunc/` | `%AppData%\com.afry.nunc\` | `~/.config/com.afry.nunc/` |
| Backups | `<data folder>/backups/` (or a custom folder in Settings) | same | same |
| Log `nunc.log` | `~/Library/Logs/com.afry.nunc/` | `%LocalAppData%\com.afry.nunc\logs\` | `~/.local/share/com.afry.nunc/logs/` |

On Android and iOS everything lives in the protected app storage. Passwords and tokens (WebDAV, Google, Todoist) are not
stored in the database but in the system keychain (service `nunc`).

## Backup and restore

- On start, at most one backup per day is created (`VACUUM INTO`, verified); older ones are rotated.
- *Settings → Backup:* "Back up now", choose a folder (desktop), list of backups, **Restore** (with confirmation) or
  **Restore from file**. Before each restore, `before-restore-<date>.db` is created; if the restore fails, the data
  stays unchanged.
- Mobile: export a backup via "Save as", restore through the file dialog.
- Backups from the Go app can be restored.

## Sync

*Settings → Sync*: exactly one provider.

- **WebDAV:** URL, login (Basic or Digest), remote path; "Test connection". Runs on start, on exit, after changes,
  manually, and when returning from the background (mobile).
- **Google Drive** (desktop): sign in via browser, file `nunc/nunc-sync.json` in the app's own Drive folder (scope
  `drive.file`).
- **Todoist** (desktop, in addition to the provider): tasks, projects and labels.
- **Automatic sync** (desktop): while the app stays open, the provider is synced every 5, 15 (default) or 30 minutes,
  or never (*Settings → Sync*, "Sync automatically every"). Todoist is not part of it. Mobile devices sync when the
  app returns from or goes to the background instead.

Merging: tasks use "last change wins" (deletion is a tombstone); sessions are only added.

## Local interface (desktop)

Other programs on the same machine can exchange tasks in todo.txt format and query their status.
*Settings → Local interface:* enable, save, "Show token". The server listens on `127.0.0.1` only (default port 42820)
and requires `Authorization: Bearer <token>`; browser requests are rejected.

```bash
T=…                                              # token from Settings
curl -H "Authorization: Bearer $T" http://127.0.0.1:42820/v1/todos                       # todo.txt
curl -H "Authorization: Bearer $T" -H "Accept: application/json" \
     "http://127.0.0.1:42820/v1/todos?completed=false&project=work"                      # JSON with status
curl -H "Authorization: Bearer $T" http://127.0.0.1:42820/v1/todos/3                     # one task
curl -X POST -H "Authorization: Bearer $T" --data-binary @todo.txt http://127.0.0.1:42820/v1/todos   # new lines only
curl -X PATCH -H "Authorization: Bearer $T" -H "Content-Type: application/json" \
     -d '{"completed":true}' http://127.0.0.1:42820/v1/todos/3                           # mark done
```

Full reference: [docs/api.md](./docs/api.md). Threat model: [ADR 0013](./docs/adr/0013-lokale-api.md).

## Development

BDD-first: the Gherkin scenarios in `features/` are both specification and test.

```bash
cargo test --workspace         # @domain and @integration (cucumber-rs), unit tests
npm run test                   # @ui (Vitest + quickpickle) and unit tests
npm run typecheck && npm run lint
npm run docs:report            # traceability + living documentation → target/docs/ (fails on @wip)
tests/nfr/measure-desktop.sh   # startup and memory of the release build
```

| Document | Contents |
|---|---|
| [CLAUDE.md](./CLAUDE.md) | Architecture rules, workflow, mandatory checks |
| [PLAN.md](./PLAN.md) · [roadmap.md](./roadmap.md) | Architecture, milestones M0–M8 |
| [docs/adr/](./docs/adr/) | Decisions (Tauri, SQLite, timer, sync, i18n, release …) |
| [features/README.md](./features/README.md) | Tags and scenario conventions |
| [docs/nfr.md](./docs/nfr.md) | Measuring startup, memory, drift |
| [docs/release.md](./docs/release.md) | Release and signing |

**Living documentation and traceability:** `npm run docs:report` generates `target/docs/living-docs.md` (all scenarios
per feature) and `traceability.md` (story/requirement → scenario → level → status, with run results if
`target/bdd/domain.json` exists). CI attaches both to the `cucumber-report` artifact.

## License

GPL-3.0-or-later, see [LICENSE](./LICENSE).
