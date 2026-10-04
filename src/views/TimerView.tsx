import { useEffect } from "react";
import { Button, Segmented } from "../components/ui";
import { useT } from "../i18n";
import { NotificationHint } from "../components/NotificationHint";
import { RingTimer } from "../components/RingTimer";
import type { TimerMode } from "../lib/api";
import { sessionLabel } from "../lib/timer";
import { titleOf } from "../lib/todos";
import { useSettingsStore } from "../stores/settingsStore";
import { useTimerStore } from "../stores/timerStore";
import { useTodoStore } from "../stores/todoStore";

export function TimerView() {
  const t = useT();
  const modes: { value: TimerMode; label: string }[] = [
    { value: "pomodoro", label: t("timer.mode.pomodoro") },
    { value: "flowtime", label: t("timer.mode.flowtime") },
  ];
  const snapshot = useTimerStore((s) => s.snapshot);
  const suggestedBreak = useTimerStore((s) => s.suggestedBreak);
  // solange der Dialog den Vorschlag zeigt, entfällt der Hinweis darunter
  const dialogOpen = useTimerStore(
    (s) =>
      s.finish?.kind === "completed" &&
      s.finish.payload.suggestedBreak !== null,
  );
  const selectedTodoId = useTimerStore((s) => s.selectedTodoId);
  const { start, startBreak, pause, resume, stop, selectTodo } =
    useTimerStore.getState();
  const settings = useSettingsStore((s) => s.settings);
  const patchSettings = useSettingsStore((s) => s.patch);
  const todos = useTodoStore((s) => s.all);

  const idle = snapshot.state === "idle";
  const mode = idle ? (settings?.activeMode ?? "pomodoro") : snapshot.mode;
  const onBreak = snapshot.sessionType !== "work";
  const openTodos = todos.filter((t) => !t.completed);
  const activeTodo = todos.find(
    (t) => t.id === (idle ? selectedTodoId : snapshot.todoId),
  );

  // ein erledigtes oder gelöschtes Todo bleibt nicht ausgewählt
  useEffect(() => {
    if (
      selectedTodoId !== null &&
      !openTodos.some((t) => t.id === selectedTodoId)
    )
      selectTodo(null);
  }, [selectedTodoId, openTodos, selectTodo]);

  const idleSeconds =
    mode === "pomodoro" ? (settings?.pomodoroMin ?? 25) * 60 : 0;

  return (
    <section
      aria-labelledby="timer-heading"
      className="mx-auto flex max-w-xl flex-col items-center gap-8 px-6 py-10"
    >
      <h1 id="timer-heading" className="sr-only">
        {t("timer.title")}
      </h1>

      <NotificationHint />

      <Segmented
        label={t("timer.mode.label")}
        value={mode}
        options={modes}
        disabled={!idle}
        onChange={(value) => void patchSettings({ activeMode: value })}
      />

      <RingTimer snapshot={snapshot} idleSeconds={idleSeconds} />

      <div className="flex flex-wrap justify-center gap-3">
        {idle && (
          <Button
            variant="primary"
            className="min-w-48 py-4 text-lg"
            onClick={() => void start(mode)}
          >
            {t("timer.start")}
          </Button>
        )}
        {snapshot.state === "running" && (
          <Button
            variant="secondary"
            className="min-w-36"
            onClick={() => void pause()}
          >
            {t("timer.pause")}
          </Button>
        )}
        {snapshot.state === "paused" && (
          <Button
            variant="primary"
            className="min-w-36"
            onClick={() => void resume()}
          >
            {t("timer.resume")}
          </Button>
        )}
        {!idle && (
          <Button
            variant="secondary"
            className="min-w-36"
            onClick={() => void stop()}
          >
            {onBreak ? t("timer.skip") : t("timer.stop")}
          </Button>
        )}
      </div>

      {idle && suggestedBreak && !dialogOpen && (
        <div className="flex w-full items-center justify-between gap-4 rounded-xl bg-surface-container-low p-4">
          <p>
            {t("timer.suggested")}{" "}
            <strong>
              {t("timer.minutes", { count: suggestedBreak.minutes })}
            </strong>{" "}
            <span className="text-on-surface-variant">
              ({sessionLabel(mode, suggestedBreak.sessionType)})
            </span>
          </p>
          <Button variant="secondary" onClick={() => void startBreak()}>
            {t("timer.startBreak")}
          </Button>
        </div>
      )}

      <div className="w-full">
        <label
          htmlFor="active-todo"
          className="label-md mb-2 block text-center text-on-surface-variant"
        >
          {t("timer.activeTodo")}
        </label>
        {idle ? (
          <select
            id="active-todo"
            value={selectedTodoId ?? ""}
            onChange={(event) =>
              selectTodo(
                event.target.value === "" ? null : Number(event.target.value),
              )
            }
            className="min-h-12 w-full rounded-lg bg-surface-container-highest px-4 text-on-surface focus:bg-secondary-container"
          >
            <option value="">{t("timer.noTodo")}</option>
            {openTodos.map((todo) => (
              <option key={todo.id} value={todo.id}>
                {titleOf(todo)}
                {todo.pomodoroCount > 0 ? ` · 🍅 ${todo.pomodoroCount}` : ""}
              </option>
            ))}
          </select>
        ) : (
          <p
            id="active-todo"
            className="min-h-12 rounded-lg bg-surface-container-low px-4 py-3 text-center"
          >
            {activeTodo ? titleOf(activeTodo) : t("timer.noTodo")}
          </p>
        )}
      </div>
    </section>
  );
}
