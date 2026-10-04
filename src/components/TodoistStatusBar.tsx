import { useTodoistStore } from "../stores/todoistStore";
import { todoistStatusText } from "./TodoistSection";

/** Schmale Statuszeile für Todoist (nur sichtbar, solange ein Konto verbunden ist). */
export function TodoistStatusBar() {
  const connected = useTodoistStore((s) => s.account?.connected ?? false);
  const status = useTodoistStore((s) => s.status);
  if (!connected) return null;

  return (
    <div
      role="status"
      aria-label="Todoist-Status"
      data-testid="todoist-status"
      data-state={status.state}
      title={status.errorMessage ?? undefined}
      className={`flex min-h-9 items-center bg-surface-container-low px-4 py-1 text-sm ${status.state === "error" ? "text-red-300" : "text-on-surface-variant"}`}
    >
      {todoistStatusText(status.state, status.lastSyncAt)}
    </div>
  );
}
