import { create } from "zustand";
import { t } from "../i18n";
import {
  errorMessage,
  commands,
  unwrap,
  type ConnectionTestResult,
  type DriveStatus,
  type SyncImportInfo,
  type SyncResult,
  type SyncStatus,
  type WebDavConfig,
  type WebDavSettings,
} from "../lib/api";
import { describeSyncResult } from "../lib/sync";
import { useUiStore } from "./uiStore";

/** Ablauf des Imports auf einem neuen Gerät: Angebot → läuft → Zusammenfassung. */
export type ImportPhase = "offer" | "running" | "done";

export interface ImportState {
  info: SyncImportInfo;
  phase: ImportPhase;
  /** Zusammenfassung nach dem Import. */
  summary: string | null;
}

interface SyncState {
  status: SyncStatus;
  webdav: WebDavSettings | null;
  drive: DriveStatus | null;
  /** Das Verbinden läuft (der Browser ist offen). */
  connecting: boolean;
  /** Fehler beim Verbinden, inline angezeigt; `null` = keiner. */
  connectError: string | null;
  importOffer: ImportState | null;
  loadStatus: () => Promise<void>;
  setStatus: (status: SyncStatus) => void;
  /** Synchronisiert jetzt (manuell). Das Ergebnis meldet `sync:complete`, Fehler `sync:status`. */
  syncNow: () => Promise<SyncResult | null>;
  loadDrive: () => Promise<void>;
  /** Meldet bei Google an. Bietet einen Import an, wenn auf Drive Stand liegt und dieses Gerät leer ist. */
  connectDrive: () => Promise<void>;
  disconnectDrive: () => Promise<void>;
  /** Fehler beim Eintragen des eigenen Google-Clients, inline angezeigt; `null` = keiner. */
  clientError: string | null;
  /** Trägt den eigenen Google-Client ein (ADR 0019); `true` bei Erfolg. */
  saveGoogleClient: (
    clientId: string,
    clientSecret: string,
  ) => Promise<boolean>;
  /** Entfernt den eigenen Google-Client (und damit die Verbindung). */
  clearGoogleClient: () => Promise<void>;
  /** Übernimmt den Stand von Drive („Importieren“). */
  runImport: () => Promise<void>;
  /** Schließt das Import-Angebot („Leer starten“ / „Fertig“). */
  closeImport: () => void;
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
  drive: null,
  connecting: false,
  connectError: null,
  clientError: null,
  importOffer: null,
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
  loadDrive: async () => {
    try {
      set({ drive: await unwrap(commands.getDriveStatus()) });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  connectDrive: async () => {
    set({ connecting: true, connectError: null });
    try {
      const drive = await unwrap(commands.connectDrive());
      set({ drive });
      const info = await unwrap(commands.checkSyncImport());
      if (info.available)
        set({ importOffer: { info, phase: "offer", summary: null } });
      else await commands.finishConnect();
      await get().loadDrive();
    } catch (error) {
      set({
        connectError:
          error instanceof Error ? errorMessage(error) : t("common.error"),
      });
    } finally {
      set({ connecting: false });
    }
  },
  disconnectDrive: async () => {
    try {
      set({ drive: await unwrap(commands.disconnectDrive()) });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  saveGoogleClient: async (clientId, clientSecret) => {
    set({ clientError: null });
    try {
      const drive = await unwrap(
        commands.setGoogleClient(clientId, clientSecret),
      );
      set({ drive, connectError: null });
      return true;
    } catch (error) {
      set({ clientError: errorMessage(error) });
      return false;
    }
  },
  clearGoogleClient: async () => {
    try {
      set({
        drive: await unwrap(commands.clearGoogleClient()),
        clientError: null,
      });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  runImport: async () => {
    const offer = get().importOffer;
    if (!offer) return;
    set({ importOffer: { ...offer, phase: "running" } });
    const result = await get().syncNow();
    const summary = result
      ? (describeSyncResult(result) ?? t("sync.import.nothing"))
      : t("sync.import.failed");
    set({ importOffer: { ...offer, phase: "done", summary } });
    await get().loadDrive();
  },
  closeImport: () => set({ importOffer: null }),
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
