import { useT } from "../i18n";
import type { TimerSnapshot } from "../lib/api";
import { formatClock } from "../lib/format";
import {
  ringDash,
  ringProgress,
  sessionColor,
  sessionLabel,
  spokenRemaining,
} from "../lib/timer";

const RADIUS = 120;
const STROKE = 10;

/**
 * Kreisförmiger Timer als reines SVG. Zeigt nur den Snapshot aus den Backend-Ereignissen:
 * Countdown (verbleibende Zeit) oder Stoppuhr (verstrichene Zeit).
 */
export function RingTimer({
  snapshot,
  idleSeconds,
  size = 320,
  compact = false,
}: {
  snapshot: TimerSnapshot;
  /** Anzeige im Ruhezustand (geplante Dauer; 0 bei Flowtime). */
  idleSeconds: number;
  size?: number;
  compact?: boolean;
}) {
  const t = useT();
  const idle = snapshot.state === "idle";
  const color = sessionColor(snapshot.mode, snapshot.sessionType);
  const seconds = idle ? idleSeconds : snapshot.remaining;
  const view = RADIUS + STROKE;
  const label = sessionLabel(snapshot.mode, snapshot.sessionType);

  return (
    <div
      role="timer"
      aria-label={idle ? t("timer.title") : label}
      data-state={snapshot.state}
      data-session={snapshot.sessionType}
      data-mode={snapshot.mode}
      className="relative mx-auto"
      style={{ width: size, height: size }}
    >
      <svg
        viewBox={`0 0 ${view * 2} ${view * 2}`}
        width={size}
        height={size}
        className="-rotate-90"
        aria-hidden="true"
      >
        <circle
          cx={view}
          cy={view}
          r={RADIUS}
          fill="none"
          stroke="var(--color-outline-variant)"
          strokeOpacity="0.2"
          strokeWidth={STROKE}
        />
        <circle
          data-testid="ring-progress"
          cx={view}
          cy={view}
          r={RADIUS}
          fill="none"
          stroke={color}
          strokeWidth={STROKE}
          strokeLinecap="round"
          strokeDasharray={ringDash(RADIUS, ringProgress(snapshot))}
          className={snapshot.state === "running" ? "timer-pulse" : undefined}
        />
      </svg>
      <div className="absolute inset-0 flex flex-col items-center justify-center">
        <span
          data-testid="ring-time"
          className={`font-display font-bold tabular-nums tracking-tight ${compact ? "" : "display-lg text-6xl"}`}
          style={
            compact ? { fontSize: `${Math.round(size * 0.21)}px` } : undefined
          }
        >
          {formatClock(seconds)}
        </span>
        {!compact && !idle && (
          <span className="label-md mt-3 rounded-lg bg-surface-container-high px-3 py-1 text-on-surface-variant">
            {label}
            {snapshot.state === "paused" ? t("timer.pausedSuffix") : ""}
          </span>
        )}
      </div>
      <p className="sr-only" role="status" aria-live="polite">
        {spokenRemaining(snapshot)}
      </p>
    </div>
  );
}
