import type { Todo } from "../bindings";

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

/** Farb-Token der Priorität (A kritisch, B mittel, sonst gedämpft). */
export function priorityTone(
  priority: string | null,
): "high" | "medium" | "low" | "none" {
  if (priority === "A") return "high";
  if (priority === "B") return "medium";
  return priority ? "low" : "none";
}
