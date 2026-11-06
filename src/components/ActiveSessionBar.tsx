import { Link } from "@tanstack/react-router";
import { useT } from "../i18n";
import { formatClock } from "../lib/format";
import { activeTitle } from "../lib/timer";
import { useTimerStore } from "../stores/timerStore";
import { useTodoStore } from "../stores/todoStore";
import { PauseIcon, PlayIcon, SkipIcon, StopIcon } from "./icons";
import { IconButton } from "./ui";

/**
 * Kompakte Leiste für die laufende Arbeit in allen Hauptbereichen (nur Abbild des Timer-Snapshots). Antippen des
 * Titels öffnet den Fokusmodus. Mobil schwebt sie über der Tab-Leiste, auf breiten Fenstern klebt sie unten.
 */
export function ActiveSessionBar({
  floating,
  keyboardOpen,
}: {
  /** Mobil: über der Tab-Leiste fixiert statt am unteren Rand des Inhalts. */
  floating: boolean;
  keyboardOpen: boolean;
}) {
  const t = useT();
  const snapshot = useTimerStore((s) => s.snapshot);
  const todo = useTodoStore((s) =>
    s.all.find((item) => item.id === snapshot.todoId),
  );
  const { pause, resume, stop } = useTimerStore.getState();
  const onBreak = snapshot.sessionType !== "work";

  const position = floating
    ? `fixed inset-x-0 z-20 ${keyboardOpen ? "bottom-0" : "bottom-[calc(3.5rem+env(safe-area-inset-bottom))]"}`
    : "sticky bottom-0 z-20";

  return (
    <div
      data-testid="active-session"
      role="region"
      aria-label={t("active.label")}
      className={`${position} glass flex items-center gap-1 px-3 py-1`}
    >
      <Link
        to="/focus"
        className="flex min-h-11 min-w-0 flex-1 items-center truncate font-semibold"
      >
        <span className="truncate">{activeTitle(snapshot, todo)}</span>
      </Link>
      <span
        role="timer"
        className="font-display shrink-0 px-2 tabular-nums"
        data-state={snapshot.state}
      >
        {formatClock(snapshot.remaining)}
      </span>
      {snapshot.state === "running" && (
        <IconButton label={t("timer.pause")} onClick={() => void pause()}>
          <PauseIcon />
        </IconButton>
      )}
      {snapshot.state === "paused" && (
        <IconButton label={t("timer.resume")} onClick={() => void resume()}>
          <PlayIcon />
        </IconButton>
      )}
      <IconButton
        label={onBreak ? t("timer.skip") : t("timer.stop")}
        onClick={() => void stop()}
      >
        {onBreak ? <SkipIcon /> : <StopIcon />}
      </IconButton>
    </div>
  );
}
