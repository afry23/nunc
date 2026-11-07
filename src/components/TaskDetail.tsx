import { useEffect, useState } from "react";
import { commands, unwrap, type TaskTime } from "../lib/api";
import { useT } from "../i18n";
import {
  estimateOf,
  formatDelta,
  formatEstimate,
  formatInUnit,
  progressPercent,
  verdictLabel,
} from "../lib/estimate";
import { formatDateTime, formatMinutes } from "../lib/format";
import { subtaskProgress, titleOf } from "../lib/todos";
import { usePlanStore } from "../stores/planStore";
import { useTimerStore } from "../stores/timerStore";
import { useTodoStore } from "../stores/todoStore";
import { TaskEditDialog } from "./TaskEditDialog";
import { StartButton } from "./StartButton";
import { Button } from "./ui";

/** Details einer Aufgabe: Beschreibung, Eigenschaften und die rohe todo.txt-Zeile zum Bearbeiten. */
export function TaskDetail({ id }: { id: number }) {
  const t = useT();
  const all = useTodoStore((s) => s.all);
  const todo = all.find((item) => item.id === id);
  const planItem = usePlanStore((s) =>
    s.plan?.items.find((item) => item.todoId === id),
  );

  const [editing, setEditing] = useState(false);
  const timerState = useTimerStore((s) => s.snapshot.state);
  const [time, setTime] = useState<TaskTime | null>(null);
  // neu laden, wenn der Timer anhält: dann ist eine Session dazugekommen
  useEffect(() => {
    let cancelled = false;
    void unwrap(commands.getTaskTime(id)).then(
      (loaded) => {
        if (!cancelled) setTime(loaded);
      },
      () => undefined,
    );
    return () => {
      cancelled = true;
    };
  }, [id, timerState]);

  if (!todo) {
    return (
      <p className="rounded-xl bg-surface-container-low p-6 text-on-surface-variant">
        {t("task.gone")}
      </p>
    );
  }

  const progress = subtaskProgress(all, id);
  const estimate = estimateOf(todo.description);
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

      <div className="flex flex-wrap items-center gap-3">
        {!todo.completed && (
          <StartButton
            todoId={todo.id}
            title={titleOf(todo)}
            variant="button"
          />
        )}
        <Button onClick={() => setEditing(true)}>{t("taskEdit.open")}</Button>
      </div>

      <p>
        <span className="text-on-surface-variant">{t("estimate.label")}: </span>
        {estimate ? formatEstimate(estimate) : t("estimate.none")}
      </p>

      {time?.estimateCheck && (
        <section
          aria-label={t("estimate.label")}
          className="flex flex-col gap-1"
        >
          <p>
            {t("estimate.progress", {
              actual: formatInUnit(
                time.estimateCheck.estimate.unit,
                time.estimateCheck.actual,
              ),
              estimate: formatEstimate(time.estimateCheck.estimate),
              delta: formatDelta(time.estimateCheck),
            })}
          </p>
          <div
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={progressPercent(time.estimateCheck)}
            aria-label={verdictLabel(time.estimateCheck.verdict)}
            className="h-2 overflow-hidden rounded-full bg-surface-container-highest"
          >
            <div
              className={`h-full rounded-full ${time.estimateCheck.verdict === "over" ? "bg-error" : "bg-primary"}`}
              style={{ width: `${progressPercent(time.estimateCheck)}%` }}
            />
          </div>
          <p className="text-on-surface-variant">
            {verdictLabel(time.estimateCheck.verdict)}
          </p>
        </section>
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

      <section aria-label={t("task.time")} className="flex flex-col gap-2">
        <h3 className="label-md text-on-surface-variant">{t("task.time")}</h3>
        {time === null || time.entries.length === 0 ? (
          <p className="text-on-surface-variant">{t("task.time.none")}</p>
        ) : (
          <>
            <p className="font-semibold">
              {t("task.time.total", { time: formatMinutes(time.totalMin) })}
            </p>
            <ul className="flex flex-col gap-1">
              {time.entries.map((entry) => (
                <li
                  key={entry.sessionId}
                  className="flex flex-col gap-0.5 sm:flex-row sm:gap-4"
                >
                  <span className="text-on-surface-variant tabular-nums">
                    {formatDateTime(entry.startedAt)}
                  </span>
                  <span className="tabular-nums">
                    {formatMinutes(entry.minutes)}
                  </span>
                  {entry.note !== "" && (
                    <span className="break-words">{entry.note}</span>
                  )}
                </li>
              ))}
            </ul>
          </>
        )}
      </section>
      {editing && (
        <TaskEditDialog todo={todo} onClose={() => setEditing(false)} />
      )}
    </article>
  );
}
