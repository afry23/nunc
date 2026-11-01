import { t } from "../i18n";
import type { Review } from "../bindings";
import {
  addDays,
  formatMinutes,
  parseIsoDate,
  shortDate,
  toIsoDate,
} from "./format";

/** Reine Hilfen für den Rückblick (ADR 0025). Alle Tage sind lokale Tage (`YYYY-MM-DD`). */

export type ReviewKind = "week" | "month" | "quarter";

export const REVIEW_KINDS: ReviewKind[] = ["week", "month", "quarter"];

const MONTHS_PER_KIND = { month: 1, quarter: 3 } as const;

/** Zeitraum, der `periodsAgo` Wochen/Monate/Quartale vor dem Zeitraum von `today` liegt (negativ: danach). */
export function reviewRange(
  kind: ReviewKind,
  today: Date,
  periodsAgo: number,
): { from: string; to: string } {
  if (kind === "week") {
    const monday = addDays(today, -((today.getDay() + 6) % 7) - 7 * periodsAgo);
    return { from: toIsoDate(monday), to: toIsoDate(addDays(monday, 6)) };
  }
  const length = MONTHS_PER_KIND[kind];
  const startMonth =
    today.getMonth() - (today.getMonth() % length) - length * periodsAgo;
  const from = new Date(today.getFullYear(), startMonth, 1, 12);
  const to = new Date(today.getFullYear(), startMonth + length, 0, 12);
  return { from: toIsoDate(from), to: toIsoDate(to) };
}

/** `02.03. – 08.03. 2026` */
export function rangeLabel(from: string, to: string): string {
  return t("review.rangeLabel", {
    start: shortDate(from),
    end: shortDate(to),
    year: String(parseIsoDate(to).getFullYear()),
  });
}

/** Änderung gegenüber der Vorperiode: `+45m`, `−40m`, `±0m`. */
export function deltaText(
  review: Pick<Review, "focusMin" | "previousFocusMin">,
): string {
  const delta = review.focusMin - review.previousFocusMin;
  if (delta === 0) return `±${formatMinutes(0)}`;
  return `${delta > 0 ? "+" : "−"}${formatMinutes(Math.abs(delta))}`;
}
