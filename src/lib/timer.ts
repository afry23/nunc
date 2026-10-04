import type { SessionType, TimerMode, TimerSnapshot } from "../bindings";

/** CSS-Farbe (Design-Token) je Modus und Sessiontyp. */
export function sessionColor(
  mode: TimerMode,
  sessionType: SessionType,
): string {
  if (sessionType === "short_break") return "var(--color-shortbrk)";
  if (sessionType === "long_break") return "var(--color-longbrk)";
  return mode === "flowtime"
    ? "var(--color-flowtime)"
    : "var(--color-pomodoro)";
}

/** Anzeigename der Session. */
export function sessionLabel(
  mode: TimerMode,
  sessionType: SessionType,
): string {
  if (sessionType === "short_break") return "Kurze Pause";
  if (sessionType === "long_break") return "Lange Pause";
  return mode === "flowtime" ? "Flowtime" : "Fokus";
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
  if (snapshot.state === "idle") return "Timer bereit";
  const minutes = Math.floor(snapshot.remaining / 60);
  const unit = minutes === 1 ? "Minute" : "Minuten";
  const what = sessionLabel(snapshot.mode, snapshot.sessionType);
  const paused = snapshot.state === "paused" ? ", pausiert" : "";
  if (snapshot.total === 0)
    return `${what}: ${minutes} ${unit} gelaufen${paused}`;
  return `${what}: noch ${minutes} ${unit}${paused}`;
}
