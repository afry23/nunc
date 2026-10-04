import { useEffect, useMemo, useState } from "react";
import { BarChart, Heatmap, LineChart, PieChart } from "../components/charts";
import { DownloadIcon } from "../components/icons";
import { Button, Segmented, SectionLabel } from "../components/ui";
import {
  commands,
  errorMessage,
  unwrap,
  type ExportFormat,
  type StatsPeriod,
  type StatsRequest,
  type StatsResponse,
} from "../lib/api";
import { formatMinutes, monthLabel } from "../lib/format";
import {
  PERIOD_NAMES,
  fillDays,
  periodRange,
  periodTotals,
} from "../lib/stats";
import { useUiStore } from "../stores/uiStore";

const PERIODS: { value: StatsPeriod; label: string }[] = [
  { value: "day", label: "Tag" },
  { value: "week", label: "Woche" },
  { value: "month", label: "Monat" },
  { value: "year", label: "Jahr" },
];

type Loaded =
  | { period: StatsPeriod; request: StatsRequest; response: StatsResponse }
  | { period: StatsPeriod; error: string };

export function StatsView() {
  const [period, setPeriod] = useState<StatsPeriod>("week");
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [exportOpen, setExportOpen] = useState(false);
  const notify = useUiStore((s) => s.notify);
  const fail = useUiStore((s) => s.fail);

  useEffect(() => {
    let cancelled = false;
    const request: StatsRequest = {
      period,
      ...periodRange(period, new Date()),
    };
    void (async () => {
      try {
        const response = await unwrap(commands.getStats(request));
        if (!cancelled) setLoaded({ period, request, response });
      } catch (error) {
        if (!cancelled) setLoaded({ period, error: errorMessage(error) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [period]);

  const current = loaded?.period === period ? loaded : null;
  const data = current && "response" in current ? current : null;
  const days = useMemo(
    () =>
      data
        ? fillDays(
            data.response.daily,
            data.request.startDate,
            data.request.endDate,
          )
        : [],
    [data],
  );

  const exportAs = async (format: ExportFormat): Promise<void> => {
    if (!data) return;
    setExportOpen(false);
    try {
      const path = await unwrap(
        commands.saveExportToFile(data.request, format),
      );
      if (path) notify(`Export gespeichert: ${path}`);
    } catch (error) {
      fail(error);
    }
  };

  const response = data?.response;
  const totals = periodTotals(days);
  const empty =
    response !== undefined &&
    days.every((d) => d.workMinutes === 0 && d.pomodoroCount === 0);

  return (
    <section
      aria-labelledby="stats-heading"
      className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <SectionLabel>Leistung</SectionLabel>
          <h1 id="stats-heading" className="headline-md">
            Statistik
          </h1>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <Segmented
            label="Zeitraum"
            value={period}
            options={PERIODS}
            onChange={setPeriod}
          />
          <div className="relative">
            <Button
              aria-haspopup="true"
              aria-expanded={exportOpen}
              disabled={!data}
              onClick={() => setExportOpen(!exportOpen)}
            >
              <DownloadIcon /> Exportieren
            </Button>
            {exportOpen && (
              <div
                role="group"
                aria-label="Exportformat"
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
        </div>
      </header>

      {current && "error" in current && (
        <p
          role="alert"
          className="rounded-xl bg-surface-container-low p-4 text-red-300"
        >
          {current.error}
        </p>
      )}
      {!current && <p className="text-on-surface-variant">Lade Statistik …</p>}

      {response && empty && (
        <p className="rounded-xl bg-surface-container-low p-8 text-center text-on-surface-variant">
          Keine Sitzungen im gewählten Zeitraum.
        </p>
      )}

      {response && !empty && (
        <>
          <dl className="grid grid-cols-2 gap-4 lg:grid-cols-3">
            <Kpi
              label="Pomodoros"
              value={String(totals.pomodoros)}
              hint={PERIOD_NAMES[period]}
            />
            <Kpi
              label="Fokuszeit"
              value={formatMinutes(totals.workMinutes)}
              hint={PERIOD_NAMES[period]}
            />
            <Kpi
              label="Ø Pomodoros / Tag"
              value={totals.perDay.toFixed(1)}
              hint={PERIOD_NAMES[period]}
            />
            <Kpi
              label="Aktuelle Serie"
              value={`${response.currentStreak} ${response.currentStreak === 1 ? "Tag" : "Tage"}`}
            />
            <Kpi
              label="Längste Serie"
              value={`${response.longestStreak} ${response.longestStreak === 1 ? "Tag" : "Tage"}`}
            />
            <Kpi
              label="Gesamt"
              value={`${response.totalPomodoros} Pomodoros`}
              hint={`${formatMinutes(response.totalWorkMin)} · alle Zeit`}
            />
          </dl>

          <Card title="Arbeitszeit pro Tag">
            <BarChart days={days} />
          </Card>

          {days.length > 1 && (
            <>
              <Card title="Pomodoros pro Tag">
                <LineChart days={days} />
              </Card>
              <Card title="Kalender">
                <Heatmap days={days} />
              </Card>
            </>
          )}

          {(period === "month" || period === "year") &&
            response.monthly.length > 0 && (
              <Card title="Pomodoros pro Monat">
                <table
                  aria-label="Pomodoros pro Monat"
                  className="w-full text-left"
                >
                  <thead className="label-md text-on-surface-variant">
                    <tr>
                      <th className="pb-2 font-semibold">Monat</th>
                      <th className="pb-2 text-right font-semibold">
                        Pomodoros
                      </th>
                      <th className="pb-2 text-right font-semibold">
                        Fokuszeit
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {response.monthly.map((m) => (
                      <tr key={`${m.year}-${m.month}`}>
                        <th scope="row" className="py-1 font-normal">
                          {monthLabel(m.year, m.month)}
                        </th>
                        <td className="py-1 text-right tabular-nums">
                          {m.pomodoroCount}
                        </td>
                        <td className="py-1 text-right text-on-surface-variant tabular-nums">
                          {formatMinutes(m.workMinutes)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </Card>
            )}

          <div className="grid gap-4 lg:grid-cols-2">
            <Card title="Projekte">
              {response.projects.length === 0 ? (
                <p className="text-on-surface-variant">
                  Keine Projekt-Tags in diesem Zeitraum.
                </p>
              ) : (
                <PieChart
                  label="Arbeitszeit nach Projekt"
                  entries={response.projects.map((p) => ({
                    label: p.project,
                    value: p.workMinutes,
                  }))}
                />
              )}
            </Card>
            <Card title="Kontexte">
              {response.contexts.length === 0 ? (
                <p className="text-on-surface-variant">
                  Keine Kontext-Tags in diesem Zeitraum.
                </p>
              ) : (
                <ul className="flex flex-col gap-2">
                  {response.contexts.map((c) => (
                    <li key={c.context} className="flex justify-between">
                      <span className="text-secondary-fixed-dim">
                        {c.context}
                      </span>
                      <span className="text-on-surface-variant">
                        {formatMinutes(c.workMinutes)}
                      </span>
                    </li>
                  ))}
                </ul>
              )}
            </Card>
          </div>

          <Card title="Top-Aufgaben">
            {response.topTodos.length === 0 ? (
              <p className="text-on-surface-variant">
                Noch keine Aufgaben mit Sitzungen.
              </p>
            ) : (
              <ol className="flex flex-col gap-3">
                {response.topTodos.map((todo) => (
                  <li key={todo.todoId} className="flex justify-between gap-4">
                    <span>{todo.description}</span>
                    <span className="shrink-0 text-on-surface-variant">
                      🍅 {todo.pomodoroCount} ·{" "}
                      {formatMinutes(todo.workMinutes)}
                    </span>
                  </li>
                ))}
              </ol>
            )}
          </Card>
        </>
      )}
    </section>
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
    <div className="rounded-2xl bg-surface-container-low p-5">
      <dt className="label-md text-on-surface-variant">{label}</dt>
      <dd className="font-display mt-2 text-3xl font-bold">{value}</dd>
      {hint && <p className="mt-1 text-sm text-on-surface-variant">{hint}</p>}
    </div>
  );
}

function Card({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-2xl bg-surface-container-low p-5">
      <h2 className="mb-4 font-display text-lg font-bold">{title}</h2>
      {children}
    </section>
  );
}
