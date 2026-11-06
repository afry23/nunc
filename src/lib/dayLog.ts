import type { TimelineBlock, Todo } from "./api";
import { titleOf } from "./todos";

/** Womit die Einträge eines Tages gruppiert werden; `none` ist die Zeitliste ohne Gruppen. */
export type DayGroupBy = "none" | "project" | "context" | "todo";

export type DayGroup = {
  /** Anzeigename; `null` für „nicht zugeordnet“ bzw. „ohne Aufgabe“. */
  label: string | null;
  minutes: number;
  blocks: TimelineBlock[];
};

/** Länge des Blocks in Minuten (gerundet). */
export function blockMinutes(block: TimelineBlock): number {
  return Math.max(
    0,
    Math.round((Date.parse(block.end) - Date.parse(block.start)) / 60000),
  );
}

/**
 * Tags, die für einen Block gelten: mit Aufgabe die der Aufgabe, sonst die eigenen der Session (ADR 0023).
 * Fehlt die Aufgabe (gelöscht), zählen die eigenen.
 */
function tagsOf(
  block: TimelineBlock,
  todo: Todo | undefined,
  by: "project" | "context",
): string[] {
  if (todo) return by === "project" ? todo.projectTags : todo.contextTags;
  return by === "project" ? block.projectTags : block.contextTags;
}

/**
 * Gruppiert Arbeitsblöcke eines Tages nach Projekt, Kontext oder Aufgabe. Ein Block mit mehreren Tags steht in jeder
 * ihrer Gruppen (wie in der Wochenübersicht); ohne Tag bzw. ohne Aufgabe kommt er in die Gruppe mit `label: null`,
 * die zuletzt steht. Die übrigen Gruppen sind nach Minuten absteigend sortiert, dann nach Name. Die Blöcke bleiben in
 * der Reihenfolge der Eingabe. Pausen zählen nicht.
 */
export function groupBlocks(
  blocks: readonly TimelineBlock[],
  todos: readonly Todo[],
  by: Exclude<DayGroupBy, "none">,
): DayGroup[] {
  const byId = new Map(todos.map((todo) => [todo.id, todo]));
  const groups = new Map<string | null, DayGroup>();
  const add = (label: string | null, block: TimelineBlock): void => {
    const group = groups.get(label) ?? { label, minutes: 0, blocks: [] };
    group.minutes += blockMinutes(block);
    group.blocks.push(block);
    groups.set(label, group);
  };
  for (const block of blocks) {
    if (block.sessionType !== "work") continue;
    const todo = block.todoId === null ? undefined : byId.get(block.todoId);
    if (by === "todo") {
      add(todo ? titleOf(todo) : null, block);
      continue;
    }
    const tags = tagsOf(block, todo, by);
    if (tags.length === 0) add(null, block);
    else for (const tag of new Set(tags)) add(tag, block);
  }
  return [...groups.values()].sort((a, b) => {
    if (a.label === null) return 1;
    if (b.label === null) return -1;
    return b.minutes - a.minutes || a.label.localeCompare(b.label);
  });
}
