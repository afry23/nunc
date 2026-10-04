import { create } from "zustand";
import {
  commands,
  unwrap,
  type ConnectionTestResult,
  type WebDavConfig,
  type WebDavSettings,
} from "../lib/api";
import { useUiStore } from "./uiStore";

interface SyncState {
  webdav: WebDavSettings | null;
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

export const useSyncStore = create<SyncState>((set, get) => ({
  webdav: null,
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
