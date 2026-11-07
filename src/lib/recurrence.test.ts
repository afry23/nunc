import { describe, expect, it } from "vitest";
import {
  formatRecurrence,
  parseRecurrence,
  recurrenceFromInput,
  recurrenceOf,
  recurrenceSummary,
  withRecurrence,
  withoutRecurrence,
} from "./recurrence";

describe("parseRecurrence", () => {
  it.each([
    ["1w", { strict: false, count: 1, unit: "w" }],
    ["+2m", { strict: true, count: 2, unit: "m" }],
    ["999d", { strict: false, count: 999, unit: "d" }],
    ["3b", { strict: false, count: 3, unit: "b" }],
    ["1y", { strict: false, count: 1, unit: "y" }],
  ])("liest %s", (value, expected) => {
    expect(parseRecurrence(value)).toEqual(expected);
  });

  it.each(["", "+", "d", "0d", "1000d", "1x", "1.5d", "-1d", "1dd"])(
    "lehnt %j ab",
    (value) => {
      expect(parseRecurrence(value)).toBeNull();
    },
  );
});

describe("formatRecurrence", () => {
  it("gibt die Wortform zurück", () => {
    expect(formatRecurrence({ strict: true, count: 1, unit: "w" })).toBe("+1w");
    expect(formatRecurrence({ strict: false, count: 12, unit: "m" })).toBe(
      "12m",
    );
  });
});

describe("recurrenceOf", () => {
  it("nimmt die erste gültige Regel", () => {
    expect(recurrenceOf("Gießen rec:bad rec:1y due:2026-03-04")).toEqual({
      strict: false,
      count: 1,
      unit: "y",
    });
  });
  it("ist null ohne Regel", () => {
    expect(recurrenceOf("Gießen")).toBeNull();
    expect(recurrenceOf("Gießen xrec:1w")).toBeNull();
  });
});

describe("withRecurrence / withoutRecurrence", () => {
  const weekly = { strict: true, count: 1, unit: "w" } as const;
  it("hängt ein neues Wort an", () => {
    expect(withRecurrence("Gießen due:2026-03-04", weekly)).toBe(
      "Gießen due:2026-03-04 rec:+1w",
    );
  });
  it("ersetzt ein vorhandenes Wort", () => {
    expect(withRecurrence("Gießen rec:1d", weekly)).toBe("Gießen rec:+1w");
  });
  it("entfernt mit null", () => {
    expect(withRecurrence("Gießen rec:1d", null)).toBe("Gießen");
  });
  it("lässt ungültige Wörter stehen", () => {
    expect(withoutRecurrence("Gießen rec:0d")).toBe("Gießen rec:0d");
  });
  it("kommt mit leerer Zeile zurecht", () => {
    expect(withRecurrence("", weekly)).toBe("rec:+1w");
  });
});

describe("recurrenceFromInput", () => {
  it("baut die Regel", () => {
    expect(
      recurrenceFromInput({ count: "2", unit: "b", strict: false }),
    ).toEqual({ strict: false, count: 2, unit: "b" });
  });
  it.each(["", "0", "1000", "1.5", "abc"])("lehnt %j ab", (count) => {
    expect(recurrenceFromInput({ count, unit: "d", strict: false })).toBeNull();
  });
});

describe("recurrenceSummary", () => {
  it("beschreibt Intervall und Basis", () => {
    const text = recurrenceSummary({ strict: false, count: 2, unit: "w" });
    expect(text).toContain("2");
    expect(text).toContain("Wochen");
    expect(text).toContain("Abschlussdatum");
    expect(recurrenceSummary({ strict: true, count: 1, unit: "m" })).toContain(
      "Frist",
    );
  });
});
