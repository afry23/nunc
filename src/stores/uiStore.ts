import { create } from "zustand";
import { errorMessage } from "../lib/api";

export type Notice = { id: number; kind: "error" | "info"; message: string };

const SIDEBAR_KEY = "nunc.sidebarCollapsed";

/** Pro Gerät gemerkte Vorliebe; ohne Speicher (z. B. gesperrt) bleibt es bei der Voreinstellung. */
function readSidebarCollapsed(): boolean {
  try {
    return localStorage.getItem(SIDEBAR_KEY) === "1";
  } catch {
    return false;
  }
}

interface UiState {
  notice: Notice | null;
  /** Seitenleiste auf Icons verkleinert. */
  sidebarCollapsed: boolean;
  /** Liest die gemerkten Vorlieben (beim Start). */
  restorePrefs: () => void;
  toggleSidebar: () => void;
  /** Zeigt eine Meldung (ersetzt die vorige). */
  notify: (message: string, kind?: Notice["kind"]) => void;
  /** Zeigt einen Fehler lesbar an. */
  fail: (error: unknown) => void;
  dismiss: () => void;
}

let nextId = 1;

export const useUiStore = create<UiState>((set) => ({
  notice: null,
  sidebarCollapsed: false,
  restorePrefs: () => set({ sidebarCollapsed: readSidebarCollapsed() }),
  toggleSidebar: () =>
    set((state) => {
      const sidebarCollapsed = !state.sidebarCollapsed;
      try {
        localStorage.setItem(SIDEBAR_KEY, sidebarCollapsed ? "1" : "0");
      } catch {
        // nicht speicherbar: gilt nur für diese Sitzung
      }
      return { sidebarCollapsed };
    }),
  notify: (message, kind = "info") =>
    set({ notice: { id: nextId++, kind, message } }),
  fail: (error) =>
    set({
      notice: { id: nextId++, kind: "error", message: errorMessage(error) },
    }),
  dismiss: () => set({ notice: null }),
}));
