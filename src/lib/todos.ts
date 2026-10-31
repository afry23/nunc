import type { Todo, TodoImportResult } from "../bindings";
import { t } from "../i18n";

export type Segment = { text: string; kind: "plain" | "project" | "context" };

/** Zerlegt eine Beschreibung in Text, `+projekt`- und `@kontext`-Tags (Leerraum bleibt erhalten). */
export function splitSegments(description: string): Segment[] {
  return description
    .split(/(\s+)/)
    .filter((part) => part !== "")
    .map((part): Segment => {
      if (part.length > 1 && part.startsWith("+"))
        return { text: part, kind: "project" };
      if (part.length > 1 && part.startsWith("@"))
        return { text: part, kind: "context" };
      return { text: part, kind: "plain" };
    })
    .reduce<Segment[]>((merged, segment) => {
      const last = merged[merged.length - 1];
      if (last && last.kind === "plain" && segment.kind === "plain") {
        last.text += segment.text;
      } else {
        merged.push({ ...segment });
      }
      return merged;
    }, []);
}

/** Beschreibung ohne Tags für die Anzeige im Titel (z. B. Auswahlliste); Tags bleiben im Rohtext. */
export function titleOf(todo: Pick<Todo, "description">): string {
  const words = todo.description
    .split(/\s+/)
    .filter(
      (w) =>
        w !== "" && !(w.length > 1 && (w.startsWith("+") || w.startsWith("@"))),
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
