import { events } from "../lib/api";
import { useNotificationStore } from "./notificationStore";
import { useSettingsStore } from "./settingsStore";
import { useTimerStore } from "./timerStore";
import { useTodoStore } from "./todoStore";

/** Lädt alle Stores neu (Start, `app:reload` nach einem Restore). */
export async function reloadAll(): Promise<void> {
  await Promise.all([
    useSettingsStore.getState().load(),
    useTodoStore.getState().load(),
    useTimerStore.getState().refresh(),
    useNotificationStore.getState().refresh(),
  ]);
}

/**
 * Verbindet die Backend-Ereignisse mit den Stores und lädt den Anfangszustand.
 * Liefert die Funktion zum Abmelden.
 */
export async function connectBackend(): Promise<() => void> {
  const timer = useTimerStore.getState();
  const unlisten = await Promise.all([
    events.timerTick.listen((event) => timer.setSnapshot(event.payload)),
    events.timerComplete.listen((event) => {
      useTimerStore
        .getState()
        .setFinish({ kind: "completed", payload: event.payload });
      // der Zähler eines Todos hat sich geändert
      void useTodoStore.getState().load();
    }),
    events.timerStopped.listen((event) => {
      useTimerStore
        .getState()
        .setFinish({ kind: "stopped", payload: event.payload });
    }),
    events.appReload.listen(() => void reloadAll()),
  ]);
  // Rückkehr aus dem Hintergrund: Ereignisse sind dort ausgeblieben, der Stand kommt neu aus dem Backend
  const onVisible = (): void => {
    if (document.visibilityState === "visible") void reloadAll();
  };
  document.addEventListener("visibilitychange", onVisible);
  await reloadAll();
  return () => {
    unlisten.forEach((fn) => fn());
    document.removeEventListener("visibilitychange", onVisible);
  };
}
