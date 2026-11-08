import { useEffect, useMemo, useState } from "react";
import { BarChart, Heatmap, LineChart, PieChart } from "../components/charts";
import { CollapsibleSection } from "../components/CollapsibleSection";
import { DownloadIcon } from "../components/icons";
import { FilterMenu } from "../components/FilterMenu";
import { PeriodNav } from "../components/PeriodNav";
import { ActivityList, TaskList } from "../components/WorkLists";
import { ActionButton, Button, SectionLabel } from "../components/ui";
import {
  commands,
  errorMessage,
  unwrap,
  type ExportFormat,
  type GroupMinutes,
  type Review,
  type StatsPeriod,
  type StatsRequest,
  type StatsResponse,
} from "../lib/api";
import { useT } from "../i18n";
import { formatMinutes } from "../lib/format";
import {
  bucketDays,
  deltaText,
  filterActivities,
  filterTasks,
  granularityFor,
  hourRange,
  projectOptions,
  reviewRange,
  type ReviewKind,
} from "../lib/review";
import { fillDays, periodTotals, type DayPoint } from "../lib/stats";
import { useUiStore } from "../stores/uiStore";

/** Zeitauflösung der Kennzahlen (Monatstabelle) je Zeitraumart. */
const STATS_PERIOD: Record<ReviewKind, StatsPeriod> = {
  day: "day",
  week: "week",
  month: "month",
  quarter: "month",
  year: "year",
};

type Loaded =
  | { review: Review; stats: StatsResponse; request: StatsRequest }
  | { error: string };

/** Auswertung: Zeit je Aufgabe, freier Tätigkeit, Projekt und Kontext mit Diagrammen für Tag bis Jahr. */
export function StatsView() {
  const t = useT();
  const [kind, setKind] = useState<ReviewKind>("week");
  const [periodsAgo, setPeriodsAgo] = useState(0);
  const [today] = useState(() => new Date());
  const [selected, setSelected] = useState<string[]>([]);
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [exportOpen, setExportOpen] = useState(false);
  const notify = useUiStore((s) => s.notify);
  const fail = useUiStore((s) => s.fail);
  const { from, to } = reviewRange(kind, today, periodsAgo);

  useEffect(() => {
    let cancelled = false;
    const request: StatsRequest = {
      period: STATS_PERIOD[kind],
      startDate: from,
      endDate: to,
    };
    void (async () => {
      try {
        const [review, stats] = await Promise.all([
          unwrap(commands.getReview(from, to)),
          unwrap(commands.getStats(request)),
        ]);
        if (!cancelled) setLoaded({ review, stats, request });
      } catch (error) {
        if (!cancelled) setLoaded({ error: errorMessage(error) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [kind, from, to]);

  const data = loaded && "review" in loaded ? loaded : null;
  const none = t("stats.noProject");
  const buckets = useMemo(
    () =>
      data ? bucketDays(data.review.days, from, to, granularityFor(kind)) : [],
    [data, from, to, kind],
  );
  const detailDays: DayPoint[] = useMemo(
    () =>
      data
        ? fillDays(
            data.stats.daily,
            data.request.startDate,
            data.request.endDate,
          )
        : [],
    [data],
  );
  const options = data
    ? projectOptions(data.review.tasks, data.review.activities, none)
    : [];
  const toggle = (value: string): void =>
    setSelected((now) =>
      now.includes(value) ? now.filter((v) => v !== value) : [...now, value],
    );

  const exportAs = async (format: ExportFormat): Promise<void> => {
    if (!data) return;
    setExportOpen(false);
    try {
      const path = await unwrap(
        commands.saveExportToFile(data.request, format),
      );
      if (path) notify(t("stats.exportSaved", { path }));
    } catch (error) {
      fail(error);
    }
  };

  const review = data?.review;
  const empty = review !== undefined && review.focusMin === 0;
  const totals = periodTotals(detailDays);
  const range = review ? hourRange(review.hours) : null;
  const hourPoints: DayPoint[] =
    review && range
      ? review.hours.slice(range.first, range.last + 1).map((minutes, i) => ({
          date: String(range.first + i),
          pomodoroCount: 0,
          workMinutes: minutes,
          breakMinutes: 0,
          pomodoroMinutes: 0,
          flowtimeCount: 0,
          flowtimeMinutes: 0,
          stopwatchCount: minutes > 0 ? 1 : 0,
          stopwatchMinutes: minutes,
        }))
      : [];

  return (
    <section
      aria-labelledby="stats-heading"
      className="mx-auto flex max-w-5xl flex-col gap-6 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <SectionLabel>{t("stats.eyebrow")}</SectionLabel>
          <h1 id="stats-heading" className="headline-md">
            {t("stats.title")}
          </h1>
        </div>
        <div className="relative">
          <ActionButton
            icon={<DownloadIcon />}
            aria-haspopup="true"
            aria-expanded={exportOpen}
            disabled={!data}
            onClick={() => setExportOpen(!exportOpen)}
          >
            {t("stats.export")}
          </ActionButton>
          {exportOpen && (
            <div
              role="group"
              aria-label={t("stats.exportFormat")}
              className="glass shadow-ambient absolute right-0 z-10 mt-2 flex flex-col gap-1 rounded-xl p-2"
            >
              <Button variant="ghost" onClick={() => void exportAs("csv")}>
                CSV
              </Button>
              <Button variant="ghost" onClick={() => void exportAs("json")}>
                JSON
              </Button>
            </div>
          )}
        </div>
      </header>

      <PeriodNav
        kind={kind}
        periodsAgo={periodsAgo}
        from={from}
        to={to}
        onKind={(value) => {
          setKind(value);
          setPeriodsAgo(0);
          setSelected([]);
        }}
        onPeriodsAgo={(value) => {
          setPeriodsAgo(value);
          setSelected([]);
        }}
      />

      {loaded && "error" in loaded && (
        <p
          role="alert"
          className="rounded-xl bg-surface-container-low p-4 text-red-300"
        >
          {loaded.error}
        </p>
      )}
      {!loaded && (
        <p className="text-on-surface-variant">{t("stats.loading")}</p>
      )}

      {review && empty && (
        <p className="rounded-xl bg-surface-container-low p-8 text-center text-on-surface-variant">
          {t("stats.none")}
        </p>
      )}

      {review && !empty && (
        <>
          <section className="rounded-2xl bg-surface-container-low p-5">
            <p className="label-md text-on-surface-variant">
              {t("stats.total")}
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
          </section>

          <Card title={t("stats.card.workPerDay")}>
            <BarChart days={buckets} />
          </Card>

          <div className="grid gap-4 lg:grid-cols-2">
            <Card title={t("stats.card.projects")}>
              <GroupBreakdown
                groups={review.projects}
                none={none}
                label={t("stats.pie")}
                emptyText={t("stats.noProjects")}
              />
            </Card>
            <Card title={t("stats.card.contexts")}>
              <GroupBreakdown
                groups={review.contexts}
                none={none}
                label={t("stats.pie")}
                emptyText={t("stats.noContexts")}
              />
            </Card>
          </div>

          {range && (
            <Card
              title={t("stats.card.hours")}
              caption={t("stats.card.hoursCaption")}
            >
              <HourBars
                points={hourPoints}
                first={range.first}
                label={t("stats.hoursChart")}
              />
            </Card>
          )}

          <Card title={t("stats.card.work")}>
            {options.length > 0 && (
              <div className="mb-3 flex">
                <FilterMenu
                  label={t("stats.filter.project")}
                  options={options}
                  selected={selected}
                  onToggle={toggle}
                />
              </div>
            )}
            <TaskList
              tasks={filterTasks(review.tasks, selected, none)}
              withTime
              empty={t("stats.noTasks")}
            />
          </Card>

          <Card title={t("stats.card.activities")}>
            <ActivityList
              activities={filterActivities(review.activities, selected, none)}
              withTime
              empty={t("stats.noActivities")}
            />
          </Card>

          <CollapsibleSection title={t("stats.card.more")} foldable>
            <div className="flex flex-col gap-6">
              <dl className="grid grid-cols-2 gap-4 lg:grid-cols-3">
                <Kpi
                  label={t("stats.kpi.pomodoros")}
                  value={String(totals.pomodoros)}
                />
                <Kpi
                  label={t("stats.kpi.perDay")}
                  value={totals.perDay.toFixed(1)}
                />
                <Kpi
                  label={t("review.sessions")}
                  value={t("review.sessionsValue", {
                    completed: review.completedSessions,
                    aborted: review.abortedSessions,
                    rate: review.abortRatePercent,
                  })}
                />
                <Kpi
                  label={t("stats.kpi.currentStreak")}
                  value={t("stats.days", {
                    count: data.stats.currentStreak,
                  })}
                />
                <Kpi
                  label={t("stats.kpi.longestStreak")}
                  value={t("stats.days", {
                    count: data.stats.longestStreak,
                  })}
                />
                <Kpi
                  label={t("stats.kpi.total")}
                  value={t("stats.pomodoros", {
                    count: data.stats.totalPomodoros,
                  })}
                  hint={t("stats.allTime", {
                    time: formatMinutes(data.stats.totalWorkMin),
                  })}
                />
              </dl>
              {detailDays.length > 1 && (
                <>
                  <LineChart days={detailDays} />
                  <Heatmap days={detailDays} />
                </>
              )}
            </div>
          </CollapsibleSection>
        </>
      )}
    </section>
  );
}

/** Kreisdiagramm mit Liste der Minuten je Projekt bzw. Kontext. */
function GroupBreakdown({
  groups,
  none,
  label,
  emptyText,
}: {
  groups: GroupMinutes[];
  none: string;
  label: string;
  emptyText: string;
}) {
  const named = groups.filter((g) => g.group !== null);
  if (named.length === 0)
    return <p className="text-on-surface-variant">{emptyText}</p>;
  return (
    <div className="flex flex-col gap-4">
      <PieChart
        label={label}
        entries={groups.map((g) => ({
          label: g.group ?? none,
          value: g.minutes,
        }))}
      />
      <ul className="flex flex-col gap-1">
        {groups.map((g) => (
          <li
            key={g.group ?? ""}
            className="flex justify-between gap-4 tabular-nums"
          >
            <span>{g.group ?? none}</span>
            <span className="text-on-surface-variant">
              {formatMinutes(g.minutes)}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

/** Arbeitszeit je Stunde als einfache Balken mit Stundenbeschriftung. */
function HourBars({
  points,
  first,
  label,
}: {
  points: DayPoint[];
  first: number;
  label: string;
}) {
  const max = Math.max(...points.map((p) => p.workMinutes), 1);
  return (
    <ol
      aria-label={label}
      data-testid="chart-hours"
      className="flex h-40 items-end gap-1"
    >
      {points.map((point, index) => (
        <li
          key={point.date}
          className="flex h-full min-w-0 flex-1 flex-col items-center justify-end gap-1"
          title={`${first + index}:00 – ${formatMinutes(point.workMinutes)}`}
        >
          <span
            aria-hidden="true"
            className="w-full rounded-t-md bg-primary"
            style={{
              height: `${(point.workMinutes / max) * 100}%`,
              opacity: point.workMinutes === 0 ? 0.2 : 1,
              minHeight: 2,
            }}
          />
          <span className="text-xs text-on-surface-variant tabular-nums">
            {first + index}
          </span>
          <span className="sr-only">{formatMinutes(point.workMinutes)}</span>
        </li>
      ))}
    </ol>
  );
}

function Kpi({
  label,
  value,
  hint,
}: {
  label: string;
  value: string;
  hint?: string;
}) {
  return (
    <div className="rounded-2xl bg-surface-container-high p-5">
      <dt className="label-md text-on-surface-variant">{label}</dt>
      <dd className="font-display mt-2 text-2xl font-bold">{value}</dd>
      {hint && <p className="mt-1 text-sm text-on-surface-variant">{hint}</p>}
    </div>
  );
}

function Card({
  title,
  caption,
  children,
}: {
  title: string;
  caption?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-2xl bg-surface-container-low p-5">
      <h2 className="font-display text-lg font-bold">{title}</h2>
      {caption && <p className="text-sm text-on-surface-variant">{caption}</p>}
      <div className="mt-4">{children}</div>
    </section>
  );
}
