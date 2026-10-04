# Features

Eine Quelle für alle Runner (siehe ADR 0006). Szenarien sind englisch, Dateien liegen nach Bereich.

**Tags pro Szenario**

| Tag | Bedeutung |
|---|---|
| `@domain` · `@integration` · `@ui` · `@e2e` | Ebene/Runner (Rust pure · Rust mit SQLite/wiremock · Vitest · End-to-End) |
| `@US-…` · `@REQ-…` | Rückverfolgbarkeit: User Story (`nunc-go/nunc-userstories`, `STORIES.md`) bzw. Abschnitt in `REQUIREMENTS.md` |
| `@sync` | `@integration`-Szenario für WebDAV, Google Drive, Todoist und die Sync-Steuerung: läuft im Runner von `nunc-sync` (wiremock-Server mit Zustand), nicht in `nunc-store` |
| `@desktop` · `@mobile` | Nur auf dieser Plattformklasse |
| `@wip` | Szenario ist nur ein Titel, Steps fehlen noch; Runner überspringen es |
| `@property` · `@manual` | Eigenschaftstest (proptest, `Then the property "…" holds for N random cases`) · nicht automatisierbar, manueller Check |

**Stand (M8):** kein Szenario trägt `@wip`; `npm run docs:report` prüft das. Neue Szenarien beginnen als `@wip` (nur Titel), Steps
ergänzen, rot → grün, dann `@wip` entfernen. `smoke*.feature` sind Runner-Selbsttests.

Fixtures für Kompatibilitätsszenarien: [`tests/fixtures/`](../tests/fixtures/README.md).

**UI-Szenarien (`@ui`)** liegen in `ui/*.feature`, getrennt von den Domänen-Dateien, weil deren `Background` zu den Rust-Runnern gehört.
Steps: `tests/steps/`, Fake-Backend mit Zustand: `tests/helpers/fakeBackend.ts`. E2E (`@e2e @desktop`): `e2e/`, `npm run test:e2e`.

**Mobile-E2E (`@e2e @mobile`)** setzen die Maestro-Flows in `e2e/mobile/` um (`npm run test:mobile`, siehe `e2e/mobile/README.md`);
ein Kommentar über dem Szenario nennt den Flow. Kein Cucumber-Runner führt diese Szenarien aus.
