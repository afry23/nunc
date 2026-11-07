import { useNavigate } from "@tanstack/react-router";
import { useT } from "../i18n";
import type { TimerMode } from "../lib/api";
import { useSettingsStore } from "../stores/settingsStore";
import { useTimerStore } from "../stores/timerStore";
import { ActionMenu } from "./ActionMenu";
import { ChevronDownIcon, PlayIcon } from "./icons";
import { ActionButton, IconButton } from "./ui";

/**
 * Geteilter Start-Knopf für eine Aufgabe: ▶ startet sofort im zuletzt benutzten Modus, der Pfeil daneben öffnet die
 * Modi (Pomodoro, Flowtime, Stoppuhr); die Wahl startet ebenfalls sofort und wird der neue Standard. Danach öffnet sich
 * der Timer, damit man gleich im Fokus ist. Läuft der Start nicht an (Fehler), bleibt die Ansicht.
 */
export function StartButton({
  todoId,
  title,
  variant = "icon",
}: {
  todoId: number;
  title: string;
  variant?: "icon" | "button";
}) {
  const t = useT();
  const navigate = useNavigate();
  const remembered =
    useSettingsStore((s) => s.settings?.activeMode) ?? "pomodoro";
  const modes: { value: TimerMode; label: string }[] = [
    { value: "pomodoro", label: t("timer.mode.pomodoro") },
    { value: "flowtime", label: t("timer.mode.flowtime") },
    { value: "stopwatch", label: t("timer.mode.stopwatch") },
  ];

  const start = async (mode?: TimerMode): Promise<void> => {
    await useTimerStore.getState().startTodo(todoId, mode);
    if (useTimerStore.getState().snapshot.state !== "idle")
      void navigate({ to: "/focus" });
  };

  return (
    <span className="inline-flex items-center">
      {variant === "button" ? (
        <ActionButton
          icon={<PlayIcon />}
          variant="primary"
          onClick={() => void start()}
        >
          {t("task.start")}
        </ActionButton>
      ) : (
        <IconButton
          label={t("todos.item.start", { title })}
          onClick={() => void start()}
        >
          <PlayIcon />
        </IconButton>
      )}
      <ActionMenu
        label={t("start.mode", { title })}
        icon={<ChevronDownIcon />}
        triggerClassName="!w-7"
        actions={modes.map((mode) => ({
          label: mode.label,
          checked: mode.value === remembered,
          onSelect: () => void start(mode.value),
        }))}
      />
    </span>
  );
}
