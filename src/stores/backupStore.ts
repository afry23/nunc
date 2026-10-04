import { create } from "zustand";
import {
  commands,
  errorMessage,
  unwrap,
  type BackupEntry,
  type BackupInfo,
  type BackupStatus,
} from "../lib/api";
import { useSettingsStore } from "./settingsStore";
import { useUiStore } from "./uiStore";

/** Eine Wiederherstellung wartet auf die Bestätigung. */
export type PendingRestore = {
  path: string;
  /** Anzeigename: Dateiname bzw. Datum des Backups. */
  label: string;
};

interface BackupState {
  status: BackupStatus | null;
  entries: BackupEntry[];
  /** „Jetzt sichern“ läuft. */
  running: boolean;
  /** Ergebnis der letzten manuellen Sicherung. */
  lastResult: BackupInfo | null;
  pending: PendingRestore | null;
  restoring: boolean;
  /** Fehler der letzten Wiederherstellung (bleibt sichtbar, bis eine neue beginnt). */
  restoreError: string | null;
  load: () => Promise<void>;
  backupNow: () => Promise<void>;
  /** Öffnet den Ordner-Dialog (nur Desktop). */
  chooseDir: () => Promise<void>;
  /** Fragt die Bestätigung für ein Backup aus der Liste an. */
  requestRestore: (entry: BackupEntry) => void;
  /** Öffnet den Datei-Dialog und fragt danach die Bestätigung an. */
  requestRestoreFromFile: () => Promise<void>;
  /** „Speichern unter“ für ein Backup (Mobil: Datei-Dialog des Systems). */
  exportBackup: (entry: BackupEntry) => Promise<void>;
  cancelRestore: () => void;
  confirmRestore: () => Promise<void>;
}

export const useBackupStore = create<BackupState>((set, get) => ({
  status: null,
  entries: [],
  running: false,
  lastResult: null,
  pending: null,
  restoring: false,
  restoreError: null,
  load: async () => {
    try {
      const [status, entries] = await Promise.all([
        unwrap(commands.getBackupStatus()),
        unwrap(commands.listBackups()),
      ]);
      set({ status, entries });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  backupNow: async () => {
    if (get().running) return;
    set({ running: true, lastResult: null });
    try {
      set({ lastResult: await unwrap(commands.triggerBackup()) });
    } catch (error) {
      useUiStore.getState().fail(error);
    } finally {
      set({ running: false });
    }
    await get().load();
  },
  chooseDir: async () => {
    try {
      const dir = await unwrap(commands.selectBackupDir());
      if (dir === null) return;
      await useSettingsStore.getState().load();
      await get().load();
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  requestRestore: (entry) =>
    set({
      pending: { path: entry.path, label: entry.filename },
      restoreError: null,
    }),
  requestRestoreFromFile: async () => {
    try {
      const path = await unwrap(commands.selectRestoreFile());
      if (path === null) return;
      const name = path.split(/[\\/]/).pop() ?? path;
      set({ pending: { path, label: name }, restoreError: null });
    } catch (error) {
      set({ restoreError: errorMessage(error) });
    }
  },
  exportBackup: async (entry) => {
    try {
      const target = await unwrap(commands.exportBackup(entry.path));
      if (target !== null) useUiStore.getState().notify("Backup exportiert.");
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  cancelRestore: () => set({ pending: null }),
  confirmRestore: async () => {
    const pending = get().pending;
    if (!pending || get().restoring) return;
    set({ restoring: true });
    try {
      await unwrap(commands.restoreFromBackup(pending.path));
      set({ pending: null });
      useUiStore.getState().notify("Daten wiederhergestellt.");
      await get().load();
    } catch (error) {
      // die Daten sind unverändert; der Dialog schließt, der Fehler bleibt neben der Liste stehen
      set({ pending: null, restoreError: errorMessage(error) });
    } finally {
      set({ restoring: false });
    }
  },
}));
