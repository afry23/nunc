import { t } from "../i18n";

/** Reine Hilfen für wiederkehrende Aufgaben (ADR 0036): `rec:[+]<n><d|b|w|m|y>`. Spiegel von `nunc_core::recurrence`. */

export const MAX_COUNT = 999;

export type RecurrenceUnit = "d" | "b" | "w" | "m" | "y";

export const RECURRENCE_UNITS: readonly RecurrenceUnit[] = [
  "d",
  "b",
  "w",
  "m",
  "y",
];

export interface Recurrence {
  /** `true`: ab der bisherigen Frist (`rec:+…`), sonst ab dem Abschlussdatum. */
  strict: boolean;
  count: number;
  unit: RecurrenceUnit;
}

const RULE = /^(\+?)(\d+)([dbwmy])$/;

/** Wert eines `rec:`-Worts (`+1w`); ungültig ist `null`. */
export function parseRecurrence(value: string): Recurrence | null {
  const match = RULE.exec(value);
  if (!match) return null;
  const count = Number(match[2]);
  if (count < 1 || count > MAX_COUNT) return null;
  return {
    strict: match[1] === "+",
    count,
    unit: match[3] as RecurrenceUnit,
  };
}

/** Wortform einer Regel, z. B. `+1w`. */
export function formatRecurrence(rule: Recurrence): string {
  return `${rule.strict ? "+" : ""}${rule.count}${rule.unit}`;
}

function ruleOfWord(word: string): Recurrence | null {
  return word.startsWith("rec:") ? parseRecurrence(word.slice(4)) : null;
}

/** Erste gültige Wiederholung einer Beschreibung. */
export function recurrenceOf(description: string): Recurrence | null {
  for (const word of description.split(/\s+/)) {
    const rule = ruleOfWord(word);
    if (rule) return rule;
  }
  return null;
}

/** Die Zeile ohne gültige `rec:`-Wörter; der übrige Text bleibt wortgleich. */
export function withoutRecurrence(line: string): string {
  return line
    .replace(/(\s*)(\S+)/g, (whole: string, _lead: string, word: string) =>
      ruleOfWord(word) ? "" : whole,
    )
    .trim();
}

/** Die Zeile mit genau dieser Regel (oder ohne); ein neues Wort steht am Ende. */
export function withRecurrence(line: string, rule: Recurrence | null): string {
  const base = withoutRecurrence(line);
  if (rule === null) return base;
  const word = `rec:${formatRecurrence(rule)}`;
  return base === "" ? word : `${base} ${word}`;
}

/** Regel aus den Eingabefeldern; `null` bei leerer oder ungültiger Anzahl. */
export function recurrenceFromInput(input: {
  count: string;
  unit: RecurrenceUnit;
  strict: boolean;
}): Recurrence | null {
  const count = Number(input.count);
  if (input.count === "" || !Number.isInteger(count)) return null;
  if (count < 1 || count > MAX_COUNT) return null;
  return { strict: input.strict, count, unit: input.unit };
}

/** Anzeigetext einer Regel, z. B. „alle 2 Wochen, ab Abschlussdatum“. */
export function recurrenceSummary(rule: Recurrence): string {
  return t("recurrence.summary", {
    count: rule.count,
    unit: t(`recurrence.unit.${rule.unit}`),
    base: t(rule.strict ? "recurrence.fromDue" : "recurrence.fromDone"),
  });
}
