import type { ReactNode } from "react";
import { useT, type MessageKey } from "../i18n";
import type { HabitItem } from "../lib/api";
import { toIsoDate } from "../lib/format";
import { isAuto, recentCells, streakKey } from "../lib/habits";
import { useHabitStore } from "../stores/habitStore";

/** Kurzbeschreibung von Rhythmus und Ziel („Mo, Mi, Fr · 45 min +work“). */
function useSummary(item: HabitItem): string {
  const t = useT();
  const { schedule, goal } = item;
  const parts: string[] = [];
  if (schedule?.kind === "daily") parts.push(t("habits.schedule.daily"));
  if (schedule?.kind === "days")
    parts.push(
      schedule.days
        .map((day) => t(`weekday.short.${day % 7}` as MessageKey))
        .join(", "),
    );
  if (schedule?.kind === "weekly")
    parts.push(t("habits.schedule.weekly", { count: schedule.perWeek }));
  if (goal?.kind === "minutes")
    parts.push(
      t("habits.goal.minutes", { minutes: goal.minutes, tag: goal.tag }),
    );
  return parts.join(" · ");
}

/** Eine Gewohnheit mit Haken (oder Status bei berechneten), Serie, Woche und den letzten Tagen. `actions` steht rechts. */
export function HabitRow({
  item,
  actions,
  muted = false,
}: {
  item: HabitItem;
  actions?: ReactNode;
  muted?: boolean;
}) {
  const t = useT();
  const today = toIsoDate(new Date());
  const { progress } = item;
  const summary = useSummary(item);
  const cells = recentCells(item.recent, today);
  const doneCount = cells.filter((cell) => cell.done).length;

  return (
    <li
      data-testid="habit"
      className={`flex flex-col gap-2 rounded-xl bg-surface-container-low p-3 ${muted ? "opacity-70" : ""}`}
    >
      <div className="flex min-w-0 items-center gap-2">
        {progress && !isAuto(item) && !item.archived ? (
          <label className="inline-flex size-11 shrink-0 items-center justify-center">
            <input
              type="checkbox"
              checked={progress.doneToday}
              onChange={() =>
                void useHabitStore
                  .getState()
                  .setDone(item.id, today, !progress.doneToday)
              }
              aria-label={t("habits.done", { name: item.name })}
              className="size-6 accent-[var(--color-primary)]"
            />
          </label>
        ) : (
          <span className="inline-block size-11 shrink-0" aria-hidden />
        )}
        <div className="min-w-0 flex-1">
          <p className="break-words font-semibold">{item.name}</p>
          {summary && (
            <p className="text-sm text-on-surface-variant">{summary}</p>
          )}
        </div>
        {actions && <span className="flex shrink-0">{actions}</span>}
      </div>
      {progress ? (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1 pl-13 text-sm text-on-surface-variant">
          {isAuto(item) && (
            <span>
              {t("habits.auto")} ·{" "}
              {progress.doneToday ? t("habits.autoDone") : t("habits.autoOpen")}
            </span>
          )}
          <span>
            {t(streakKey(progress), { count: progress.currentStreak })}
          </span>
          <span>
            {t("habits.week", {
              done: progress.weekDone,
              target: progress.weekTarget,
            })}
          </span>
          <span
            role="img"
            aria-label={t("habits.recent", { count: doneCount })}
            className="ml-auto flex gap-0.5"
          >
            {cells.map((cell) => (
              <span
                key={cell.day}
                className={`size-2 rounded-full ${cell.done ? "bg-primary" : "bg-surface-container-highest"}`}
              />
            ))}
          </span>
        </div>
      ) : (
        <p className="pl-13 text-sm text-on-surface-variant">
          {t("habits.unknown")}
        </p>
      )}
    </li>
  );
}
