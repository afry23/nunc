import type { Goal, HabitItem, HabitProgress, Schedule } from "./api";
import { addDays, parseIsoDate, toIsoDate } from "./format";

/** Eingabestand des Formulars für eine Gewohnheit (ADR 0028). */
export type HabitDraft = {
  name: string;
  schedule: Schedule["kind"];
  /** Wochentage für `days`, 1 = Montag … 7 = Sonntag. */
  days: number[];
  perWeek: number;
  goal: Goal["kind"];
  minutes: number;
  tag: string;
};

export const EMPTY_DRAFT: HabitDraft = {
  name: "",
  schedule: "daily",
  days: [1, 2, 3, 4, 5],
  perWeek: 3,
  goal: "check",
  minutes: 30,
  tag: "",
};

/** Formularstand einer vorhandenen Gewohnheit; unbekannte Kodierungen fallen auf die Vorgaben zurück. */
export function draftOf(item: HabitItem): HabitDraft {
  const { schedule, goal } = item;
  return {
    ...EMPTY_DRAFT,
    name: item.name,
    ...(schedule?.kind === "days" && {
      schedule: "days" as const,
      days: schedule.days,
    }),
    ...(schedule?.kind === "weekly" && {
      schedule: "weekly" as const,
      perWeek: schedule.perWeek,
    }),
    ...(goal?.kind === "minutes" && {
      goal: "minutes" as const,
      minutes: goal.minutes,
      tag: goal.tag,
    }),
  };
}

export function scheduleOf(draft: HabitDraft): Schedule {
  if (draft.schedule === "days") return { kind: "days", days: draft.days };
  if (draft.schedule === "weekly")
    return { kind: "weekly", perWeek: draft.perWeek };
  return { kind: "daily" };
}

export function goalOf(draft: HabitDraft): Goal {
  return draft.goal === "minutes"
    ? { kind: "minutes", minutes: draft.minutes, tag: draft.tag.trim() }
    : { kind: "check" };
}

/** Schaltet einen Wochentag um; das Ergebnis ist aufsteigend sortiert. */
export function toggleDay(days: readonly number[], day: number): number[] {
  const next = days.includes(day)
    ? days.filter((d) => d !== day)
    : [...days, day];
  return next.sort((a, b) => a - b);
}

/** Reicht die Eingabe zum Speichern? Den Rest (Grenzen) prüft das Backend. */
export function isDraftValid(draft: HabitDraft): boolean {
  if (draft.name.trim() === "") return false;
  if (draft.schedule === "days" && draft.days.length === 0) return false;
  if (draft.goal === "minutes" && draft.tag.trim() === "") return false;
  return true;
}

/** Wird die Gewohnheit aus den Sessions berechnet (kein Haken von Hand)? */
export function isAuto(item: HabitItem): boolean {
  return item.goal?.kind === "minutes";
}

/** Meldungsschlüssel der Serie, passend zu Einheit und Anzahl. */
export function streakKey(
  progress: Pick<HabitProgress, "unit" | "currentStreak">,
):
  | "habits.streak.day"
  | "habits.streak.days"
  | "habits.streak.week"
  | "habits.streak.weeks" {
  const one = progress.currentStreak === 1;
  if (progress.unit === "weeks")
    return one ? "habits.streak.week" : "habits.streak.weeks";
  return one ? "habits.streak.day" : "habits.streak.days";
}

export type RecentCell = { day: string; done: boolean };

/** Die letzten `count` Tage bis einschließlich `today` (aufsteigend), je mit Kennzeichen „erledigt“. */
export function recentCells(
  recent: readonly string[],
  today: string,
  count = 14,
): RecentCell[] {
  const done = new Set(recent);
  const end = parseIsoDate(today);
  return Array.from({ length: count }, (_, index) => {
    const day = toIsoDate(addDays(end, index - count + 1));
    return { day, done: done.has(day) };
  });
}

/** Aktive und archivierte Gewohnheiten, je in der Reihenfolge der Eingabe. */
export function splitHabits(items: readonly HabitItem[]): {
  active: HabitItem[];
  archived: HabitItem[];
} {
  return {
    active: items.filter((item) => !item.archived),
    archived: items.filter((item) => item.archived),
  };
}

/** Gewohnheiten für „Heute“: nicht archiviert und heute fällig oder erledigt; offene zuerst. */
export function todayHabits(items: readonly HabitItem[]): HabitItem[] {
  const due = items.filter(
    (item) =>
      !item.archived &&
      item.progress !== null &&
      (item.progress.openToday || item.progress.doneToday),
  );
  const open = due.filter((item) => item.progress?.openToday);
  return [...open, ...due.filter((item) => !item.progress?.openToday)];
}
