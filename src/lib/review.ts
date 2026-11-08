import { t } from "../i18n";
import type {
  ActivityEntry,
  DayTotals,
  ModeTotals,
  Review,
  TaskTotals,
  Todo,
} from "../bindings";
import type { DayPoint } from "./stats";
import {
  addDays,
  formatMinutes,
  parseIsoDate,
  shortDate,
  toIsoDate,
} from "./format";

/** Reine Hilfen für den Rückblick (ADR 0025). Alle Tage sind lokale Tage (`YYYY-MM-DD`). */

export type ReviewKind = "day" | "week" | "month" | "quarter" | "year";

export const REVIEW_KINDS: ReviewKind[] = [
  "day",
  "week",
  "month",
  "quarter",
  "year",
];

const MONTHS_PER_KIND = { month: 1, quarter: 3, year: 12 } as const;

/** Zeitraum, der `periodsAgo` Wochen/Monate/Quartale vor dem Zeitraum von `today` liegt (negativ: danach). */
export function reviewRange(
  kind: ReviewKind,
  today: Date,
  periodsAgo: number,
): { from: string; to: string } {
  if (kind === "day") {
    const day = toIsoDate(addDays(today, -periodsAgo));
    return { from: day, to: day };
  }
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

/** Ein Balken des Zeitverlaufs: Beginn des Abschnitts, mit den Minuten je Zeitmessung. */
export type Granularity = "day" | "week" | "month";

/** Abschnitte der Balken: Tage bis zu einem Monat, Wochen im Quartal, Monate im Jahr. */
export function granularityFor(kind: ReviewKind): Granularity {
  if (kind === "quarter") return "week";
  if (kind === "year") return "month";
  return "day";
}

function bucketStart(date: string, granularity: Granularity): string {
  if (granularity === "day") return date;
  const day = parseIsoDate(date);
  if (granularity === "month")
    return toIsoDate(new Date(day.getFullYear(), day.getMonth(), 1, 12));
  return toIsoDate(addDays(day, -((day.getDay() + 6) % 7)));
}

/**
 * Tage zu Balken zusammenfassen: jeder Tag von `from` bis `to` (ohne Arbeit mit 0) bei `day`, sonst Summen je
 * Woche (Montag) oder Monat. Die Datumsangabe eines Balkens ist sein Beginn.
 */
export function bucketDays(
  days: readonly DayTotals[],
  from: string,
  to: string,
  granularity: Granularity,
): DayPoint[] {
  const byDate = new Map(days.map((d) => [d.date, d]));
  const buckets = new Map<string, DayPoint>();
  const last = parseIsoDate(to);
  for (let day = parseIsoDate(from); day <= last; day = addDays(day, 1)) {
    const date = toIsoDate(day);
    const key = bucketStart(date, granularity);
    const found = byDate.get(date);
    const bucket = buckets.get(key) ?? {
      date: key,
      pomodoroCount: 0,
      workMinutes: 0,
      breakMinutes: 0,
      pomodoroMinutes: 0,
      flowtimeCount: 0,
      flowtimeMinutes: 0,
      stopwatchCount: 0,
      stopwatchMinutes: 0,
    };
    bucket.pomodoroCount += found?.pomodoros ?? 0;
    bucket.workMinutes += found?.minutes ?? 0;
    bucket.pomodoroMinutes += found?.pomodoroMin ?? 0;
    bucket.flowtimeMinutes += found?.flowtimeMin ?? 0;
    bucket.stopwatchMinutes += found?.stopwatchMin ?? 0;
    buckets.set(key, bucket);
  }
  return [...buckets.values()];
}

/** Gehört etwas mit diesen Projekten zur Auswahl? `none` steht in der Auswahl für „ohne Projekt“. */
function matchesProject(
  projects: readonly string[],
  selected: readonly string[],
  none: string,
): boolean {
  if (selected.length === 0) return true;
  if (projects.length === 0) return selected.includes(none);
  return projects.some((project) => selected.includes(project));
}

/** Aufgaben, die zu mindestens einem der gewählten Projekte gehören (leere Auswahl: alle). */
export function filterTasks(
  tasks: readonly TaskTotals[],
  selected: readonly string[],
  none: string,
): TaskTotals[] {
  return tasks.filter((task) => matchesProject(task.projects, selected, none));
}

/** Freie Tätigkeiten mit mindestens einem der gewählten Projekte (leere Auswahl: alle). */
export function filterActivities(
  activities: readonly ActivityEntry[],
  selected: readonly string[],
  none: string,
): ActivityEntry[] {
  return activities.filter((entry) =>
    matchesProject(entry.projects, selected, none),
  );
}

/** Alle Projekte der Aufgaben und Tätigkeiten, sortiert; `none` zuerst, wenn etwas ohne Projekt vorkommt. */
export function projectOptions(
  tasks: readonly TaskTotals[],
  activities: readonly ActivityEntry[],
  none: string,
): string[] {
  const all = [
    ...tasks.map((task) => task.projects),
    ...activities.map((entry) => entry.projects),
  ];
  const names = [...new Set(all.flat())].sort();
  return all.some((projects) => projects.length === 0)
    ? [none, ...names]
    : names;
}

/** Arbeitsminuten je Stunde als Balken-Daten; zeigt nur Stunden von der ersten bis zur letzten mit Arbeit. */
export function hourRange(hours: readonly number[]): {
  first: number;
  last: number;
} | null {
  const used = hours.flatMap((minutes, hour) => (minutes > 0 ? [hour] : []));
  return used.length === 0
    ? null
    : { first: used[0], last: used[used.length - 1] };
}
