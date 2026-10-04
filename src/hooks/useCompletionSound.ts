import { useEffect } from "react";
import { playSound } from "../lib/sound";
import { useSettingsStore } from "../stores/settingsStore";
import { useTimerStore } from "../stores/timerStore";

/** Spielt am Ende einer Session den eingestellten Signalton (nur im Hauptfenster). */
export function useCompletionSound(): void {
  const seq = useTimerStore((s) => s.finishSeq);

  useEffect(() => {
    const { finish } = useTimerStore.getState();
    const settings = useSettingsStore.getState().settings;
    if (seq === 0 || finish?.kind !== "completed" || !settings?.soundEnabled)
      return;
    const isWork = finish.payload.session.sessionType === "work";
    playSound(
      isWork ? settings.soundEndPomodoro : settings.soundEndBreak,
      settings.soundVolume,
    );
  }, [seq]);
}
