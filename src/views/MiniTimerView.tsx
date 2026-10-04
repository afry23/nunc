import { RingTimer } from "../components/RingTimer";
import { Button } from "../components/ui";
import { useViewportSize } from "../hooks/useMediaQuery";
import { commands } from "../lib/api";
import { useT } from "../i18n";
import { miniRingSize } from "../lib/timer";
import { useSettingsStore } from "../stores/settingsStore";
import { useTimerStore } from "../stores/timerStore";

/** Kompakter Timer für das zweite Fenster (always-on-top). */
export function MiniTimerView() {
  const t = useT();
  const snapshot = useTimerStore((s) => s.snapshot);
  const { pause, resume, stop } = useTimerStore.getState();
  const settings = useSettingsStore((s) => s.settings);
  const { width, height } = useViewportSize();
  const idleSeconds =
    snapshot.mode === "pomodoro" ? (settings?.pomodoroMin ?? 25) * 60 : 0;

  return (
    <main className="flex min-h-dvh flex-col items-center justify-center gap-3 overflow-hidden bg-surface-container-lowest p-3">
      <h1 className="sr-only">{t("mini.title")}</h1>
      <RingTimer
        snapshot={snapshot}
        idleSeconds={idleSeconds}
        size={miniRingSize(width, height)}
        compact
      />
      <div className="flex gap-2">
        {snapshot.state === "running" && (
          <Button onClick={() => void pause()}>{t("timer.pause")}</Button>
        )}
        {snapshot.state === "paused" && (
          <Button variant="primary" onClick={() => void resume()}>
            {t("timer.resume")}
          </Button>
        )}
        {snapshot.state !== "idle" && (
          <Button onClick={() => void stop()}>{t("timer.stop")}</Button>
        )}
      </div>
      <Button variant="ghost" onClick={() => void commands.showMainWindow()}>
        {t("mini.toApp")}
      </Button>
    </main>
  );
}
