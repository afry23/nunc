import { t } from "../i18n";
import type { ModeTotals, Review, TaskTotals, Todo } from "../bindings";
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

/** Ab Freitag ist Zeit für den Wochenrückblick (ADR 0026). */
export function isWeekReviewTime(today: Date): boolean {
  const day = today.getDay();
  return day === 0 || day >= 5;
}

/** `day` auf den Zeitraum `from` bis `to` begrenzen (ISO-Tage lassen sich als Text vergleichen). */
export function clampDay(day: string, from: string, to: string): string {
  if (day < from) return from;
  if (day > to) return to;
  return day;
}

/**
 * Fokuszeit im engen Sinn: Zeit mit Pomodoro und Flowtime. Die Gesamtsumme des Rückblicks („Erfasste Arbeitszeit“)
 * enthält auch Stoppuhr und Nachgetragenes (Meetings, freie Tätigkeiten).
 */
export function focusOnlyMinutes(modes: readonly ModeTotals[]): number {
  return modes
    .filter((entry) => entry.mode === "pomodoro" || entry.mode === "flowtime")
    .reduce((sum, entry) => sum + entry.minutes, 0);
}

/** Aufgaben, an denen im Zeitraum gearbeitet wurde und die noch offen sind (in der Reihenfolge der Eingabe). */
export function openTasks(
  tasks: readonly TaskTotals[],
  todos: readonly Todo[],
): TaskTotals[] {
  const open = new Set(
    todos.filter((todo) => !todo.completed).map((todo) => todo.id),
  );
  return tasks.filter((task) => open.has(task.todoId));
}
