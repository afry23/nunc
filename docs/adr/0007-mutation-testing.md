# ADR 0007 – Mutation-Tests

- Status: akzeptiert (Rust); TypeScript zurückgestellt
- Datum: 2026-10-03

## Kontext

Die Fachlogik (`nunc-core`, Persistenz und Sync-Protokolle) ist pure Logik mit vielen Grenzfällen (50-%-Regel,
Mindest-Flow-Zeit, Rundung, LWW-Merge, Streaks). Zeilen-Coverage sagt nur, dass Code ausgeführt wurde, nicht, ob die
Tests ein falsches Verhalten bemerken würden. Mutation-Tests ändern den Code gezielt (z. B. `<` → `<=`) und prüfen, ob
mindestens ein Test fehlschlägt.

## Entscheidung

**Rust: `cargo-mutants`** für `nunc-core`, `nunc-store` und `nunc-sync`.

- Konfiguration in `.cargo/mutants.toml`: ausgenommen sind `src-tauri/` (Glue), `nunc-platform/` (OS-Adapter) und
  Testcode; `Debug`/`Display`-Implementierungen erzeugen nur Rauschen. `timeout_multiplier = 3`, damit der
  cucumber-Runner (`harness = false`) pro Mutant nicht in ein Timeout läuft.
- Pro Mutant läuft `cargo test` des jeweiligen Pakets, also **Unit-Tests und BDD-Szenarien (`@domain`/`@integration`)
  zusammen**. Ein überlebender Mutant bedeutet meist: es fehlt ein Szenario oder eine Beispielzeile
  (`Examples`-Tabelle), nicht ein zusätzlicher Unit-Test.
- Lokal: `npm run mutate:rust` (alles) oder für ein Ticket nur geänderten Code:
  `git diff main...HEAD > /tmp/pr.diff && cargo mutants --in-diff /tmp/pr.diff -p nunc-core -p nunc-store`.
  Exit-Code 2 = überlebende Mutanten, 3 = Timeouts, 0 = alles erkannt oder keine Mutanten im Diff.
- CI (`.github/workflows/mutation.yml`): auf Pull Requests nur der geänderte Code (`--in-diff`), wöchentlich und per
  `workflow_dispatch` ein vollständiger Lauf; `mutants.out/` als Artefakt. Der Job ist **kein Pflicht-Check** der
  Branch Protection, bis genug Erfahrung mit Laufzeit und Stabilität vorliegt.
- Zielwerte für den Mutation-Score (erkannt / (erkannt + überlebt)): `nunc-core` ≥ 80 %, `nunc-store` ≥ 70 %. In neu
  geschriebener Fachlogik darf ein Mutant nur überleben, wenn er **äquivalent** ist (gleiches Verhalten); das wird in
  der PR-Beschreibung oder über `exclude_re` in `.cargo/mutants.toml` begründet.

### Verifikation im Spike

Mit einer Wegwerf-Funktion `break_minutes(worked, ratio, min_flow)` und einem einzigen Test: 7 Mutanten, 5 erkannt,
2 überlebt (`<` → `<=` und `<` → `==` an der Mindest-Flow-Grenze). Genau dieser Grenzfall ist `@US-FT-02`
(„A session of exactly the minimum flow state suggests a break“). `--in-diff` und die Exit-Codes wurden lokal geprüft.

## TypeScript: zurückgestellt

Stryker (`@stryker-mutator/core` + `vitest-runner` 10.0.0) wurde ausprobiert. Mit dem installierten **Vitest 5.0.3**
führt der Runner bei Mutanten **still 0 Tests** aus („Ran 0.00 tests per mutant“) und meldet alle als überlebt, ohne
Fehlermeldung. Mit **Vitest 4.1.11** läuft derselbe Aufbau korrekt (5 von 6 Mutanten erkannt). Weitere Stolpersteine:
Stryker kopiert das Projekt in ein Sandbox-Verzeichnis und scheitert an `target/` (Socket-Dateien) →
`ignorePatterns` für `target`, `src-tauri`, `crates`, `docs`, `.git` nötig; `ignoreStatic` und `perTest`-Coverage
funktionieren; ein Block-Statement-Mutant kann am TypeScript-Checker scheitern (`CompileError`, zählt nicht).

Entscheidung: Vitest 5 bleibt, TypeScript-Mutation-Tests folgen, sobald ein Stryker-Release Vitest 5 unterstützt
(Prüfen bei jedem Update von `vitest` oder `@stryker-mutator/*`). Geplanter Umfang dann: `src/lib/**` (pure
Hilfsfunktionen, ohne `api.ts`/`bindings.ts`), `thresholds.break = 80`, inkrementell mit `reports/stryker-
incremental.json`. Bis dahin gelten für das Frontend die 100 % Coverage der pure Utils (CLAUDE.md).

## Konsequenzen

- Mutation-Läufe sind teuer (jeder Mutant = Build + Test): lokal nur mit `--in-diff`, vollständig nur in CI/nachts.
- Die Qualität der BDD-Szenarien wird messbar; Grenzfälle gehören als `Examples`-Zeilen in die Features.
- Äquivalente Mutanten brauchen manuelles Urteil und eine kurze Begründung.
- Das Frontend ist vorerst nicht mutationsgetestet; das Risiko ist klein, solange Fachlogik im Rust-Kern liegt.
