/** Hilfen für Projekte (`+name`) und Kontexte (`@name`) einer Session ohne Aufgabe (ADR 0023). */

export type TagPrefix = "+" | "@";

/**
 * Macht aus einer Eingabe ein Tag mit Präfix. `null`, wenn nichts übrig bleibt (leer, nur Präfix) oder die Eingabe
 * Leerraum enthält. Spiegelt `normalize_tags` im Backend, das beim Speichern maßgeblich ist.
 */
export function toTag(raw: string, prefix: TagPrefix): string | null {
  const word = raw.trim();
  const name = word.startsWith(prefix) ? word.slice(1) : word;
  if (name === "" || /\s/.test(name)) return null;
  return `${prefix}${name}`;
}

/** Hängt `raw` als Tag an `tags` an; ungültige Eingaben und Duplikate ändern nichts. */
export function addTag(
  tags: string[],
  raw: string,
  prefix: TagPrefix,
): string[] {
  const tag = toTag(raw, prefix);
  return tag === null || tags.includes(tag) ? tags : [...tags, tag];
}

/** Entfernt `tag` aus `tags`. */
export function removeTag(tags: string[], tag: string): string[] {
  return tags.filter((candidate) => candidate !== tag);
}

/** Vorschläge, die noch nicht gewählt sind und zur Eingabe passen (ohne Beachtung von Groß-/Kleinschreibung). */
export function matchingSuggestions(
  suggestions: string[],
  chosen: string[],
  input: string,
): string[] {
  const needle = input.trim().replace(/^[+@]/, "").toLowerCase();
  return suggestions.filter(
    (tag) =>
      !chosen.includes(tag) && tag.slice(1).toLowerCase().includes(needle),
  );
}
