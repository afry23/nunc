import type { ActivityEntry, TaskTotals } from "../lib/api";
import { useT } from "../i18n";
import { formatMinutes, shortDate } from "../lib/format";
import { formatDelta, formatEstimate, formatInUnit } from "../lib/estimate";

function Status({ task }: { task: TaskTotals }) {
  const t = useT();
  return task.completed ? (
    <span className="text-primary">
      ✓{" "}
      {task.completedAt
        ? t("work.done", { date: shortDate(task.completedAt) })
        : null}
    </span>
  ) : (
    <span className="text-on-surface-variant">{t("work.open")}</span>
  );
}

/** Aufgaben, an denen gearbeitet wurde, mit Status (erledigt/offen); `withTime` zeigt zusätzlich Zeit und Schätzung. */
export function TaskList({
  tasks,
  withTime,
  empty,
}: {
  tasks: readonly TaskTotals[];
  withTime: boolean;
  empty: string;
}) {
  const t = useT();
  if (tasks.length === 0)
    return <p className="text-on-surface-variant">{empty}</p>;
  return (
    <ul className="flex flex-col gap-2">
      {tasks.map((task) => (
        <li
          key={task.todoId}
          className="flex flex-wrap items-baseline justify-between gap-x-4"
        >
          <span className="min-w-0 break-words">{task.text}</span>
          <span className="flex flex-wrap gap-x-3 tabular-nums">
            {withTime && (
              <span className="text-on-surface-variant">
                {t("work.taskValue", {
                  time: formatMinutes(task.minutes),
                  sessions: task.sessions,
                })}
              </span>
            )}
            {withTime && task.estimateCheck && (
              <span
                className={
                  task.estimateCheck.verdict === "over"
                    ? "text-error"
                    : "text-on-surface-variant"
                }
              >
                {t("review.estimatePair", {
                  estimate: formatEstimate(task.estimateCheck.estimate),
                  actual: formatInUnit(
                    task.estimateCheck.estimate.unit,
                    task.estimateCheck.actual,
                  ),
                  delta: formatDelta(task.estimateCheck),
                })}
              </span>
            )}
            <Status task={task} />
          </span>
        </li>
      ))}
    </ul>
  );
}

/** Freie Tätigkeiten (Sessions ohne Aufgabe), jede einzeln; `withTime` zeigt die Dauer. */
export function ActivityList({
  activities,
  withTime,
  empty,
}: {
  activities: readonly ActivityEntry[];
  withTime: boolean;
  empty: string;
}) {
  const t = useT();
  if (activities.length === 0)
    return <p className="text-on-surface-variant">{empty}</p>;
  return (
    <ul className="flex flex-col gap-2">
      {activities.map((entry) => (
        <li
          key={entry.sessionId}
          className="flex flex-wrap items-baseline justify-between gap-x-4"
        >
          <span className="min-w-0 break-words">
            <span className="text-on-surface-variant tabular-nums">
              {shortDate(entry.date)}
            </span>{" "}
            {entry.text || t("work.untitled")}
            {[...entry.projects, ...entry.contexts].length > 0 && (
              <span className="text-on-surface-variant">
                {" "}
                {[...entry.projects, ...entry.contexts].join(" ")}
              </span>
            )}
          </span>
          {withTime && (
            <span className="text-on-surface-variant tabular-nums">
              {formatMinutes(entry.minutes)}
            </span>
          )}
        </li>
      ))}
    </ul>
  );
}
