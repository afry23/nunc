import { useEffect, useState, type ReactNode } from "react";
import { ChevronLeftIcon, ChevronRightIcon, DownloadIcon } from "./icons";
import { Button, IconButton, Segmented } from "./ui";
import { useT } from "../i18n";
import {
  commands,
  errorMessage,
  unwrap,
  type GroupMinutes,
  type Review,
} from "../lib/api";
import { formatMinutes, shortDate, weekdayLabel } from "../lib/format";
import {
  REVIEW_KINDS,
  deltaText,
  rangeLabel,
  reviewRange,
  type ReviewKind,
} from "../lib/review";
import { useUiStore } from "../stores/uiStore";

type Loaded = { review: Review } | { error: string };

function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="rounded-2xl bg-surface-container-low p-5">
      <h2 className="font-display mb-3 text-lg font-bold">{title}</h2>
      <ul className="flex flex-col gap-1">{children}</ul>
    </section>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <li className="flex flex-wrap justify-between gap-x-4 tabular-nums">
      <span className="min-w-0 break-words">{label}</span>
      <span className="text-on-surface-variant">{value}</span>
    </li>
  );
}

function GroupList({
  title,
  groups,
}: {
  title: string;
  groups: GroupMinutes[];
}) {
  const t = useT();
  if (groups.length === 0) return null;
  return (
    <section className="rounded-2xl bg-surface-container-low p-5">
      <h2 className="font-display mb-3 text-lg font-bold">{title}</h2>
      <ul className="flex flex-col gap-1">
        {groups.map((entry) => (
          <li
            key={entry.group ?? ""}
            className="flex justify-between gap-4 tabular-nums"
          >
            <span>{entry.group ?? t("timesheet.unassigned")}</span>
            <span className="text-on-surface-variant">
              {formatMinutes(entry.minutes)}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}

/** Rückblick (ADR 0025): Fokuszeit mit Vergleich, Sessions, Projekte/Kontexte und erledigte Aufgaben eines Zeitraums. */
export function ReviewPanel() {
  const t = useT();
  const notify = useUiStore((s) => s.notify);
  const [kind, setKind] = useState<ReviewKind>("week");
  const [periodsAgo, setPeriodsAgo] = useState(0);
  const [today] = useState(() => new Date());
  const { from, to } = reviewRange(kind, today, periodsAgo);
  const [loaded, setLoaded] = useState<Loaded | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const review = await unwrap(commands.getReview(from, to));
        if (!cancelled) setLoaded({ review });
      } catch (error) {
        if (!cancelled) setLoaded({ error: errorMessage(error) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [from, to]);

  const exportMarkdown = async (): Promise<void> => {
    try {
      const path = await unwrap(commands.saveReviewMarkdown(from, to));
      if (path) notify(t("review.exportSaved", { path }));
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };

  const review = loaded && "review" in loaded ? loaded.review : null;

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-1">
          <IconButton
            label={t("review.prev")}
            onClick={() => setPeriodsAgo(periodsAgo + 1)}
          >
            <ChevronLeftIcon />
          </IconButton>
          <p className="font-display min-w-44 text-center text-lg font-bold">
            {rangeLabel(from, to)}
          </p>
          <IconButton
            label={t("review.next")}
            onClick={() => setPeriodsAgo(periodsAgo - 1)}
          >
            <ChevronRightIcon />
          </IconButton>
          <Button
            variant="ghost"
            disabled={periodsAgo === 0}
            onClick={() => setPeriodsAgo(0)}
          >
            {t("review.today")}
          </Button>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <Segmented
            label={t("review.kind")}
            value={kind}
            options={REVIEW_KINDS.map((value) => ({
              value,
              label: t(`review.kind.${value}`),
            }))}
            onChange={(value) => {
              setKind(value);
              setPeriodsAgo(0);
            }}
          />
          <Button disabled={!review} onClick={() => void exportMarkdown()}>
            <DownloadIcon /> {t("review.export")}
          </Button>
        </div>
      </div>

      {loaded && "error" in loaded && (
        <p
          role="alert"
          className="rounded-xl bg-surface-container-low p-4 text-red-300"
        >
          {loaded.error}
        </p>
      )}
      {!loaded && (
        <p className="text-on-surface-variant">{t("review.loading")}</p>
      )}

      {review && (
        <>
          <section className="rounded-2xl bg-surface-container-low p-5">
            <p className="label-md text-on-surface-variant">
              {t("review.focus")}
            </p>
            <p className="headline-md tabular-nums">
              {formatMinutes(review.focusMin)}
            </p>
            <p className="text-on-surface-variant">
              {t("review.vsPrevious", {
                delta: deltaText(review),
                previous: formatMinutes(review.previousFocusMin),
              })}
            </p>
            <p className="label-md mt-4 text-on-surface-variant">
              {t("review.pomodoros")}
            </p>
            <p className="tabular-nums">{review.pomodoroCount}</p>
            <p className="label-md mt-4 text-on-surface-variant">
              {t("review.sessions")}
            </p>
            <p>
              {t("review.sessionsValue", {
                completed: review.completedSessions,
                aborted: review.abortedSessions,
                rate: review.abortRatePercent,
              })}
            </p>
          </section>
          <Card title={t("review.modes")}>
            {review.modes.map((entry) => (
              <Row
                key={entry.mode}
                label={t(`review.mode.${entry.mode}`)}
                value={t("review.modeValue", {
                  sessions: entry.sessions,
                  time: formatMinutes(entry.minutes),
                })}
              />
            ))}
          </Card>
          {review.tasks.length > 0 && (
            <Card title={t("review.tasks")}>
              {review.tasks.map((task) => (
                <Row
                  key={task.todoId}
                  label={task.text}
                  value={t("review.taskValue", {
                    time: formatMinutes(task.minutes),
                    pomodoros: task.pomodoros,
                    sessions: task.sessions,
                  })}
                />
              ))}
            </Card>
          )}
          {review.days.length > 0 && (
            <Card title={t("review.days")}>
              {review.days.map((day) => (
                <Row
                  key={day.date}
                  label={`${weekdayLabel(day.date)} ${shortDate(day.date)}`}
                  value={t("review.dayValue", {
                    time: formatMinutes(day.minutes),
                    pomodoros: day.pomodoros,
                  })}
                />
              ))}
            </Card>
          )}
          <GroupList title={t("review.projects")} groups={review.projects} />
          <GroupList title={t("review.contexts")} groups={review.contexts} />
          {review.combinations.length > 0 && (
            <Card title={t("review.combinations")}>
              {review.combinations.map((entry) => {
                const tags = [...entry.projects, ...entry.contexts];
                return (
                  <Row
                    key={tags.join(" ")}
                    label={
                      tags.length === 0
                        ? t("timesheet.unassigned")
                        : tags.join(" ")
                    }
                    value={t("review.combinationValue", {
                      time: formatMinutes(entry.minutes),
                      sessions: entry.sessions,
                    })}
                  />
                );
              })}
            </Card>
          )}
          <section className="rounded-2xl bg-surface-container-low p-5">
            <h2 className="font-display mb-3 text-lg font-bold">
              {t("review.completed")}
            </h2>
            {review.completedTodos.length === 0 ? (
              <p className="text-on-surface-variant">
                {t("review.noCompleted")}
              </p>
            ) : (
              <ul className="flex flex-col gap-1">
                {review.completedTodos.map((todo) => (
                  <li
                    key={`${todo.completedAt}|${todo.text}`}
                    className="flex gap-4"
                  >
                    <span className="text-on-surface-variant tabular-nums">
                      {shortDate(todo.completedAt)}
                    </span>
                    <span>{todo.text}</span>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </>
      )}
    </div>
  );
}
