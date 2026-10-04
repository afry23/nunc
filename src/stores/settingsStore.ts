import { create } from "zustand";
import { applySettingsLanguage } from "../i18n/apply";
import {
  commands,
  errorMessage,
  unwrap,
  type Settings,
  type SyncProvider,
} from "../lib/api";
import { useUiStore } from "./uiStore";

interface SettingsState {
  settings: Settings | null;
  load: () => Promise<void>;
  /** Speichert alle Einstellungen. `false` bei einem Fehler (er wird angezeigt und ist in `lastError`). */
  save: (settings: Settings) => Promise<boolean>;
  /** Wählt den Sync-Anbieter und speichert die Wahl sofort. `false` bei einem Fehler (er wird angezeigt). */
  setSyncProvider: (provider: SyncProvider) => Promise<boolean>;
  /** Ändert einzelne Felder und speichert sofort. */
  patch: (change: Partial<Settings>) => Promise<boolean>;
  lastError: string | null;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  lastError: null,
  load: async () => {
    try {
      const settings = await unwrap(commands.getSettings());
      applySettingsLanguage(settings);
      set({ settings });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  save: async (settings) => {
    try {
      await unwrap(commands.saveSettings(settings));
      // das Backend behält die ihm gehörenden Felder: Ergebnis neu lesen
      const loaded = await unwrap(commands.getSettings());
      applySettingsLanguage(loaded);
      set({ settings: loaded, lastError: null });
      return true;
    } catch (error) {
      set({ lastError: errorMessage(error) });
      useUiStore.getState().fail(error);
      return false;
    }
  },
  setSyncProvider: async (provider) => {
    try {
      await unwrap(commands.setSyncProvider(provider));
      const settings = await unwrap(commands.getSettings());
      applySettingsLanguage(settings);
      set({ settings });
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  },
  patch: async (change) => {
    const current = get().settings;
    if (!current) return false;
    return get().save({ ...current, ...change });
  },
}));
