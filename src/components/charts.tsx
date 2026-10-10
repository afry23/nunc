import { Fragment } from "react";
import { t } from "../i18n";
import { shortDate, formatMinutes, weekdayLabel } from "../lib/format";
import {
  barLayout,
  heatmapCells,
  linePoints,
  pathThrough,
  pieSlices,
  slicePath,
  stackSegments,
  timerCount,
  timerMinutes,
  TIMERS,
  type DayPoint,
  type TimerKey,
} from "../lib/stats";

const W = 600;
const H = 180;
const LABEL_H = 24;
/** Rand innerhalb der Zeichenfläche: Platz für Zahlen über den Werten und für die Randpunkte der Linie. */
const PAD_X = 18;
const PAD_TOP = 24;
/** Bis zu dieser Zahl Tage stehen die Werte als Zahl im Diagramm. */
const MAX_LABELLED = 14;

/** Beschriftung jedes n-ten Tages: bis 7 Tage der Wochentag, sonst das Datum. */
function dayLabel(days: DayPoint[], index: number): string | null {
  const step = Math.ceil(days.length / 8);
  if (index % step !== 0) return null;
  return days.length <= 7
    ? weekdayLabel(days[index].date)
    : shortDate(days[index].date);
}

/** Beschreibung der Anzahlen je Timer für Tooltips, z. B. „4 Pomodoro, 1 Stoppuhr“. */
function timerDetail(day: DayPoint): string {
  return TIMERS.map((timer) => ({
    count: timerCount(day, timer.key),
    name: t(`timer.mode.${timer.key}`),
  }))
    .filter((entry) => entry.count > 0)
    .map((entry) => t("charts.timerCount", entry))
    .join(", ");
}

/** Legende der Timer-Farben. */
function Legend({ keys }: { keys: readonly TimerKey[] }) {
  return (
    <ul
      data-testid="chart-legend"
      aria-label={t("charts.legend")}
      className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-on-surface-variant"
    >
      {TIMERS.filter((timer) => keys.includes(timer.key)).map((timer) => (
        <li key={timer.key} className="flex items-center gap-1.5">
          <span
            aria-hidden="true"
            className="inline-block h-2.5 w-2.5 rounded-full"
            style={{ background: timer.color }}
          />
          {t(`timer.mode.${timer.key}`)}
        </li>
      ))}
    </ul>
  );
}

/** Timer, die im Zeitraum Werte haben; Pomodoro steht immer dabei. */
function usedTimers(days: DayPoint[]): TimerKey[] {
  return TIMERS.filter(
    (timer) =>
      timer.key === "pomodoro" ||
      days.some((day) => timerCount(day, timer.key) > 0),
  ).map((timer) => timer.key);
}

/** Säulendiagramm: Arbeitszeit je Tag, gestapelt nach Timer; darüber die Anzahl je Timer in dessen Farbe. */
export function BarChart({ days }: { days: DayPoint[] }) {
  const bars = barLayout(
    days.map((d) => d.workMinutes),
    W,
    H - PAD_TOP,
    days.length > 31 ? 1 : 6,
  );
  const total = days.reduce((sum, d) => sum + d.workMinutes, 0);
  const labelled = days.length <= MAX_LABELLED;
  return (
    <>
      <svg
        role="img"
        data-testid="chart-bar"
        aria-label={t("charts.bar", { total: formatMinutes(total) })}
        viewBox={`0 0 ${W} ${H + LABEL_H}`}
        className="w-full"
      >
        {bars.map((bar, index) => {
          const day = days[index];
          const base = { ...bar, y: bar.y + PAD_TOP };
          const segments = stackSegments(
            base,
            TIMERS.map((timer) => timerMinutes(day, timer.key)),
          );
          const counts = TIMERS.map((timer) => ({
            color: timer.color,
            count: timerCount(day, timer.key),
          })).filter((entry) => entry.count > 0);
          return (
            <g key={day.date}>
              <title>
                {t("charts.barTitle", {
                  date: day.date,
                  detail: timerDetail(day) || "0",
                  time: formatMinutes(bar.value),
                })}
              </title>
              {bar.value > 0 ? (
                TIMERS.map((timer, n) =>
                  segments[n].height > 0 ? (
                    <rect
                      key={timer.key}
                      data-testid={`bar-${timer.key}`}
                      x={bar.x}
                      y={segments[n].y}
                      width={bar.width}
                      height={segments[n].height}
                      rx={
                        segments.filter((s) => s.height > 0).length === 1
                          ? 3
                          : 0
                      }
                      fill={timer.color}
                    />
                  ) : null,
                )
              ) : (
                <rect
                  x={bar.x}
                  y={base.y}
                  width={bar.width}
                  height={2}
                  rx={3}
                  fill="var(--color-primary)"
                  opacity={0.2}
                />
              )}
              {labelled && (
                <text
                  data-testid="bar-value"
                  x={bar.x + bar.width / 2}
                  y={base.y - 6}
                  textAnchor="middle"
                  fontSize="13"
                  fontWeight="600"
                  fill="var(--color-on-surface)"
                >
                  {counts.length === 0
                    ? "0"
                    : counts.map((entry, n) => (
                        <Fragment key={entry.color}>
                          {n > 0 && " · "}
                          <tspan fill={entry.color}>{entry.count}</tspan>
                        </Fragment>
                      ))}
                </text>
              )}
              {dayLabel(days, index) && (
                <text
                  x={bar.x + bar.width / 2}
                  y={H + 16}
                  textAnchor="middle"
                  fontSize="11"
                  fill="var(--color-on-surface-variant)"
                >
                  {dayLabel(days, index)}
                </text>
              )}
            </g>
          );
        })}
      </svg>
      <Legend keys={usedTimers(days)} />
    </>
  );
}

/** Liniendiagramm: Anzahl je Tag, eine Linie je Timer in dessen Farbe (gemeinsame Skala). */
export function LineChart({ days }: { days: DayPoint[] }) {
  const keys = usedTimers(days);
  const lines = TIMERS.filter((timer) => keys.includes(timer.key));
  const max = Math.max(
    ...lines.flatMap((timer) => days.map((d) => timerCount(d, timer.key))),
  );
  const drawn = lines.map((timer) => ({
    timer,
    points: linePoints(
      days.map((d) => timerCount(d, timer.key)),
      W - 2 * PAD_X,
      H - PAD_TOP,
      max,
    ).map((p) => ({ x: p.x + PAD_X, y: p.y + PAD_TOP })),
  }));
  const total = days.reduce(
    (sum, d) =>
      sum + TIMERS.reduce((n, timer) => n + timerCount(d, timer.key), 0),
    0,
  );
  const labelled = days.length <= MAX_LABELLED;
  const axis = drawn[0].points;
  return (
    <>
      <svg
        role="img"
        data-testid="chart-line"
        aria-label={t("charts.line", { total })}
        viewBox={`0 0 ${W} ${H + LABEL_H}`}
        className="w-full"
      >
        {drawn.map(({ timer, points }) => (
          <g key={timer.key} data-testid={`line-${timer.key}`}>
            <path
              d={pathThrough(points)}
              fill="none"
              stroke={timer.color}
              strokeWidth="3"
              strokeLinejoin="round"
              strokeLinecap="round"
            />
            {points.map((p, index) => (
              <circle
                key={days[index].date}
                cx={p.x}
                cy={p.y}
                r={days.length > 31 ? 0 : 4}
                fill={timer.color}
              >
                <title>
                  {t("charts.lineTitle", {
                    date: days[index].date,
                    detail: t("charts.timerCount", {
                      count: timerCount(days[index], timer.key),
                      name: t(`timer.mode.${timer.key}`),
                    }),
                  })}
                </title>
              </circle>
            ))}
            {labelled &&
              points.map((p, index) => {
                const count = timerCount(days[index], timer.key);
                // Pomodoro-Linie beschriftet jeden Tag, die übrigen nur Tage mit Werten
                if (count === 0 && timer.key !== "pomodoro") return null;
                return (
                  <text
                    key={`n-${days[index].date}`}
                    data-testid="line-value"
                    x={p.x}
                    y={p.y - 10}
                    textAnchor="middle"
                    fontSize="13"
                    fontWeight="600"
                    fill={timer.color}
                  >
                    {count}
                  </text>
                );
              })}
          </g>
        ))}
        {days.map((day, index) =>
          dayLabel(days, index) ? (
            <text
              key={day.date}
              x={axis[index].x}
              y={H + 16}
              textAnchor="middle"
              fontSize="11"
              fill="var(--color-on-surface-variant)"
            >
              {dayLabel(days, index)}
            </text>
          ) : null,
        )}
      </svg>
      <Legend keys={keys} />
    </>
  );
}

const LEVEL_OPACITY = [0.08, 0.3, 0.5, 0.75, 1];
const CELL = 16;
const GAP = 4;

/** Heatmap-Kalender: Wochen als Spalten, Wochentage als Zeilen, Farbstärke nach Pomodoros. */
export function Heatmap({ days }: { days: DayPoint[] }) {
  const cells = heatmapCells(days);
  const columns =
    cells.length === 0 ? 0 : Math.max(...cells.map((c) => c.column)) + 1;
  const width = columns * (CELL + GAP);
  const height = 7 * (CELL + GAP);
  return (
    <div className="overflow-x-auto">
      <svg
        role="img"
        data-testid="chart-heatmap"
        aria-label={t("charts.heatmap")}
        viewBox={`0 0 ${width} ${height}`}
        width={width}
        height={height}
        className="max-w-none"
      >
        {cells.map((cell) => (
          <rect
            key={cell.date}
            x={cell.column * (CELL + GAP)}
            y={cell.row * (CELL + GAP)}
            width={CELL}
            height={CELL}
            rx={3}
            fill="var(--color-primary)"
            opacity={LEVEL_OPACITY[cell.level]}
            data-level={cell.level}
          >
            <title>
              {t("charts.heatmapTitle", { date: cell.date, count: cell.count })}
            </title>
          </rect>
        ))}
      </svg>
    </div>
  );
}

const PIE_COLORS = [
  "var(--color-primary)",
  "var(--color-secondary-fixed-dim)",
  "var(--color-flowtime)",
  "var(--color-pomodoro)",
  "var(--color-longbrk)",
  "var(--color-shortbrk)",
];

/** Kreisdiagramm mit Legende: Verteilung der Arbeitszeit. */
export function PieChart({
  entries,
  label,
}: {
  entries: { label: string; value: number }[];
  label: string;
}) {
  const slices = pieSlices(entries);
  return (
    <div className="flex flex-wrap items-center gap-6">
      <svg
        role="img"
        data-testid="chart-pie"
        aria-label={label}
        viewBox="0 0 120 120"
        className="size-36"
      >
        {slices.map((slice, index) => (
          <path
            key={slice.label}
            d={slicePath(60, 60, 56, slice.start, slice.end)}
            fill={PIE_COLORS[index % PIE_COLORS.length]}
          >
            <title>{`${slice.label}: ${formatMinutes(slice.value)}`}</title>
          </path>
        ))}
      </svg>
      <ul className="flex flex-col gap-1">
        {slices.map((slice, index) => (
          <li key={slice.label} className="flex items-center gap-2 text-sm">
            <span
              className="size-3 rounded-full"
              style={{ background: PIE_COLORS[index % PIE_COLORS.length] }}
            />
            <span>{slice.label}</span>
            <span className="text-on-surface-variant">
              {formatMinutes(slice.value)} · {Math.round(slice.share * 100)} %
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
