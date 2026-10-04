import { shortDate, formatMinutes, weekdayLabel } from "../lib/format";
import {
  barLayout,
  heatmapCells,
  linePoints,
  pathThrough,
  pieSlices,
  slicePath,
  type DayPoint,
} from "../lib/stats";

const W = 600;
const H = 180;
const LABEL_H = 24;

/** Beschriftung jedes n-ten Tages: bis 7 Tage der Wochentag, sonst das Datum. */
function dayLabel(days: DayPoint[], index: number): string | null {
  const step = Math.ceil(days.length / 8);
  if (index % step !== 0) return null;
  return days.length <= 7
    ? weekdayLabel(days[index].date)
    : shortDate(days[index].date);
}

/** Säulendiagramm: Arbeitszeit je Tag. */
export function BarChart({ days }: { days: DayPoint[] }) {
  const bars = barLayout(
    days.map((d) => d.workMinutes),
    W,
    H,
    days.length > 31 ? 1 : 6,
  );
  const total = days.reduce((sum, d) => sum + d.workMinutes, 0);
  return (
    <svg
      role="img"
      data-testid="chart-bar"
      aria-label={`Arbeitszeit pro Tag, insgesamt ${formatMinutes(total)}`}
      viewBox={`0 0 ${W} ${H + LABEL_H}`}
      className="w-full"
    >
      {bars.map((bar, index) => (
        <g key={days[index].date}>
          <rect
            x={bar.x}
            y={bar.y}
            width={bar.width}
            height={Math.max(bar.height, 2)}
            rx={3}
            fill="var(--color-primary)"
            opacity={bar.value > 0 ? 1 : 0.2}
          >
            <title>{`${days[index].date}: ${formatMinutes(bar.value)}`}</title>
          </rect>
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
      ))}
    </svg>
  );
}

/** Liniendiagramm: abgeschlossene Pomodoros je Tag. */
export function LineChart({ days }: { days: DayPoint[] }) {
  const points = linePoints(
    days.map((d) => d.pomodoroCount),
    W,
    H,
  );
  const total = days.reduce((sum, d) => sum + d.pomodoroCount, 0);
  return (
    <svg
      role="img"
      data-testid="chart-line"
      aria-label={`Pomodoros pro Tag, insgesamt ${total}`}
      viewBox={`0 0 ${W} ${H + LABEL_H}`}
      className="w-full"
    >
      <path
        d={pathThrough(points)}
        fill="none"
        stroke="var(--color-secondary-fixed-dim)"
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
          fill="var(--color-primary)"
        >
          <title>{`${days[index].date}: ${days[index].pomodoroCount} Pomodoros`}</title>
        </circle>
      ))}
      {days.length <= 14 &&
        points.map((p, index) => (
          <text
            key={`n-${days[index].date}`}
            x={p.x}
            y={Math.max(12, p.y - 10)}
            textAnchor="middle"
            fontSize="12"
            fontWeight="600"
            fill="var(--color-on-surface)"
            data-testid="line-value"
          >
            {days[index].pomodoroCount}
          </text>
        ))}
      {days.map((day, index) =>
        dayLabel(days, index) ? (
          <text
            key={day.date}
            x={points[index].x}
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
        aria-label="Kalender der Pomodoros pro Tag"
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
            <title>{`${cell.date}: ${cell.count} Pomodoros`}</title>
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
