import type { Todo, TodoImportResult } from "../bindings";
import { t } from "../i18n";

export type Segment =
  | { text: string; kind: "plain" | "project" | "context" | "field" }
  | { text: string; kind: "link"; href: string };

/** Markdown-Link `[Text](https://…)` oder nackte Adresse; `)` beendet eine nackte Adresse. */
const LINK = /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)|https?:\/\/[^\s)]+/g;

/** Kurzform einer nackten Adresse für die Anzeige: nur der Host. */
function hostOf(url: string): string {
  return url.replace(/^https?:\/\//, "").split(/[/?#]/)[0] ?? url;
}

function wordSegments(text: string): Segment[] {
  return text
    .split(/(\s+)/)
    .filter((part) => part !== "")
    .map((part): Segment => {
      if (part.length > 1 && part.startsWith("+"))
        return { text: part, kind: "project" };
      if (part.length > 1 && part.startsWith("@"))
        return { text: part, kind: "context" };
      if (PLAN_WORD.test(part)) return { text: part, kind: "field" };
      return { text: part, kind: "plain" };
    });
}

/**
 * Zerlegt eine Beschreibung in Text, `+projekt`-/`@kontext`-Tags, Links (Markdown oder nackt, mit Anzeigetext) und
 * Planungsfelder (`kind: "field"`, werden nicht angezeigt). Leerraum bleibt erhalten.
 */
export function splitSegments(description: string): Segment[] {
  const parts: Segment[] = [];
  let last = 0;
  for (const match of description.matchAll(LINK)) {
    parts.push(...wordSegments(description.slice(last, match.index)));
    const href = match[2] ?? match[0];
    parts.push({ text: match[1] ?? hostOf(href), kind: "link", href });
    last = match.index + match[0].length;
  }
  parts.push(...wordSegments(description.slice(last)));
  return parts.reduce<Segment[]>((merged, segment) => {
    const prev = merged[merged.length - 1];
    if (prev && prev.kind === "plain" && segment.kind === "plain") {
      prev.text += segment.text;
    } else {
      merged.push({ ...segment });
    }
    return merged;
  }, []);
}

/** Planungsfelder des Tagesplaners (`due:`, `plan:`, `est:`, `rec:`, ADR 0027, 0036); nur zum Ausblenden im Titel. */
const PLAN_WORD =
  /^(?:(?:due|plan):\d{4}-\d{2}-\d{2}|estp?:\d+|rec:\+?\d+[dbwmy])$/;

/** Fälligkeitsdatum (`due:YYYY-MM-DD`) einer Beschreibung, sonst `null`. */
export function dueOf(description: string): string | null {
  return (
    /(?:^|\s)due:(\d{4}-\d{2}-\d{2})(?=\s|$)/.exec(description)?.[1] ?? null
  );
}

/** Beschreibung ohne Tags und Planungsfelder für die Anzeige im Titel (z. B. Auswahlliste); sie bleiben im Rohtext. */
export function titleOf(todo: Pick<Todo, "description">): string {
  const words = todo.description
    .split(/\s+/)
    .filter(
      (w) =>
        w !== "" &&
        !(w.length > 1 && (w.startsWith("+") || w.startsWith("@"))) &&
        !PLAN_WORD.test(w),
    );
  return words.join(" ") || todo.description.trim();
}

/** Todos für die Aufgabenauswahl: offene plus das aktuell gewählte (auch wenn erledigt). */
export function selectableTodos(
  todos: Todo[],
  currentId: number | null,
): Todo[] {
  return todos.filter((todo) => !todo.completed || todo.id === currentId);
}

/** Alle vorkommenden Tags (mit Präfix), alphabetisch und ohne Duplikate. */
export function collectTags(todos: Todo[]): {
  projects: string[];
  contexts: string[];
} {
  const unique = (values: string[]): string[] =>
    [...new Set(values)].sort((a, b) => a.localeCompare(b));
  return {
    projects: unique(todos.flatMap((t) => t.projectTags)),
    contexts: unique(todos.flatMap((t) => t.contextTags)),
  };
}

/**
 * Neue Gesamtreihenfolge nach dem Verschieben in einer gefilterten Ansicht: Die sichtbaren Todos belegen
 * weiter genau ihre bisherigen Plätze in `allIds`, nur in der neuen Reihenfolge. Unsichtbare behalten ihren Platz.
 */
export function mergeOrder(
  allIds: number[],
  newVisibleOrder: number[],
): number[] {
  const visible = new Set(newVisibleOrder);
  let next = 0;
  return allIds.map((id) => (visible.has(id) ? newVisibleOrder[next++] : id));
}

/** Verschiebt das Element an `from` nach `to` (neues Array). Ungültige Indizes ändern nichts. */
export function moveItem<T>(items: T[], from: number, to: number): T[] {
  if (
    from === to ||
    from < 0 ||
    to < 0 ||
    from >= items.length ||
    to >= items.length
  ) {
    return items;
  }
  const next = [...items];
  const [moved] = next.splice(from, 1);
  next.splice(to, 0, moved);
  return next;
}

/** Eine Zeile des Todo-Baums: Tiefe, Kinder und Pomodoros des ganzen Teilbaums. */
export type TreeRow = {
  todo: Todo;
  depth: number;
  hasChildren: boolean;
  /** Pomodoros des Todos und aller Nachfahren (nur Anzeige). */
  pomodoroTotal: number;
  /** Zeilenindex des Elternknotens in der Liste, `null` bei Wurzeln. */
  parentIndex: number | null;
};

/**
 * Ordnet die Liste als Baum (Tiefensuche): Kinder stehen direkt unter ihrem Elternknoten, Geschwister behalten
 * die Reihenfolge der Eingabe. Ein Todo, dessen Elternknoten nicht in der Liste steht, ist eine Wurzel.
 * Eingeklappte Todos (`collapsed`) zeigen ihre Nachfahren nicht, zählen sie aber in `pomodoroTotal`.
 */
export function buildTree(
  todos: Todo[],
  collapsed: ReadonlySet<number> = new Set(),
): TreeRow[] {
  const ids = new Set(todos.map((todo) => todo.id));
  const children = new Map<number | null, Todo[]>();
  for (const todo of todos) {
    const parent =
      todo.parentId != null && ids.has(todo.parentId) ? todo.parentId : null;
    children.set(parent, [...(children.get(parent) ?? []), todo]);
  }
  const total = (todo: Todo, seen: Set<number>): number => {
    if (seen.has(todo.id)) return 0;
    seen.add(todo.id);
    return (children.get(todo.id) ?? []).reduce(
      (sum, child) => sum + total(child, seen),
      todo.pomodoroCount,
    );
  };
  const rows: TreeRow[] = [];
  const walk = (
    todo: Todo,
    depth: number,
    parentIndex: number | null,
  ): void => {
    const kids = children.get(todo.id) ?? [];
    const index = rows.length;
    rows.push({
      todo,
      depth,
      hasChildren: kids.length > 0,
      pomodoroTotal: 0,
      parentIndex,
    });
    if (collapsed.has(todo.id)) return;
    for (const kid of kids) walk(kid, depth + 1, index);
  };
  for (const root of children.get(null) ?? []) walk(root, 0, null);
  for (const row of rows) row.pomodoroTotal = total(row.todo, new Set());
  return rows;
}

/** Neuer Elternknoten beim Einrücken: das vorherige Geschwister (gleiche Tiefe, selber Elternknoten) oder `null`. */
export function indentParent(rows: TreeRow[], index: number): number | null {
  const row = rows[index];
  if (!row) return null;
  for (let i = index - 1; i >= 0; i--) {
    if (rows[i].depth < row.depth) return null;
    if (rows[i].depth === row.depth) return rows[i].todo.id;
  }
  return null;
}

/** Neuer Elternknoten beim Ausrücken: der Elternknoten des Elternknotens; `undefined`, wenn schon Wurzel. */
export function outdentParent(
  rows: TreeRow[],
  index: number,
): number | null | undefined {
  const parentIndex = rows[index]?.parentIndex ?? null;
  if (parentIndex === null) return undefined;
  const grand = rows[parentIndex].parentIndex;
  return grand === null ? null : rows[grand].todo.id;
}

/** Farb-Token der Priorität (A kritisch, B mittel, sonst gedämpft). */
export function priorityTone(
  priority: string | null,
): "high" | "medium" | "low" | "none" {
  if (priority === "A") return "high";
  if (priority === "B") return "medium";
  return priority ? "low" : "none";
}

/** Meldung zum Ergebnis eines Imports: „3 Aufgaben importiert, 1 Duplikat übersprungen.“ */
export function describeImport({
  added,
  duplicates,
}: TodoImportResult): string {
  if (added === 0 && duplicates === 0) return t("todos.import.none");
  const parts = [t("todos.import.added", { count: added })];
  if (duplicates > 0)
    parts.push(t("todos.import.duplicates", { count: duplicates }));
  return `${parts.join(", ")}.`;
}

/** Fortschritt der direkten Unteraufgaben: erledigte und alle; `null` ohne Unteraufgaben. */
export function subtaskProgress(
  todos: readonly Todo[],
  id: number,
): { done: number; total: number } | null {
  const kids = todos.filter((todo) => todo.parentId === id);
  if (kids.length === 0) return null;
  return {
    done: kids.filter((todo) => todo.completed).length,
    total: kids.length,
  };
}

/** Rohtext für eine Kopie: eine Erledigt-Markierung (`x 2026-03-03 …`) entfällt, die Kopie ist offen. */
export function duplicateText(rawText: string): string {
  return rawText.replace(/^x \d{4}-\d{2}-\d{2}( \d{4}-\d{2}-\d{2})? /, "");
}
