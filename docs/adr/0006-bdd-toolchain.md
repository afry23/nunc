# ADR 0006 – BDD-Toolchain

- Status: akzeptiert
- Datum: 2026-10-03

## Kontext

Gherkin-Szenarien (englisch) sollen Spezifikation und Test sein, aus einer Quelle für Backend und Frontend, mit
Rückverfolgbarkeit Story → Szenario → Status.

## Entscheidung

Ein Ordner `features/` für alle Runner, getrennt über Tags:

| Ebene | Runner | Tag |
|---|---|---|
| Fachlogik | `cucumber` 0.23 (cucumber-rs) in `crates/nunc-core/tests/bdd.rs`, `harness = false` | `@domain` |
| Persistenz/Sync | derselbe Runner in `nunc-store`/`nunc-sync` (In-Memory-SQLite, wiremock) | `@integration` |
| UI | Vitest + `quickpickle` + Testing Library, IPC über `mockIPC` | `@ui` |
| E2E | WebdriverIO + `tauri-driver` (Desktop, Linux/Windows), Maestro (Mobile) | `@e2e` |

Festlegungen aus der Einrichtung:

- **Rust-Runner** nutzt `filter_run_and_exit` (nicht `filter_run`, sonst Exit-Code 0 bei Fehlschlägen) und
  `fail_on_skipped()` (undefined Steps schlagen fehl). Szenarien mit `@wip` werden übersprungen. Mit
  `NUNC_BDD_REPORT=<abs. Pfad>` entsteht zusätzlich ein Cucumber-JSON-Report (Feature `output-json`).
- **quickpickle** kennt keinen Include-Tag-Filter: ein `Before`-Hook in `tests/steps/hooks.steps.ts` überspringt alle
  Szenarien ohne `@ui`. Das Paket muss in Vitest unter `server.deps.inline` stehen (sonst scheitert der Import von
  `pngjs/browser`). `@wip`/`@todo` markiert quickpickle selbst als todo.
- **Backend-Mock im UI-Test:** `tests/helpers/ipc.ts` (`mockBackend`, `resetBackend`) – unbekannte Commands werfen,
  damit fehlende Mocks auffallen.
- Reports als CI-Artefakt `cucumber-report` (`target/bdd/domain.json`, `ui.xml`) für die Living Documentation.
- Rust-Unit-Tests und `proptest` ergänzen, wo Gherkin zu kleinteilig wäre (Parser-Roundtrip, Merge-Eigenschaften).

Alternativen: reine `#[test]`-Tabellen (keine gemeinsame, lesbare Spezifikation), Cypress-Cucumber/Playwright
(schwer mit Tauri-IPC), `cucumber-js` (zweite Runner-Welt neben Vitest).

## Konsequenzen

- Szenarien werden einmal geschrieben und je Ebene dem passenden Runner zugeordnet; Step-Definitions leben pro
  Runner getrennt.
- Der Desktop-E2E-Test läuft nicht auf macOS (`tauri-driver`); dort deckt die `@ui`-Ebene die Logik ab.
- CI erzwingt: keine undefined/pending Steps, keine `@wip` auf `main` bei abgeschlossenen Tickets (Abnahme M8-08).
