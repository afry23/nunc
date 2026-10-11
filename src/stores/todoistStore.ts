import { create } from "zustand";
import { t } from "../i18n";
import {
  errorMessage,
  commands,
  unwrap,
  type SyncResult,
  type SyncStatus,
  type TodoistStatus,
} from "../lib/api";
import { useUiStore } from "./uiStore";

interface TodoistState {
  /** Verbindung; `null`, bis sie geladen ist. */
  account: TodoistStatus | null;
  status: SyncStatus;
  connecting: boolean;
  /** Fehler beim Verbinden, inline angezeigt. */
  connectError: string | null;
  load: () => Promise<void>;
  setStatus: (status: SyncStatus) => void;
  /** Verbindet mit dem persönlichen API-Token (ADR 0020); `true` bei Erfolg. */
  connect: (token: string) => Promise<boolean>;
  disconnect: () => Promise<void>;
  /** Gleicht jetzt ab. Das Ergebnis meldet `todoist:sync_complete`, Fehler `todoist:sync_status`. */
  syncNow: () => Promise<SyncResult | null>;
}

export const IDLE_TODOIST: SyncStatus = {
  state: "idle",
  lastSyncAt: null,
  errorMessage: null,
};

export const useTodoistStore = create<TodoistState>((set) => ({
  account: null,
  status: IDLE_TODOIST,
  connecting: false,
  connectError: null,
  load: async () => {
    try {
      const [account, status] = await Promise.all([
        unwrap(commands.getTodoistStatus()),
        commands.getTodoistSyncStatus(),
      ]);
      set({ account, status });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  setStatus: (status) => set({ status }),
  connect: async (token) => {
    set({ connecting: true, connectError: null });
    try {
      set({ account: await unwrap(commands.connectTodoistToken(token)) });
      return true;
    } catch (error) {
      set({
        connectError:
          error instanceof Error ? errorMessage(error) : t("common.error"),
      });
      return false;
    } finally {
      set({ connecting: false });
    }
  },
  disconnect: async () => {
    try {
      set({
        account: await unwrap(commands.disconnectTodoist()),
        status: IDLE_TODOIST,
      });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  syncNow: async () => {
    try {
      return await unwrap(commands.todoistSyncNow());
    } catch {
      // der Fehler kommt als `todoist:sync_status` und steht in der Statuszeile
      return null;
    }
  },
}));
