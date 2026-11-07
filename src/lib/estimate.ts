import { t } from "../i18n";
import type { Estimate, EstimateCheck, Verdict } from "./api";
import { formatMinutes } from "./format";

/** Reine Hilfen für die Aufwandsschätzung einer Aufgabe (ADR 0035): `est:N` (Minuten) oder `estp:N` (Pomodoros). */

export const MAX_MINUTES = 7 * 24 * 60;
export const MAX_POMODOROS = 999;

/** Eingabeart im Schätzfeld. */
export type EstimateMode = "pomodoros" | "time";

function parseWord(description: string, key: string, max: number) {
  for (const word of description.split(/\s+/)) {
    const match = new RegExp(`^${key}:(\\d+)$`).exec(word);
    if (!match) continue;
    const value = Number(match[1]);
    if (value >= 1 && value <= max) return value;
  }
  return null;
}

/** Spiegel von `nunc_core::estimate::estimate_of`: bei beiden Angaben gewinnen die Minuten. */
export function estimateOf(description: string): Estimate | null {
  const minutes = parseWord(description, "est", MAX_MINUTES);
  if (minutes !== null) return { unit: "minutes", value: minutes };
  const pomodoros = parseWord(description, "estp", MAX_POMODOROS);
  return pomodoros !== null ? { unit: "pomodoros", value: pomodoros } : null;
}

function isEstimateWord(word: string): boolean {
  return (
    parseWord(word, "est", MAX_MINUTES) !== null ||
    parseWord(word, "estp", MAX_POMODOROS) !== null
  );
}

/** Die todo.txt-Zeile ohne gültige `est:`/`estp:`-Wörter; der übrige Text bleibt wortgleich. */
export function withoutEstimate(line: string): string {
  return line
    .replace(/(\s*)(\S+)/g, (whole: string, _lead: string, word: string) =>
      isEstimateWord(word) ? "" : whole,
    )
    .trim();
}

/** Die Zeile mit genau dieser Schätzung (oder ohne): ersetzt vorhandene `est:`/`estp:`-Wörter, neue stehen am Ende. */
export function withEstimate(line: string, estimate: Estimate | null): string {
  const base = withoutEstimate(line);
  if (estimate === null) return base;
  const word = `${estimate.unit === "pomodoros" ? "estp" : "est"}:${estimate.value}`;
  return base === "" ? word : `${base} ${word}`;
}

/** Die Schätzung aus den Eingabefeldern; `null` bei leerer oder ungültiger Eingabe. */
export function estimateFromInput(
  mode: EstimateMode,
  input: { pomodoros: string; hours: string; minutes: string },
): Estimate | null {
  if (mode === "pomodoros") {
    const value = Number(input.pomodoros);
    return Number.isInteger(value) && value >= 1 && value <= MAX_POMODOROS
      ? { unit: "pomodoros", value }
      : null;
  }
  const hours = input.hours === "" ? 0 : Number(input.hours);
  const minutes = input.minutes === "" ? 0 : Number(input.minutes);
  if (!Number.isInteger(hours) || !Number.isInteger(minutes)) return null;
  if (hours < 0 || minutes < 0) return null;
  const total = hours * 60 + minutes;
  return total >= 1 && total <= MAX_MINUTES
    ? { unit: "minutes", value: total }
    : null;
}

/** Zerlegt Minuten in Stunden und Minuten für die Eingabefelder. */
export function splitMinutes(total: number): {
  hours: number;
  minutes: number;
} {
  return { hours: Math.floor(total / 60), minutes: total % 60 };
}

/** `3 Pomodoros` bzw. `1h 30m`. */
export function formatEstimate(estimate: Estimate): string {
  return estimate.unit === "pomodoros"
    ? t("estimate.pomodoros", { count: estimate.value })
    : formatMinutes(estimate.value);
}

/** Wert in der Einheit der Schätzung (für „tatsächlich“). */
export function formatInUnit(unit: Estimate["unit"], value: number): string {
  return unit === "pomodoros"
    ? t("estimate.pomodoros", { count: value })
    : formatMinutes(value);
}

/** „+1 Pomodoro“, „−15m“, „±0“. */
export function formatDelta(check: EstimateCheck): string {
  if (check.delta === 0) return t("estimate.exact");
  const sign = check.delta > 0 ? "+" : "−";
  return `${sign}${formatInUnit(check.estimate.unit, Math.abs(check.delta))}`;
}

export function verdictLabel(verdict: Verdict): string {
  return t(`estimate.verdict.${verdict}`);
}

/** Anteil „im Rahmen“ in Prozent (gerundet), `null` ohne Schätzungen. */
export function onTargetPercent(summary: {
  count: number;
  onTarget: number;
}): number | null {
  return summary.count === 0
    ? null
    : Math.round((summary.onTarget * 100) / summary.count);
}

/** Füllgrad des Soll/Ist-Balkens in Prozent (0–100) gegen die Schätzung. */
export function progressPercent(check: EstimateCheck): number {
  return Math.min(100, Math.round((check.actual * 100) / check.estimate.value));
}
