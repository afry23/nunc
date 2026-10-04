import type { Settings } from "../lib/api";
import { resolveLanguage, setLanguage, systemLanguages } from ".";

/** Übernimmt die Sprache aus den Einstellungen (`system` folgt dem Betriebssystem). */
export function applySettingsLanguage(settings: Pick<Settings, "language">) {
  setLanguage(resolveLanguage(settings.language, systemLanguages()));
}
