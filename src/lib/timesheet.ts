import { t } from "../i18n";
import type { SessionType, TimelineBlock, TimesheetRow } from "../bindings";
import { addDays, parseIsoDate, shortDate, toIsoDate } from "./format";

/** Reine Hilfen für Zeiten-Reiter, Timeline und Eintrags-Dialog. Alle Tage sind lokale Tage (`YYYY-MM-DD`). */

const pad = (n: number): string => String(n).padStart(2, "0");

/** Montag bis Sonntag der Woche, die `weeksAgo` Wochen vor der Woche von `today` liegt (negativ: danach). */
export function weekRange(
  today: Date,
  weeksAgo: number,
): { startDate: string; endDate: string } {
  const sinceMonday = (today.getDay() + 6) % 7;
  const monday = addDays(today, -sinceMonday - 7 * weeksAgo);
  return {
    startDate: toIsoDate(monday),
    endDate: toIsoDate(addDays(monday, 6)),
  };
}

/** `28.09. – 04.10.` */
export function weekLabel(startDate: string, endDate: string): string {
  return t("timesheet.weekLabel", {
    start: shortDate(startDate),
    end: shortDate(endDate),
  });
}

/** Der Tag, den die Timeline beim Öffnen einer Woche zeigt: heute, wenn er darin liegt, sonst der Montag. */
export function dayToShow(
  range: { startDate: string; endDate: string },
  today: Date,
): string {
  const todayIso = toIsoDate(today);
  return todayIso >= range.startDate && todayIso <= range.endDate
    ? todayIso
    : range.startDate;
}

/** Name einer Zeile; Sessions ohne Tag bzw. Todo heißen „ohne Zuordnung“. */
export function rowLabel(row: TimesheetRow): string {
  return row.group ?? t("timesheet.unassigned");
}

/** Stabiler Schlüssel einer Zeile (die Gruppe `null` kommt höchstens einmal vor). */
export function rowKey(row: TimesheetRow): string {
  return `${row.todoId ?? ""}|${row.group ?? ""}`;
}

/** Minuten des lokalen Tages (0–1440) eines Zeitpunkts auf `date`; Zeitpunkte außerhalb werden begrenzt. */
export function minuteOfDay(iso: string, date: string): number {
  const moment = new Date(iso).getTime();
  const dayStart = parseIsoDate(date);
  dayStart.setHours(0, 0, 0, 0);
  const minutes = (moment - dayStart.getTime()) / 60_000;
  return Math.min(1440, Math.max(0, Math.round(minutes)));
}

export type Window = { fromHour: number; toHour: number };

const DEFAULT_WINDOW: Window = { fromHour: 8, toHour: 18 };

/** Sichtbare Stunden der Timeline: alle Blöcke plus je eine Stunde Rand, mindestens vier Stunden. */
export function timelineWindow(
  blocks: Pick<TimelineBlock, "start" | "end">[],
  date: string,
): Window {
  if (blocks.length === 0) return DEFAULT_WINDOW;
  const first = Math.min(...blocks.map((b) => minuteOfDay(b.start, date)));
  const last = Math.max(...blocks.map((b) => minuteOfDay(b.end, date)));
  let fromHour = Math.max(0, Math.floor(first / 60) - 1);
  let toHour = Math.min(24, Math.ceil(last / 60) + 1);
  if (toHour - fromHour < 4) {
    toHour = Math.min(24, fromHour + 4);
    fromHour = Math.max(0, toHour - 4);
  }
  return { fromHour, toHour };
}

/** Lage eines Blocks im Fenster in Prozent der Höhe; sehr kurze Blöcke bleiben antippbar. */
export function blockPlacement(
  block: Pick<TimelineBlock, "start" | "end">,
  date: string,
  window: Window,
): { top: number; height: number } {
  const total = (window.toHour - window.fromHour) * 60;
  const start = minuteOfDay(block.start, date) - window.fromHour * 60;
  const end = minuteOfDay(block.end, date) - window.fromHour * 60;
  const top = Math.min(100, Math.max(0, (start / total) * 100));
  const bottom = Math.min(100, Math.max(0, (end / total) * 100));
  return { top, height: Math.max(bottom - top, 2) };
}

/** Farb-Token eines Blocks nach Typ der Session. */
export function blockColor(type: SessionType): string {
  return {
    work: "bg-pomodoro",
    short_break: "bg-shortbrk",
    long_break: "bg-longbrk",
  }[type];
}

/** Wert für ein `datetime-local`-Feld (lokale Zeit, ohne Zone) aus einem Zeitpunkt (RFC 3339). */
export function toLocalInput(iso: string): string {
  const d = new Date(iso);
  return `${toIsoDate(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** Zeitpunkt (UTC, RFC 3339) aus dem Wert eines `datetime-local`-Feldes; `null`, wenn er leer oder ungültig ist. */
export function fromLocalInput(value: string): string | null {
  const moment = new Date(value);
  return value === "" || Number.isNaN(moment.getTime())
    ? null
    : moment.toISOString();
}

/** Voreinstellung für einen Nachtrag auf `date`: 09:00 bis 10:00 Uhr lokal. */
export function defaultEntry(date: string): { start: string; end: string } {
  return { start: `${date}T09:00`, end: `${date}T10:00` };
}
