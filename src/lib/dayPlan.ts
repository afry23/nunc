import type { DayPlan, PlanItem, TimelineBlock, Todo } from "./api";
import { addDays, parseIsoDate, toIsoDate } from "./format";

/** Der Tag nach `day` (`YYYY-MM-DD`). */
export function nextDay(day: string): string {
  return toIsoDate(addDays(parseIsoDate(day), 1));
}

/** Ein Eintrag des Tagesplans samt seinem Todo. */
export type PlanRow = { item: PlanItem; todo: Todo };

/** Verbindet die Einträge mit der Todo-Liste; Einträge, deren Todo (noch) fehlt, entfallen. */
export function planRows(plan: DayPlan, todos: readonly Todo[]): PlanRow[] {
  const byId = new Map(todos.map((todo) => [todo.id, todo]));
  return plan.items.flatMap((item) => {
    const todo = byId.get(item.todoId);
    return todo ? [{ item, todo }] : [];
  });
}

/** Füllstand des Kapazitätsbalkens in Prozent, auf 0–100 begrenzt. */
export function fillPercent(estimateMin: number, capacityMin: number): number {
  if (capacityMin <= 0) return estimateMin > 0 ? 100 : 0;
  return Math.min(
    100,
    Math.max(0, Math.round((estimateMin / capacityMin) * 100)),
  );
}

/** Was der Tagesabschluss vorfindet: eingeplante Einträge (die er ändern kann) und solche mit Frist (die er nicht anfasst). */
export function closeSummary(plan: DayPlan): {
  planned: number;
  fixed: number;
} {
  const planned = plan.items.filter((item) => item.reason === "planned").length;
  return { planned, fixed: plan.items.length - planned };
}

/**
 * Erfasste Arbeit eines Tages in Minuten: Vereinigung der Arbeitsblöcke (Pausen zählen nicht, überlappende
 * Blöcke zählen nur einmal).
 */
export function trackedMinutes(blocks: readonly TimelineBlock[]): number {
  const spans = blocks
    .filter((block) => block.sessionType === "work")
    .map((block) => [Date.parse(block.start), Date.parse(block.end)] as const)
    .filter(([start, end]) => end > start)
    .sort((a, b) => a[0] - b[0]);
  let total = 0;
  let until = Number.NEGATIVE_INFINITY;
  for (const [start, end] of spans) {
    const from = Math.max(start, until);
    if (end > from) total += end - from;
    until = Math.max(until, end);
  }
  return Math.round(total / 60000);
}
