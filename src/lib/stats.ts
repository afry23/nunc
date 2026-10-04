import { t } from "../i18n";
import type { DailyStats, StatsPeriod } from "../bindings";
import { addDays, parseIsoDate, toIsoDate } from "./format";

/** Zeitraum eines Auswertungsfensters, das mit `today` endet (lokale Tage, beide Grenzen inklusive). */
export function periodRange(
  period: StatsPeriod,
  today: Date,
): { startDate: string; endDate: string } {
  const length = { day: 1, week: 7, month: 30, year: 365 }[period];
  return {
    startDate: toIsoDate(addDays(today, -(length - 1))),
    endDate: toIsoDate(today),
  };
}

export type DayPoint = {
  date: string;
  pomodoroCount: number;
  workMinutes: number;
  breakMinutes: number;
};

/** Jeder Tag von `start` bis `end` (inklusive); Tage ohne Sessions mit 0. */
export function fillDays(
  daily: DailyStats[],
  start: string,
  end: string,
): DayPoint[] {
  const byDate = new Map(daily.map((d) => [d.date, d]));
  const days: DayPoint[] = [];
  const last = parseIsoDate(end);
  for (let day = parseIsoDate(start); day <= last; day = addDays(day, 1)) {
    const date = toIsoDate(day);
    const found = byDate.get(date);
    days.push({
      date,
      pomodoroCount: found?.pomodoroCount ?? 0,
      workMinutes: found?.workMinutes ?? 0,
      breakMinutes: found?.breakMinutes ?? 0,
    });
  }
  return days;
}

/** Obergrenze einer Achse: nächste „runde“ Zahl (1, 2, 5 × 10ⁿ) ≥ max; mindestens 1. */
export function niceMax(max: number): number {
  if (max <= 1) return 1;
  const magnitude = 10 ** Math.floor(Math.log10(max));
  for (const step of [1, 2, 5, 10]) {
    if (step * magnitude >= max) return step * magnitude;
  }
  return 10 * magnitude;
}

export type Bar = {
  x: number;
  y: number;
  width: number;
  height: number;
  value: number;
};

/** Balken eines Säulendiagramms in einer Fläche `width × height` (Ursprung oben links). */
export function barLayout(
  values: number[],
  width: number,
  height: number,
  gap = 4,
): Bar[] {
  if (values.length === 0) return [];
  const top = niceMax(Math.max(...values));
  const barWidth = Math.max(
    1,
    (width - gap * (values.length - 1)) / values.length,
  );
  return values.map((value, index) => {
    const barHeight = (value / top) * height;
    return {
      x: index * (barWidth + gap),
      y: height - barHeight,
      width: barWidth,
      height: barHeight,
      value,
    };
  });
}

/** Punkte einer Linie (gleichmäßig verteilt) in `width × height`; ein einzelner Wert liegt mittig. */
export function linePoints(
  values: number[],
  width: number,
  height: number,
): { x: number; y: number }[] {
  if (values.length === 0) return [];
  const top = niceMax(Math.max(...values));
  return values.map((value, index) => ({
    x: values.length === 1 ? width / 2 : (index / (values.length - 1)) * width,
    y: height - (value / top) * height,
  }));
}

/** SVG-Pfad (`M…L…`) durch die Punkte. */
export function pathThrough(points: { x: number; y: number }[]): string {
  return points
    .map((p, i) => `${i === 0 ? "M" : "L"}${p.x.toFixed(1)},${p.y.toFixed(1)}`)
    .join(" ");
}

export type HeatCell = {
  date: string;
  count: number;
  level: 0 | 1 | 2 | 3 | 4;
  column: number;
  row: number;
};

/** Heatmap-Kalender: Spalten sind Wochen (Montag zuerst), Zeilen Wochentage (0 = Montag). */
export function heatmapCells(days: DayPoint[]): HeatCell[] {
  if (days.length === 0) return [];
  const max = Math.max(...days.map((d) => d.pomodoroCount));
  const firstRow = (parseIsoDate(days[0].date).getDay() + 6) % 7;
  return days.map((day, index) => {
    const slot = firstRow + index;
    const level =
      day.pomodoroCount === 0
        ? 0
        : Math.min(4, Math.ceil((4 * day.pomodoroCount) / max));
    return {
      date: day.date,
      count: day.pomodoroCount,
      level: level as HeatCell["level"],
      column: Math.floor(slot / 7),
      row: slot % 7,
    };
  });
}

export type Slice = {
  label: string;
  value: number;
  share: number;
  start: number;
  end: number;
};

/** Winkel (Radiant, ab 12 Uhr im Uhrzeigersinn) für Kreisdiagramm-Segmente; Nullwerte entfallen. */
export function pieSlices(
  entries: { label: string; value: number }[],
): Slice[] {
  const positive = entries.filter((e) => e.value > 0);
  const total = positive.reduce((sum, e) => sum + e.value, 0);
  let angle = 0;
  return positive.map((entry) => {
    const share = entry.value / total;
    const slice = {
      label: entry.label,
      value: entry.value,
      share,
      start: angle,
      end: angle + share * 2 * Math.PI,
    };
    angle = slice.end;
    return slice;
  });
}

/** SVG-Pfad eines Kreissegments (Vollkreis wird in zwei Bögen gezeichnet). */
export function slicePath(
  cx: number,
  cy: number,
  r: number,
  start: number,
  end: number,
): string {
  const point = (a: number): string =>
    `${(cx + r * Math.sin(a)).toFixed(2)},${(cy - r * Math.cos(a)).toFixed(2)}`;
  if (end - start >= 2 * Math.PI - 1e-9) {
    return `M${point(0)} A${r},${r} 0 1 1 ${point(Math.PI)} A${r},${r} 0 1 1 ${point(0)} Z`;
  }
  const large = end - start > Math.PI ? 1 : 0;
  return `M${cx},${cy} L${point(start)} A${r},${r} 0 ${large} 1 ${point(end)} Z`;
}

/** Summen über die Tage des Zeitraums (die Gesamtwerte der Antwort zählen dagegen alle Sessions). */
export function periodTotals(days: DayPoint[]): {
  pomodoros: number;
  workMinutes: number;
  perDay: number;
} {
  const pomodoros = days.reduce((sum, d) => sum + d.pomodoroCount, 0);
  const workMinutes = days.reduce((sum, d) => sum + d.workMinutes, 0);
  return {
    pomodoros,
    workMinutes,
    perDay: days.length === 0 ? 0 : pomodoros / days.length,
  };
}

/** Bezeichnung des Auswertungsfensters für die Anzeige. */
export function periodName(period: StatsPeriod): string {
  return t(`stats.period.${period}`);
}
