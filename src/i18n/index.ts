/**
 * Texte der Oberfläche (roadmap M8-03, ADR 0011).
 *
 * Alle sichtbaren Texte stehen in `de.ts` (Standard, Quelle der Schlüssel) und `en.ts` (muss dieselben
 * Schlüssel haben, das prüft der Typ). Komponenten benutzen `useT()`, damit sie bei einem Sprachwechsel neu
 * rendern; Code außerhalb von React (Stores, `lib/`) ruft `t()` direkt auf.
 *
 * - Platzhalter: `{name}` wird durch `params.name` ersetzt.
 * - Mehrzahl: `"{count} Pomodoro|{count} Pomodoros"` – die erste Form gilt für „eins“, die zweite sonst
 *   (Regel der Sprache über `Intl.PluralRules`), Voraussetzung ist `params.count`.
 */
import { create } from "zustand";
import { de } from "./de";
import { en } from "./en";

export type MessageKey = keyof typeof de;
export type Language = "de" | "en";
/** Einstellung des Nutzers: `system` folgt dem Betriebssystem. */
export type LanguageSetting = Language | "system";
export type Params = Record<string, string | number>;

const CATALOGS: Record<Language, Record<MessageKey, string>> = { de, en };

/** Sprachen, die zur Auswahl stehen (Anzeigenamen in der eigenen Sprache). */
export const LANGUAGES: { value: LanguageSetting; native?: string }[] = [
  { value: "system" },
  { value: "de", native: "Deutsch" },
  { value: "en", native: "English" },
];

/** Löst die Einstellung in eine vorhandene Sprache auf; unbekannte Systemsprachen fallen auf Deutsch zurück. */
export function resolveLanguage(
  setting: LanguageSetting | string | undefined,
  systemLanguages: readonly string[] = [],
): Language {
  if (setting === "de" || setting === "en") return setting;
  for (const tag of systemLanguages) {
    const base = tag.toLowerCase().split("-")[0];
    if (base === "de" || base === "en") return base;
  }
  return "de";
}

interface LanguageState {
  language: Language;
}

/** Aktive Sprache. Bis die Einstellungen geladen sind, gilt Deutsch. */
export const useLanguageStore = create<LanguageState>(() => ({
  language: "de",
}));

export function currentLanguage(): Language {
  return useLanguageStore.getState().language;
}

/** Setzt die Sprache und das `lang`-Attribut der Seite (Screenreader, Silbentrennung). */
export function setLanguage(language: Language): void {
  useLanguageStore.setState({ language });
  if (typeof document !== "undefined") document.documentElement.lang = language;
}

/** Sprache des Systems (Browser/WebView). */
export function systemLanguages(): readonly string[] {
  return typeof navigator === "undefined"
    ? []
    : navigator.languages?.length
      ? navigator.languages
      : [navigator.language];
}

/** Locale für `Intl` und `toLocale…`. */
export function currentLocale(): string {
  return currentLanguage() === "en" ? "en-GB" : "de-DE";
}

function interpolate(
  message: string,
  language: Language,
  params?: Params,
): string {
  let text = message;
  if (params && "count" in params && text.includes("|")) {
    const [one, other] = text.split("|");
    const rule = new Intl.PluralRules(language).select(Number(params.count));
    text = rule === "one" ? one : other;
  }
  if (!params) return text;
  return text.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

/** Übersetzt einen Schlüssel in der aktiven Sprache. */
export function translate(
  language: Language,
  key: MessageKey,
  params?: Params,
): string {
  return interpolate(CATALOGS[language][key] ?? de[key], language, params);
}

export function t(key: MessageKey, params?: Params): string {
  return translate(currentLanguage(), key, params);
}

export type Translate = (key: MessageKey, params?: Params) => string;

/** Übersetzungsfunktion für Komponenten; rendert bei einem Sprachwechsel neu. */
export function useT(): Translate {
  const language = useLanguageStore((s) => s.language);
  return (key, params) => translate(language, key, params);
}

/** `true`, wenn es für den Schlüssel einen Text gibt. */
export function hasMessage(key: string): key is MessageKey {
  return key in de;
}
