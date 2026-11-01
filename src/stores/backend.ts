import { commands, events } from "../lib/api";
import { isMobile } from "../lib/platform";
import { changedData, describeSyncResult } from "../lib/sync";
import { useNotificationStore } from "./notificationStore";
import { useSettingsStore } from "./settingsStore";
import { useSyncStore } from "./syncStore";
import { useTimerStore } from "./timerStore";
import { useTodoStore } from "./todoStore";
import { useUiStore } from "./uiStore";

/** Lädt alle Stores neu (Start, `app:reload` nach einem Restore). */
export async function reloadAll(): Promise<void> {
  await Promise.all([
    useSettingsStore.getState().load(),
    useTodoStore.getState().load(),
    useTimerStore.getState().refresh(),
    useNotificationStore.getState().refresh(),
    useSyncStore.getState().loadStatus(),
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
    events.syncStatus.listen((event) => {
      const status = event.payload;
      useSyncStore.getState().setStatus(status);
      // Fehler erscheinen zusätzlich als Hinweis, der nichts blockiert
      if (status.state === "error" && status.errorMessage)
        useUiStore.getState().notify(status.errorMessage, "error");
    }),
    events.syncComplete.listen((event) => {
      const result = event.payload;
      if (!changedData(result)) return;
      // vom Server übernommene Daten sind im Backend, die Ansichten laden sie neu
      void useTodoStore.getState().load();
      const message = describeSyncResult(result);
      if (message) useUiStore.getState().notify(message);
    }),
  ]);
  // Rückkehr aus dem Hintergrund: Ereignisse sind dort ausgeblieben, der Stand kommt neu aus dem Backend
  const onVisible = (): void => {
    if (document.visibilityState === "visible") void reloadAll();
    // Mobil wird die App im Hintergrund eingefroren: vorher noch einmal synchronisieren
    else if (isMobile())
      void commands.syncInBackground().catch(() => undefined);
  };
  document.addEventListener("visibilitychange", onVisible);
  await reloadAll();
  return () => {
    unlisten.forEach((fn) => fn());
    document.removeEventListener("visibilitychange", onVisible);
  };
}
