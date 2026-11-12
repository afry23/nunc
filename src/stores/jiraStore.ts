import { create } from "zustand";
import {
  commands,
  unwrap,
  type ConnectionTestResult,
  type JiraConfig,
  type JiraSettings,
} from "../lib/api";
import { useUiStore } from "./uiStore";

interface JiraState {
  settings: JiraSettings | null;
  load: () => Promise<void>;
  /** Speichert den Zugang; `token: null` lässt das gespeicherte Token unverändert. */
  save: (config: JiraConfig, token: string | null) => Promise<boolean>;
  /** Prüft die Verbindung mit den übergebenen Werten; `null` bei einem Fehler des Befehls selbst. */
  test: (
    config: JiraConfig,
    token: string | null,
  ) => Promise<ConnectionTestResult | null>;
  /** Entfernt Einstellungen und Token. */
  clear: () => Promise<boolean>;
}

export const useJiraStore = create<JiraState>((set, get) => ({
  settings: null,
  load: async () => {
    try {
      set({ settings: await unwrap(commands.getJiraConfig()) });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  save: async (config, token) => {
    try {
      await unwrap(commands.saveJiraConfig(config, token));
      await get().load();
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  },
  test: async (config, token) => {
    try {
      return await unwrap(commands.testJiraConnection(config, token));
    } catch (error) {
      useUiStore.getState().fail(error);
      return null;
    }
  },
  clear: async () => {
    try {
      await unwrap(commands.clearJiraConfig());
      await get().load();
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  },
}));
