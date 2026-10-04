import { t, type MessageKey } from "../i18n";

/** Reine Formatierungs- und Datumshilfen. Kalendertage sind immer lokale Tage (`YYYY-MM-DD`). */

const pad = (n: number): string => String(n).padStart(2, "0");

/** `MM:SS`, ab einer Stunde `H:MM:SS`. Negative Werte zählen als 0. */
export function formatClock(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
}

/** `45m`, `2h`, `1h 5m`. */
export function formatMinutes(minutes: number): string {
  const total = Math.max(0, Math.round(minutes));
  const h = Math.floor(total / 60);
  const m = total % 60;
  if (h === 0) return `${m}m`;
  if (m === 0) return `${h}h`;
  return `${h}h ${m}m`;
}

/** Lokales Datum als `YYYY-MM-DD` (nicht über `toISOString`, das wäre UTC). */
export function toIsoDate(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** `YYYY-MM-DD` als lokales Datum mittags (sicher gegen Zeitumstellungen). */
export function parseIsoDate(text: string): Date {
  const [y, m, d] = text.split("-").map(Number);
  return new Date(y, m - 1, d, 12);
}

export function addDays(date: Date, days: number): Date {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
}

/** Kurzer Wochentag (`Mo`) eines Datums `YYYY-MM-DD`. */
export function weekdayLabel(isoDate: string): string {
  return t(`weekday.short.${parseIsoDate(isoDate).getDay()}` as MessageKey);
}

/** `04.10.` für ein Datum `YYYY-MM-DD`. */
export function shortDate(isoDate: string): string {
  const d = parseIsoDate(isoDate);
  return t("format.shortDate", {
    day: pad(d.getDate()),
    month: pad(d.getMonth() + 1),
  });
}

/** Größe in Bytes als `12 KB` / `3,4 MB`. */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${Math.round(kb)} KB`;
  return `${(kb / 1024).toFixed(1).replace(".", t("format.decimal"))} MB`;
}

/** Zeitpunkt (RFC 3339) in lokaler Zeit als `04.10.2026, 08:30`. */
export function formatDateTime(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  const date = t("format.date", {
    day: pad(d.getDate()),
    month: pad(d.getMonth() + 1),
    year: d.getFullYear(),
  });
  return t("format.dateTime", {
    date,
    time: `${pad(d.getHours())}:${pad(d.getMinutes())}`,
  });
}

/** Uhrzeit eines Zeitpunkts (RFC 3339) in lokaler Zeit als `08:30`. */
export function formatTimeOfDay(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** Kalendertag eines Zeitpunkts in UTC als `04.10.2026` (Backup-Dateien tragen ihren Tag im Namen, 12:00 UTC). */
export function formatUtcDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return t("format.date", {
    day: pad(d.getUTCDate()),
    month: pad(d.getUTCMonth() + 1),
    year: d.getUTCFullYear(),
  });
}

/** Flowtime-Verhältnis als Badge: `1 : 0.2`. */
export function ratioBadge(ratio: number): string {
  return `1 : ${Number(ratio.toFixed(2))}`;
}

/** Ankerbezeichnung des Flowtime-Verhältnisses (Schieberegler 0.1 – 0.5). */
export function ratioAnchor(ratio: number): "STRICT" | "BALANCED" | "RELAXED" {
  if (ratio <= 0.2) return "STRICT";
  if (ratio <= 0.35) return "BALANCED";
  return "RELAXED";
}

/** `Oktober 2026` für Jahr und Monat (1–12). */
export function monthLabel(year: number, month: number): string {
  const name =
    month >= 1 && month <= 12
      ? t(`month.${month}` as MessageKey)
      : String(month);
  return `${name} ${year}`;
}
