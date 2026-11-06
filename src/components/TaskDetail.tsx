import { useState } from "react";
import { useT } from "../i18n";
import { formatMinutes } from "../lib/format";
import { subtaskProgress, titleOf } from "../lib/todos";
import { usePlanStore } from "../stores/planStore";
import { useTimerStore } from "../stores/timerStore";
import { useTodoStore } from "../stores/todoStore";
import { PlayIcon } from "./icons";
import { ActionButton, Button } from "./ui";

/** Details einer Aufgabe: Beschreibung, Eigenschaften und die rohe todo.txt-Zeile zum Bearbeiten. */
export function TaskDetail({ id }: { id: number }) {
  const t = useT();
  const all = useTodoStore((s) => s.all);
  const todo = all.find((item) => item.id === id);
  const planItem = usePlanStore((s) =>
    s.plan?.items.find((item) => item.todoId === id),
  );

  if (!todo) {
    return (
      <p className="rounded-xl bg-surface-container-low p-6 text-on-surface-variant">
        {t("task.gone")}
      </p>
    );
  }

  const progress = subtaskProgress(all, id);
  const rows: [string, string][] = [
    [t("task.priority"), todo.priority ?? "–"],
    [t("task.projects"), todo.projectTags.join(" ") || "–"],
    [t("task.contexts"), todo.contextTags.join(" ") || "–"],
    [t("task.pomodoros"), String(todo.pomodoroCount)],
    ...(progress
      ? [
          [
            t("task.subtasks"),
            t("todos.item.progress", {
              done: progress.done,
              total: progress.total,
            }),
          ] as [string, string],
        ]
      : []),
    ...(planItem
      ? [
          [
            t("task.today"),
            planItem.estimateMin !== null
              ? t("task.estimate", {
                  time: formatMinutes(planItem.estimateMin),
                })
              : t("todos.item.today"),
          ] as [string, string],
        ]
      : []),
  ];

  return (
    <article
      aria-labelledby="task-heading"
      className="flex flex-col gap-5 rounded-xl bg-surface-container-low p-5"
    >
      <header className="flex flex-col gap-1">
        <h2 id="task-heading" className="headline-md">
          {t("task.heading")}
        </h2>
        <p className="font-display text-lg font-bold break-words">
          {titleOf(todo)}
        </p>
      </header>

      {!todo.completed && (
        <div>
          <ActionButton
            icon={<PlayIcon />}
            variant="primary"
            onClick={() => void useTimerStore.getState().startTodo(todo.id)}
          >
            {t("task.start")}
          </ActionButton>
        </div>
      )}

      <section
        aria-label={t("task.description")}
        className="flex flex-col gap-1"
      >
        <h3 className="label-md text-on-surface-variant">
          {t("task.description")}
        </h3>
        <p className="break-all whitespace-pre-wrap">{todo.description}</p>
      </section>

      <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1">
        {rows.map(([name, value]) => (
          <div key={name} className="contents">
            <dt className="text-on-surface-variant">{name}</dt>
            <dd className="break-words">{value}</dd>
          </div>
        ))}
      </dl>

      {/* neuer Schlüssel nach jedem Speichern: das Feld beginnt wieder mit der gespeicherten Zeile */}
      <RawLine key={todo.rawText} id={todo.id} rawText={todo.rawText} />
    </article>
  );
}

function RawLine({ id, rawText }: { id: number; rawText: string }) {
  const t = useT();
  const [draft, setDraft] = useState(rawText);
  const changed = draft.trim() !== rawText.trim();
  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        if (changed) void useTodoStore.getState().update(id, draft);
      }}
      className="flex flex-col gap-2"
    >
      <label className="flex flex-col gap-1">
        <span className="label-md text-on-surface-variant">
          {t("task.raw")}
        </span>
        <textarea
          value={draft}
          rows={2}
          onChange={(event) => setDraft(event.target.value)}
          className="w-full min-w-0 rounded-lg bg-surface-container-highest p-3 font-mono text-sm text-on-surface focus:bg-secondary-container"
        />
      </label>
      <div>
        <Button type="submit" variant="primary" disabled={!changed}>
          {t("task.save")}
        </Button>
      </div>
    </form>
  );
}
