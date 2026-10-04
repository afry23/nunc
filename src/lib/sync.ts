import { t } from "../i18n";
import type { SyncResult, WebDavConfig } from "./api";

/** `http://` oder `https://` gefolgt von einem Host (wie `is_http_url` im Kern). */
export function isHttpUrl(text: string): boolean {
  const match = /^https?:\/\/([^/?#]*)/i.exec(text.trim());
  const host = match?.[1] ?? "";
  return host !== "" && !/\s/.test(host) && !host.startsWith(":");
}

/** Reicht die Konfiguration für einen Sync (Adresse und Benutzer ausgefüllt)? */
export function isWebDavComplete(config: WebDavConfig): boolean {
  return isHttpUrl(config.url) && config.username.trim() !== "";
}

/** Meldung zu einem Sync-Lauf, `null`, wenn es nichts zu melden gibt. */
export function describeSyncResult(result: SyncResult): string | null {
  const parts: string[] = [];
  if (result.importedSessions > 0)
    parts.push(t("sync.result.sessions", { count: result.importedSessions }));
  if (result.importedTodos > 0)
    parts.push(t("sync.result.todos", { count: result.importedTodos }));
  const notes: string[] = [];
  if (parts.length > 0)
    notes.push(
      t("sync.result.imported", { parts: parts.join(t("sync.result.and")) }),
    );
  if (result.conflictedTodos > 0)
    notes.push(t("sync.result.conflicts", { count: result.conflictedTodos }));
  return notes.length > 0 ? notes.join(" ") : null;
}

/** Hat ein Lauf Daten verändert, die die Ansichten neu laden müssen? */
export function changedData(result: SyncResult): boolean {
  return (
    result.importedTodos + result.importedSessions + result.conflictedTodos > 0
  );
}
