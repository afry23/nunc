import { t } from "../i18n";
import type { SessionType, TimerMode, TimerSnapshot } from "../bindings";

/** CSS-Farbe (Design-Token) je Modus und Sessiontyp. */
export function sessionColor(
  mode: TimerMode,
  sessionType: SessionType,
): string {
  if (sessionType === "short_break") return "var(--color-shortbrk)";
  if (sessionType === "long_break") return "var(--color-longbrk)";
  switch (mode) {
    case "flowtime":
      return "var(--color-flowtime)";
    case "stopwatch":
      return "var(--color-stopwatch)";
    case "pomodoro":
      return "var(--color-pomodoro)";
  }
}

/** Anzeigename der Session. */
export function sessionLabel(
  mode: TimerMode,
  sessionType: SessionType,
): string {
  if (sessionType === "short_break") return t("session.shortBreak");
  if (sessionType === "long_break") return t("session.longBreak");
  switch (mode) {
    case "flowtime":
      return t("session.flowtime");
    case "stopwatch":
      return t("session.stopwatch");
    case "pomodoro":
      return t("session.focus");
  }
}

/** Fortschritt des Rings 0–1. Countdown: verstrichener Anteil; Stoppuhr: aktuelle Stunde. */
export function ringProgress(snapshot: TimerSnapshot): number {
  if (snapshot.state === "idle") return 0;
  if (snapshot.total > 0) {
    const done = (snapshot.total - snapshot.remaining) / snapshot.total;
    return Math.min(1, Math.max(0, done));
  }
  return (snapshot.remaining % 3600) / 3600;
}

/** `stroke-dasharray` eines Kreises für den Fortschritt: sichtbarer Bogen, Rest unsichtbar. */
export function ringDash(radius: number, progress: number): string {
  const circumference = 2 * Math.PI * radius;
  const visible = circumference * Math.min(1, Math.max(0, progress));
  return `${visible.toFixed(2)} ${(circumference - visible).toFixed(2)}`;
}

/** Ansage für Screenreader; ändert sich nur pro Minute, damit nicht jede Sekunde vorgelesen wird. */
export function spokenRemaining(snapshot: TimerSnapshot): string {
  if (snapshot.state === "idle") return t("timer.spoken.idle");
  const count = Math.floor(snapshot.remaining / 60);
  const what = sessionLabel(snapshot.mode, snapshot.sessionType);
  const paused = snapshot.state === "paused" ? t("timer.spoken.paused") : "";
  return t(
    snapshot.total === 0 ? "timer.spoken.elapsed" : "timer.spoken.remaining",
    { what, count, paused },
  );
}

/** Ringdurchmesser des Mini-Timers: passt in das Fenster (Platz für Tasten darunter), 90–200 px. */
export function miniRingSize(width: number, height: number): number {
  return Math.round(
    Math.min(200, Math.max(90, Math.min(width - 24, height - 110))),
  );
}
