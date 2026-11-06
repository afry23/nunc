import { Link } from "@tanstack/react-router";
import { useEffect, useState, type FormEvent, type ReactNode } from "react";
import { ChevronLeftIcon, ChevronRightIcon, DownloadIcon } from "./icons";
import { Button, IconButton, Segmented } from "./ui";
import { currentLanguage, useT } from "../i18n";
import {
  commands,
  errorMessage,
  unwrap,
  type GroupMinutes,
  type Review,
} from "../lib/api";
import {
  formatMinutes,
  shortDate,
  toIsoDate,
  weekdayLabel,
} from "../lib/format";
import {
  REVIEW_KINDS,
  deltaText,
  focusOnlyMinutes,
  isWeekReviewTime,
  openTasks,
  rangeLabel,
  reviewRange,
  type ReviewKind,
} from "../lib/review";
import { useTodoStore } from "../stores/todoStore";
import { useUiStore } from "../stores/uiStore";
import { CollapsibleSection } from "./CollapsibleSection";
import {
  DayNotesCard,
  REFLECTION_FIELD_ID,
  ReflectionCard,
  ResultsCard,
} from "./ReviewJournal";

type Loaded = { review: Review } | { error: string };

function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <CollapsibleSection title={title}>
      <ul className="flex flex-col gap-1">{children}</ul>
    </CollapsibleSection>
  );
}

function Row({
  label,
  value,
  children,
}: {
  label: string;
  value: string;
  children?: ReactNode;
}) {
  return (
    <li>
      <div className="flex flex-wrap justify-between gap-x-4 tabular-nums">
        <span className="min-w-0 break-words">{label}</span>
        <span className="text-on-surface-variant">{value}</span>
      </div>
      {children}
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
    <CollapsibleSection title={title}>
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
    </CollapsibleSection>
  );
}

/** Folgeaufgabe für den nächsten Zeitraum: legt eine Aufgabe an (todo.txt-Syntax). */
function NextPeriodCard() {
  const t = useT();
  const [text, setText] = useState("");
  const submit = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    if (text.trim() === "") return;
    if (await useTodoStore.getState().add(text)) {
      setText("");
      useUiStore.getState().notify(t("review.next.created"));
    }
  };
  return (
    <CollapsibleSection title={t("review.nextTitle")} defaultOpen>
      <form
        onSubmit={(event) => void submit(event)}
        className="flex flex-col gap-2"
      >
        <input
          type="text"
          value={text}
          autoComplete="off"
          aria-label={t("review.next.label")}
          placeholder={t("review.next.placeholder")}
          onChange={(event) => setText(event.target.value)}
          className="min-h-11 w-full rounded-lg bg-surface-container-high px-3"
        />
        <div>
          <Button type="submit" disabled={text.trim() === ""}>
            {t("review.next.create")}
          </Button>
        </div>
      </form>
    </CollapsibleSection>
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
  const [version, setVersion] = useState(0);

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
  }, [from, to, version]);

  const exportMarkdown = async (): Promise<void> => {
    try {
      const path = await unwrap(
        commands.saveReviewMarkdown(from, to, currentLanguage()),
      );
      if (path) notify(t("review.exportSaved", { path }));
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };

  const review = loaded && "review" in loaded ? loaded.review : null;
  const todos = useTodoStore((state) => state.all);
  const inProgress = review ? openTasks(review.tasks, todos) : [];
  const reloadReview = (): void => setVersion((value) => value + 1);

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
          {kind === "week" && (
            <Button
              variant="primary"
              disabled={!review}
              onClick={() =>
                document.getElementById(REFLECTION_FIELD_ID)?.focus()
              }
            >
              {t("review.startWeek")}
            </Button>
          )}
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
            <p className="mt-2 text-on-surface-variant">
              {t("review.focusPart", {
                time: formatMinutes(focusOnlyMinutes(review.modes)),
              })}
            </p>
          </section>
          <ResultsCard
            review={review}
            today={toIsoDate(today)}
            onChanged={reloadReview}
          />
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
                  >
                    {entry.notes.length > 0 && (
                      <ul
                        aria-label={t("review.notes")}
                        className="mt-1 mb-2 ml-4 flex flex-col gap-0.5 text-on-surface-variant"
                      >
                        {entry.notes.map((note, index) => (
                          <li key={`${note.date}|${index}`}>
                            <span className="tabular-nums">
                              {shortDate(note.date)}
                            </span>{" "}
                            {note.text}
                          </li>
                        ))}
                      </ul>
                    )}
                  </Row>
                );
              })}
            </Card>
          )}
          <CollapsibleSection title={t("review.completed")}>
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
          </CollapsibleSection>
          {inProgress.length > 0 && (
            <Card title={t("review.inProgress")}>
              {inProgress.map((task) => (
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
          <DayNotesCard
            review={review}
            today={toIsoDate(today)}
            onChanged={reloadReview}
          />
          <ReflectionCard
            review={review}
            isWeek={kind === "week"}
            weekHint={
              kind === "week" && periodsAgo === 0 && isWeekReviewTime(today)
            }
            onChanged={reloadReview}
          />
          <NextPeriodCard />
          <CollapsibleSection title={t("review.workStyle")} foldable>
            <div className="flex flex-col gap-4">
              <div>
                <p className="label-md text-on-surface-variant">
                  {t("review.pomodoros")}
                </p>
                <p className="tabular-nums">{review.pomodoroCount}</p>
              </div>
              <div>
                <p className="label-md text-on-surface-variant">
                  {t("review.sessions")}
                </p>
                <p>
                  {t("review.sessionsValue", {
                    completed: review.completedSessions,
                    aborted: review.abortedSessions,
                    rate: review.abortRatePercent,
                  })}
                </p>
              </div>
              <div>
                <p className="label-md mb-1 text-on-surface-variant">
                  {t("review.modes")}
                </p>
                <ul className="flex flex-col gap-1">
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
                </ul>
              </div>
              {review.days.length > 0 && (
                <div>
                  <p className="label-md mb-1 text-on-surface-variant">
                    {t("review.days")}
                  </p>
                  <ul className="flex flex-col gap-1">
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
                  </ul>
                </div>
              )}
              <div>
                <Link
                  to="/stats"
                  className="inline-flex min-h-11 items-center rounded-lg bg-surface-container-high px-4 font-semibold hover:bg-surface-bright"
                >
                  {t("review.openStats")}
                </Link>
              </div>
            </div>
          </CollapsibleSection>
        </>
      )}
    </div>
  );
}
