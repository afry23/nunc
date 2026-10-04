# Features

Eine Quelle für alle Runner (siehe ADR 0006). Szenarien sind englisch, Dateien liegen nach Bereich.

**Tags pro Szenario**

| Tag | Bedeutung |
|---|---|
| `@domain` · `@integration` · `@ui` · `@e2e` | Ebene/Runner (Rust pure · Rust mit SQLite/wiremock · Vitest · End-to-End) |
| `@US-…` · `@REQ-…` | Rückverfolgbarkeit: User Story (`nunc-go/nunc-userstories`, `STORIES.md`) bzw. Abschnitt in `REQUIREMENTS.md` |
| `@desktop` · `@mobile` | Nur auf dieser Plattformklasse |
| `@wip` | Szenario ist nur ein Titel, Steps fehlen noch; Runner überspringen es |
| `@property` · `@manual` | Eigenschaftstest (proptest, `Then the property "…" holds for N random cases`) · nicht automatisierbar, manueller Check |

**Stand (M0-13, `nunc-store` M2):** alle Szenarien tragen `@wip` und bestehen nur aus dem Titel. Beim Umsetzen eines Tickets Steps
ergänzen, rot → grün, dann `@wip` des Szenarios entfernen. `smoke*.feature` sind Runner-Selbsttests.

Fixtures für Kompatibilitätsszenarien: [`tests/fixtures/`](../tests/fixtures/README.md).

**UI-Szenarien (`@ui`)** liegen in `ui/*.feature`, getrennt von den Domänen-Dateien, weil deren `Background` zu den Rust-Runnern gehört.
Steps: `tests/steps/`, Fake-Backend mit Zustand: `tests/helpers/fakeBackend.ts`. E2E (`@e2e @desktop`): `e2e/`, `npm run test:e2e`.

**Mobile-E2E (`@e2e @mobile`)** setzen die Maestro-Flows in `e2e/mobile/` um (`npm run test:mobile`, siehe `e2e/mobile/README.md`);
ein Kommentar über dem Szenario nennt den Flow. Kein Cucumber-Runner führt diese Szenarien aus.
