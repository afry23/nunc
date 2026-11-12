import type { JiraSettings } from "./api";
import { isHttpUrl } from "./sync";

/** Reicht der gespeicherte Zugang für eine Suche (Adresse, Token, bei Cloud auch die E-Mail)? */
export function isJiraConfigured(settings: JiraSettings | null): boolean {
  if (!settings || !settings.hasToken) return false;
  const { baseUrl, kind, email } = settings.config;
  return isHttpUrl(baseUrl) && (kind === "data_center" || email.trim() !== "");
}
