import { create } from "zustand";
import {
  commands,
  unwrap,
  type ConnectionTestResult,
  type SyncResult,
  type SyncStatus,
  type WebDavConfig,
  type WebDavSettings,
} from "../lib/api";
import { useUiStore } from "./uiStore";

interface SyncState {
  status: SyncStatus;
  webdav: WebDavSettings | null;
  loadStatus: () => Promise<void>;
  setStatus: (status: SyncStatus) => void;
  /** Synchronisiert jetzt (manuell). Das Ergebnis meldet `sync:complete`, Fehler `sync:status`. */
  syncNow: () => Promise<SyncResult | null>;
  loadWebdav: () => Promise<void>;
  /** Speichert die Einstellungen; `password: null` lässt das gespeicherte Passwort unverändert. */
  saveWebdav: (
    config: WebDavConfig,
    password: string | null,
  ) => Promise<boolean>;
  /** Prüft die Verbindung mit den übergebenen Werten; `null` bei einem Fehler des Befehls selbst. */
  testWebdav: (
    config: WebDavConfig,
    password: string | null,
  ) => Promise<ConnectionTestResult | null>;
}

export const IDLE_STATUS: SyncStatus = {
  state: "idle",
  lastSyncAt: null,
  errorMessage: null,
};

export const useSyncStore = create<SyncState>((set, get) => ({
  status: IDLE_STATUS,
  webdav: null,
  loadStatus: async () => {
    try {
      set({ status: await commands.getSyncStatus() });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  setStatus: (status) => set({ status }),
  syncNow: async () => {
    try {
      return await unwrap(commands.syncNow());
    } catch {
      // der Fehler kommt als `sync:status` und steht in der Statusleiste
      return null;
    }
  },
  loadWebdav: async () => {
    try {
      set({ webdav: await unwrap(commands.getWebdavConfig()) });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  saveWebdav: async (config, password) => {
    try {
      await unwrap(commands.saveWebdavConfig(config, password));
      await get().loadWebdav();
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  },
  testWebdav: async (config, password) => {
    try {
      return await unwrap(commands.testWebdavConnection(config, password));
    } catch (error) {
      useUiStore.getState().fail(error);
      return null;
    }
  },
}));
