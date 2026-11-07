import { useEffect, useState } from "react";
import { commands, unwrap, type TaskTime, type Todo } from "../lib/api";
import { useT } from "../i18n";
import {
  formatDelta,
  formatEstimate,
  formatInUnit,
  progressPercent,
  verdictLabel,
} from "../lib/estimate";
import { formatDateTime, formatMinutes } from "../lib/format";
import { subtaskProgress, titleOf } from "../lib/todos";
import { useTimerStore } from "../stores/timerStore";
import { useTodoStore } from "../stores/todoStore";
import { StartButton } from "./StartButton";

/**
 * Der Infobereich im Aufgaben-Dialog: Start-Knopf, Schätzung gegen bisherigen Aufwand, Fortschritt der
 * Unteraufgaben und die Zeiteinträge der Aufgabe.
 */
export function TaskInfo({ todo }: { todo: Todo }) {
  const t = useT();
  const all = useTodoStore((s) => s.all);
  const timerState = useTimerStore((s) => s.snapshot.state);
  const [time, setTime] = useState<TaskTime | null>(null);
  // neu laden, wenn der Timer anhält: dann ist eine Session dazugekommen
  useEffect(() => {
    let cancelled = false;
    void unwrap(commands.getTaskTime(todo.id)).then(
      (loaded) => {
        if (!cancelled) setTime(loaded);
      },
      () => undefined,
    );
    return () => {
      cancelled = true;
    };
  }, [todo.id, timerState]);

  const progress = subtaskProgress(all, todo.id);
  const check = time?.estimateCheck ?? null;
  const rows: [string, string][] = [
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
  ];

  return (
    <div className="flex flex-col gap-4 text-on-surface">
      {!todo.completed && (
        <div>
          <StartButton
            todoId={todo.id}
            title={titleOf(todo)}
            variant="button"
          />
        </div>
      )}

      {check && (
        <section
          aria-label={t("estimate.label")}
          className="flex flex-col gap-1"
        >
          <p>
            {t("estimate.progress", {
              actual: formatInUnit(check.estimate.unit, check.actual),
              estimate: formatEstimate(check.estimate),
              delta: formatDelta(check),
            })}
          </p>
          <div
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={progressPercent(check)}
            aria-label={verdictLabel(check.verdict)}
            className="h-2 overflow-hidden rounded-full bg-surface-container-highest"
          >
            <div
              className={`h-full rounded-full ${check.verdict === "over" ? "bg-error" : "bg-primary"}`}
              style={{ width: `${progressPercent(check)}%` }}
            />
          </div>
          <p className="text-on-surface-variant">
            {verdictLabel(check.verdict)}
          </p>
        </section>
      )}

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
    </div>
  );
}
