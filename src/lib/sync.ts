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
    parts.push(
      `${result.importedSessions} ${result.importedSessions === 1 ? "Session" : "Sessions"}`,
    );
  if (result.importedTodos > 0)
    parts.push(
      `${result.importedTodos} ${result.importedTodos === 1 ? "Aufgabe" : "Aufgaben"}`,
    );
  const notes: string[] = [];
  if (parts.length > 0) notes.push(`${parts.join(" und ")} importiert.`);
  if (result.conflictedTodos > 0)
    notes.push(
      result.conflictedTodos === 1
        ? "1 Aufgabe wurde durch eine neuere Version vom Server ersetzt."
        : `${result.conflictedTodos} Aufgaben wurden durch neuere Versionen vom Server ersetzt.`,
    );
  return notes.length > 0 ? notes.join(" ") : null;
}

/** Hat ein Lauf Daten verändert, die die Ansichten neu laden müssen? */
export function changedData(result: SyncResult): boolean {
  return (
    result.importedTodos + result.importedSessions + result.conflictedTodos > 0
  );
}
