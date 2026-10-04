import { create } from "zustand";
import {
  loadNotificationInfo,
  requestNotificationPermission,
  type NotificationInfo,
} from "../lib/notifications";
import { t } from "../i18n";
import { isMobile } from "../lib/platform";
import { useSettingsStore } from "./settingsStore";
import { useUiStore } from "./uiStore";

interface NotificationState {
  /** `null` auf dem Desktop und vor der ersten Abfrage. */
  info: NotificationInfo | null;
  /** Liest den Stand neu (Start, Rückkehr aus dem Hintergrund); nur mobil. */
  refresh: () => Promise<void>;
  /** Fragt die Berechtigung an, falls noch offen. Fehler brechen nie den Timer-Start ab. */
  request: () => Promise<void>;
  /**
   * Vor dem ersten Timer-Start: fragt die Berechtigung an (ADR 0004) und weist einmal darauf hin, wenn sie
   * fehlt. Der Timer läuft in jedem Fall; ohne Erlaubnis meldet er sein Ende nur bei geöffneter App.
   */
  ensureForTimer: () => Promise<void>;
}

let hinted = false;

export const useNotificationStore = create<NotificationState>((set, get) => ({
  info: null,
  refresh: async () => {
    if (!isMobile()) return;
    try {
      set({ info: await loadNotificationInfo() });
    } catch (error) {
      console.warn("Benachrichtigungsstatus nicht lesbar", error);
    }
  },
  request: async () => {
    if (!isMobile()) return;
    try {
      set({ info: await requestNotificationPermission() });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  ensureForTimer: async () => {
    if (
      !isMobile() ||
      !useSettingsStore.getState().settings?.notificationsEnabled
    )
      return;
    if (!get().info) await get().refresh();
    if (get().info?.permission === "prompt") await get().request();
    if (get().info?.permission === "denied" && !hinted) {
      hinted = true;
      useUiStore.getState().notify(t("notifications.blockedHint"));
    }
  },
}));

/** Nur für Tests: der Hinweis erscheint pro App-Start einmal. */
export function resetNotificationHint(): void {
  hinted = false;
}
