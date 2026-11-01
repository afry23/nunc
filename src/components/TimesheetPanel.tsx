import { useCallback, useEffect, useState } from "react";
import { ChevronLeftIcon, ChevronRightIcon, DownloadIcon } from "./icons";
import { SessionDialog, type EditedSession } from "./SessionDialog";
import { Timeline } from "./Timeline";
import { Button, IconButton, Segmented } from "./ui";
import { useT } from "../i18n";
import {
  commands,
  errorMessage,
  unwrap,
  type DayTimeline,
  type GroupBy,
  type TimelineBlock,
  type Timesheet,
} from "../lib/api";
import {
  formatMinutes,
  shortDate,
  toIsoDate,
  weekdayLabel,
} from "../lib/format";
import {
  dayToShow,
  rowKey,
  rowLabel,
  weekLabel,
  weekRange,
} from "../lib/timesheet";
import { useTodoStore } from "../stores/todoStore";
import { useUiStore } from "../stores/uiStore";

const GROUPS: GroupBy[] = ["project", "context", "todo"];

type Loaded =
  { timesheet: Timesheet; timeline: DayTimeline } | { error: string };
type Editing = { session: EditedSession | null } | null;

/**
 * Reiter „Zeiten“: Wochen-Timesheet (Gruppe × Tag) und Timeline des gewählten Tages mit Bearbeiten,
 * Nachtragen und Löschen.
 */
export function TimesheetPanel() {
  const t = useT();
  const todos = useTodoStore((s) => s.all);
  const notify = useUiStore((s) => s.notify);
  const [weeksAgo, setWeeksAgo] = useState(0);
  const [groupBy, setGroupBy] = useState<GroupBy>("project");
  const [today] = useState(() => new Date());
  const range = weekRange(today, weeksAgo);
  const [chosen, setChosen] = useState<string | null>(null);
  const day = chosen ?? dayToShow(range, today);
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [editing, setEditing] = useState<Editing>(null);
  const [reload, setReload] = useState(0);
  const { startDate, endDate } = range;

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const [timesheet, timeline] = await Promise.all([
          unwrap(commands.getTimesheet(startDate, endDate, groupBy)),
          unwrap(commands.getDayTimeline(day)),
        ]);
        if (!cancelled) setLoaded({ timesheet, timeline });
      } catch (error) {
        if (!cancelled) setLoaded({ error: errorMessage(error) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [startDate, endDate, groupBy, day, reload]);

  const moveWeek = (delta: number): void => {
    setWeeksAgo(weeksAgo + delta);
    setChosen(null);
  };

  const goToToday = (): void => {
    setWeeksAgo(0);
    setChosen(null);
  };

  const exportCsv = async (): Promise<void> => {
    try {
      const path = await unwrap(
        commands.saveExportToFile(
          { period: "week", startDate, endDate },
          "csv",
        ),
      );
      if (path) notify(t("stats.exportSaved", { path }));
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };

  const select = useCallback(
    (block: TimelineBlock) =>
      setEditing({
        session: {
          id: block.sessionId,
          start: block.start,
          end: block.end,
          todoId: block.todoId,
          note: block.note,
          projectTags: block.projectTags,
          contextTags: block.contextTags,
        },
      }),
    [],
  );

  const data = loaded && "timesheet" in loaded ? loaded : null;
  const timesheet = data?.timesheet;

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-1">
          <IconButton
            label={t("timesheet.prevWeek")}
            onClick={() => moveWeek(1)}
          >
            <ChevronLeftIcon />
          </IconButton>
          <p className="font-display min-w-36 text-center text-lg font-bold">
            {weekLabel(startDate, endDate)}
          </p>
          <IconButton
            label={t("timesheet.nextWeek")}
            onClick={() => moveWeek(-1)}
          >
            <ChevronRightIcon />
          </IconButton>
          <Button
            variant="ghost"
            disabled={weeksAgo === 0 && chosen === null}
            onClick={goToToday}
          >
            {t("timesheet.today")}
          </Button>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <Segmented
            label={t("timesheet.groupBy")}
            value={groupBy}
            options={GROUPS.map((value) => ({
              value,
              label: t(`timesheet.group.${value}`),
            }))}
            onChange={setGroupBy}
          />
          <Button disabled={!data} onClick={() => void exportCsv()}>
            <DownloadIcon /> {t("timesheet.export")}
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
        <p className="text-on-surface-variant">{t("timesheet.loading")}</p>
      )}

      {timesheet && (
        <section className="rounded-2xl bg-surface-container-low p-5">
          {timesheet.rows.length === 0 && (
            <p className="mb-4 text-on-surface-variant">
              {t("timesheet.none")}
            </p>
          )}
          <div className="overflow-x-auto">
            <table
              aria-label={t("timesheet.table")}
              className="w-full text-left"
            >
              <thead className="label-md text-on-surface-variant">
                <tr>
                  <th className="pb-2 font-semibold">
                    {t("timesheet.col.group")}
                  </th>
                  {timesheet.days.map((date) => (
                    <th key={date} className="pb-2 text-right font-semibold">
                      <button
                        type="button"
                        aria-pressed={date === day}
                        aria-label={`${weekdayLabel(date)} ${shortDate(date)}`}
                        onClick={() => setChosen(date)}
                        className={`min-h-11 rounded-lg px-2 ${
                          date === day
                            ? "bg-surface-container-high text-on-surface"
                            : "hover:text-on-surface"
                        }`}
                      >
                        {weekdayLabel(date)}
                        <br />
                        <span className="font-normal">{shortDate(date)}</span>
                      </button>
                    </th>
                  ))}
                  <th className="pb-2 text-right font-semibold">
                    {t("timesheet.col.total")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {timesheet.rows.map((row) => (
                  <tr key={rowKey(row)}>
                    <th scope="row" className="py-1 pr-3 font-normal">
                      {rowLabel(row)}
                    </th>
                    {row.minutes.map((minutes, index) => (
                      <td
                        key={timesheet.days[index]}
                        className="px-2 py-1 text-right text-on-surface-variant tabular-nums"
                      >
                        {minutes === 0 ? "–" : formatMinutes(minutes)}
                      </td>
                    ))}
                    <td className="px-2 py-1 text-right tabular-nums">
                      {formatMinutes(row.total)}
                    </td>
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr className="font-semibold">
                  <th scope="row" className="pt-2 pr-3 text-left">
                    {t("timesheet.total")}
                  </th>
                  {timesheet.dayTotals.map((minutes, index) => (
                    <td
                      key={timesheet.days[index]}
                      className="px-2 pt-2 text-right tabular-nums"
                    >
                      {minutes === 0 ? "–" : formatMinutes(minutes)}
                    </td>
                  ))}
                  <td className="px-2 pt-2 text-right tabular-nums">
                    {formatMinutes(timesheet.total)}
                  </td>
                </tr>
              </tfoot>
            </table>
          </div>
        </section>
      )}

      {data && (
        <section
          aria-labelledby="timeline-heading"
          className="rounded-2xl bg-surface-container-low p-5"
        >
          <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
            <h2
              id="timeline-heading"
              className="font-display text-lg font-bold"
            >
              {t("timeline.title", {
                date: `${weekdayLabel(day)} ${shortDate(day)}`,
              })}
            </h2>
            <Button onClick={() => setEditing({ session: null })}>
              {t("timeline.add")}
            </Button>
          </div>
          {data.timeline.blocks.length === 0 && (
            <p className="mb-4 text-on-surface-variant">
              {t("timeline.empty")}
            </p>
          )}
          <Timeline
            date={day}
            timeline={data.timeline}
            todos={todos}
            onSelect={select}
          />
        </section>
      )}

      {editing && (
        <SessionDialog
          session={editing.session}
          date={day === "" ? toIsoDate(today) : day}
          todos={todos}
          onClose={() => setEditing(null)}
          onSaved={(overlaps) => {
            setEditing(null);
            setReload(reload + 1);
            if (overlaps > 0) notify(t("entry.overlap"));
          }}
        />
      )}
    </div>
  );
}
