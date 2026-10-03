# ADR 0003 – Typisierte Bridge mit tauri-specta

- Status: akzeptiert
- Datum: 2026-10-03

## Kontext

Die Go-App hatte einen handgepflegten TypeScript-Wrapper (`wailsbridge.ts`). Commands, Payloads und Events sollen
zwischen Rust und TypeScript nicht auseinanderlaufen.

## Entscheidung

- **`tauri-specta`** generiert `src/bindings.ts` aus den Rust-Signaturen (`#[tauri::command]` +
  `#[specta::specta]`, Typen mit `specta::Type`, serde `rename_all = "camelCase"`).
- Versionen sind **exakt gepinnt** (`specta = "=2.0.0-rc.25"`, `tauri-specta = "=2.0.0-rc.25"`, `specta-typescript
  0.0.12`), weil die Release-Candidates bei Updates brechen können.
- Export im Debug-Build nur auf dem Desktop (`#[cfg(all(debug_assertions, desktop))]`; auf Mobilgeräten existiert der
  Host-Pfad nicht und die App stürzte beim Start ab). Ohne GUI: `cargo test -p nunc export_bindings`.
  `src/bindings.ts` ist eingecheckt, CI prüft per `git diff --exit-code`, dass sie aktuell ist.
- Frontend greift nur über `src/lib/api.ts` zu (Re-Export von `commands` und Typen). ESLint verbietet `invoke` und
  Imports von `bindings` außerhalb von `src/lib/`.
- Fehlbare Commands liefern im Frontend ein Ergebnisobjekt (`{ status: "ok", data } | { status: "error", error }`)
  statt zu werfen; das Frontend muss beide Fälle behandeln.

Alternativen: handgeschriebene Wrapper (Drift-Gefahr), `ts-rs` (nur Typen, keine Commands/Events), JSON-Schema-
Generator (zusätzlicher Build-Schritt).

## Konsequenzen

- Rust ist die Quelle der Wahrheit für die API; Änderungen an Commands erfordern ein neu generiertes `bindings.ts`.
- Events werden mit `Event`-Derive typisiert und über `mount_events` registriert (mit den Timer-Events in M3).
- RC-Status des Crates: Updates bewusst und einzeln durchführen, Migration auf eine stabile Version beobachten.
