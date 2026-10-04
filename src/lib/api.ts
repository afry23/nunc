/**
 * Einziger Zugang der UI zum Rust-Backend.
 *
 * Re-exportiert die von tauri-specta generierten Bindings (`src/bindings.ts`,
 * Debug-Build bzw. `cargo test -p nunc export_bindings`). Außerhalb von
 * `src/lib/` sind `invoke` und `bindings` per ESLint gesperrt.
 */
import type { AppError } from "../bindings";
import { currentLanguage, hasMessage, t } from "../i18n";

export { commands, events } from "../bindings";
export type * from "../bindings";

type Result<T> =
  { status: "ok"; data: T } | { status: "error"; error: AppError };

/** Fehler eines Backend-Commands: stabiler `code`, deutscher `message` zur Anzeige. */
export class ApiError extends Error {
  readonly code: string;

  constructor(error: AppError) {
    super(error.message);
    this.name = "ApiError";
    this.code = error.code;
  }
}

/** Packt das Ergebnis eines Commands aus; Fehler werden als [`ApiError`] geworfen. */
export async function unwrap<T>(result: Promise<Result<T>>): Promise<T> {
  const outcome = await result;
  if (outcome.status === "error") throw new ApiError(outcome.error);
  return outcome.data;
}

/**
 * Lesbare Meldung für beliebige Fehler (Anzeige im UI). Backend-Fehler tragen einen deutschen Text mit Details;
 * in anderen Sprachen ersetzt ihn der Text zum stabilen `code` (ADR 0011), sonst bleibt die Meldung.
 */
export function errorMessage(error: unknown): string {
  if (error instanceof ApiError) {
    const key = `error.${error.code}`;
    if (currentLanguage() !== "de" && hasMessage(key)) return t(key);
    return error.message;
  }
  if (error instanceof Error) return error.message;
  return t("error.unknown");
}
