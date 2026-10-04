import { afterEach, describe, expect, it } from "vitest";
import {
  resolveLanguage,
  setLanguage,
  t,
  translate,
  useLanguageStore,
} from ".";
import { de } from "./de";
import { en } from "./en";
import { ApiError, errorMessage } from "../lib/api";

const placeholders = (text: string) =>
  [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

describe("resolveLanguage", () => {
  it("nimmt eine feste Wahl unverändert", () => {
    expect(resolveLanguage("en", ["de-DE"])).toBe("en");
    expect(resolveLanguage("de", ["en-US"])).toBe("de");
  });

  it("folgt bei „system“ der ersten bekannten Systemsprache", () => {
    expect(resolveLanguage("system", ["fr-FR", "en-US"])).toBe("en");
    expect(resolveLanguage("system", ["DE-at"])).toBe("de");
  });

  it("fällt ohne passende Sprache auf Deutsch zurück", () => {
    expect(resolveLanguage("system", ["fr-FR"])).toBe("de");
    expect(resolveLanguage("system", [])).toBe("de");
    expect(resolveLanguage("klingon", ["en"])).toBe("en");
    expect(resolveLanguage(undefined)).toBe("de");
  });
});

describe("Texte", () => {
  afterEach(() => setLanguage("de"));

  it("hat in beiden Sprachen dieselben Schlüssel, Platzhalter und keine leeren Texte", () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(de).sort());
    for (const key of Object.keys(de) as (keyof typeof de)[]) {
      expect(de[key].trim(), `de ${key}`).not.toBe("");
      expect(en[key].trim(), `en ${key}`).not.toBe("");
      expect(placeholders(en[key]), key).toEqual(placeholders(de[key]));
    }
  });

  it("ersetzt Platzhalter und lässt unbekannte stehen", () => {
    expect(translate("de", "language.label")).toBe("Sprache");
    expect(translate("en", "language.label")).toBe("Language");
  });

  it("wechselt mit der Sprache und setzt das lang-Attribut", () => {
    setLanguage("en");
    expect(t("language.label")).toBe("Language");
    expect(document.documentElement.lang).toBe("en");
    expect(useLanguageStore.getState().language).toBe("en");
  });
});

describe("errorMessage", () => {
  afterEach(() => setLanguage("de"));
  const error = new ApiError({
    code: "not_found",
    message: "Todo 7 nicht gefunden",
  });

  it("zeigt auf Deutsch die Meldung des Backends mit Details", () => {
    expect(errorMessage(error)).toBe("Todo 7 nicht gefunden");
  });

  it("übersetzt in anderen Sprachen über den Code, sonst bleibt die Meldung", () => {
    setLanguage("en");
    expect(errorMessage(error)).toBe("Not found.");
    expect(
      errorMessage(new ApiError({ code: "neu", message: "Unbekannt" })),
    ).toBe("Unbekannt");
  });

  it("kennt Fehler ohne Code", () => {
    expect(errorMessage(new Error("kaputt"))).toBe("kaputt");
    expect(errorMessage("x")).toBe("Unbekannter Fehler");
    setLanguage("en");
    expect(errorMessage("x")).toBe("Unknown error");
  });
});

describe("Platzhalter und Mehrzahl", () => {
  afterEach(() => setLanguage("de"));

  it("wählt die Form nach der Anzahl und ersetzt Platzhalter", () => {
    expect(translate("de", "sync.result.sessions", { count: 1 })).toBe(
      "1 Session",
    );
    expect(translate("de", "sync.result.sessions", { count: 3 })).toBe(
      "3 Sessions",
    );
    expect(translate("en", "stats.days", { count: 1 })).toBe("1 day");
    expect(translate("en", "stats.days", { count: 0 })).toBe("0 days");
    expect(translate("de", "todos.item.edit", { title: "Bericht" })).toBe(
      "Bearbeiten: Bericht",
    );
  });

  it("formatiert Zahlen und Datum je Sprache", async () => {
    const { formatBytes, shortDate } = await import("../lib/format");
    setLanguage("en");
    expect(formatBytes(3.4 * 1024 * 1024)).toBe("3.4 MB");
    expect(shortDate("2026-10-04")).toBe("04/10");
    setLanguage("de");
    expect(formatBytes(3.4 * 1024 * 1024)).toBe("3,4 MB");
    expect(shortDate("2026-10-04")).toBe("04.10.");
  });
});
