import type { MessageKey } from "../i18n";

export type ShortcutAction =
  | "toggle-timer"
  | "stop"
  | "new-todo"
  | "go-timer"
  | "go-todos"
  | "go-times"
  | "go-stats"
  | "go-settings"
  | "go-help";

export type ShortcutEvent = {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  target: EventTarget | null;
};

const TYPING = new Set(["INPUT", "TEXTAREA", "SELECT"]);
const ACTIVATING = new Set(["BUTTON", "A", "SUMMARY"]);

/**
 * Welche Aktion löst eine Taste aus (Desktop)? Nichts passiert bei Modifikatoren, beim Tippen in Eingabefeldern
 * und – für Leertaste und Enter – auf Schaltflächen und Links, deren eigene Aktivierung Vorrang hat.
 */
export function shortcutFor(
  event: ShortcutEvent,
  dialogOpen = false,
): ShortcutAction | null {
  if (event.ctrlKey || event.metaKey || event.altKey || dialogOpen) return null;
  const target = event.target instanceof HTMLElement ? event.target : null;
  if (target && (TYPING.has(target.tagName) || target.isContentEditable))
    return null;
  const activating = target
    ? ACTIVATING.has(target.tagName) || target.getAttribute("role") === "button"
    : false;

  switch (event.key) {
    case " ":
      return activating ? null : "toggle-timer";
    case "Escape":
      return "stop";
    case "n":
    case "N":
      return "new-todo";
    case "1":
      return "go-timer";
    case "2":
      return "go-todos";
    case "3":
      return "go-times";
    case "4":
      return "go-stats";
    case ",":
      return "go-settings";
    case "?":
      return "go-help";
    default:
      return null;
  }
}

/** Anzeige der Kürzel in der Hilfe; `shortcutFor` bleibt die Quelle der Belegung. */
export const SHORTCUTS: { keys: string; label: MessageKey }[] = [
  { keys: "Space", label: "help.shortcuts.toggle" },
  { keys: "Esc", label: "help.shortcuts.stop" },
  { keys: "N", label: "help.shortcuts.newTodo" },
  { keys: "1", label: "help.shortcuts.timer" },
  { keys: "2", label: "help.shortcuts.todos" },
  { keys: "3", label: "help.shortcuts.times" },
  { keys: "4", label: "help.shortcuts.stats" },
  { keys: ",", label: "help.shortcuts.settings" },
  { keys: "?", label: "help.shortcuts.help" },
];
