import type { WebDavConfig } from "./api";

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
