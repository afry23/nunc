/**
 * Einziger Zugang der UI zum Rust-Backend.
 *
 * Re-exportiert die von tauri-specta generierten Bindings (`src/bindings.ts`,
 * Debug-Build bzw. `cargo test -p nunc export_bindings`). Außerhalb von
 * `src/lib/` sind `invoke` und `bindings` per ESLint gesperrt.
 */
export { commands } from "../bindings";
export type * from "../bindings";
