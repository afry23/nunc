import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { isMobile } from "../lib/platform";
import { shortcutFor, type ShortcutAction } from "../lib/shortcuts";
import { useSettingsStore } from "../stores/settingsStore";
import { useTimerStore } from "../stores/timerStore";

/** Tastenkürzel (nur Desktop). Space/Esc steuern den Timer, N/1/2/3/,/? navigieren. */
export function useShortcuts(): void {
  const navigate = useNavigate();

  useEffect(() => {
    if (isMobile()) return;

    const run = (action: ShortcutAction): void => {
      const timer = useTimerStore.getState();
      switch (action) {
        case "toggle-timer": {
          if (timer.snapshot.state === "running") void timer.pause();
          else if (timer.snapshot.state === "paused") void timer.resume();
          else
            void timer.start(
              useSettingsStore.getState().settings?.activeMode ?? "pomodoro",
            );
          return;
        }
        case "stop":
          if (timer.snapshot.state !== "idle") void timer.stop();
          return;
        case "new-todo":
          void navigate({ to: "/todos" }).then(() =>
            document.getElementById("todo-input")?.focus(),
          );
          return;
        case "go-timer":
          void navigate({ to: "/" });
          return;
        case "go-todos":
          void navigate({ to: "/todos" });
          return;
        case "go-stats":
          void navigate({ to: "/stats" });
          return;
        case "go-settings":
          void navigate({ to: "/settings" });
          return;
        case "go-help":
          void navigate({ to: "/help" });
      }
    };

    const onKeyDown = (event: KeyboardEvent): void => {
      const action = shortcutFor(
        event,
        document.querySelector("[role=dialog]") !== null,
      );
      if (!action) return;
      event.preventDefault();
      run(action);
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [navigate]);
}
