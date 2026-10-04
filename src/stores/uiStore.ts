import { create } from "zustand";
import { errorMessage } from "../lib/api";

export type Notice = { id: number; kind: "error" | "info"; message: string };

interface UiState {
  notice: Notice | null;
  /** Zeigt eine Meldung (ersetzt die vorige). */
  notify: (message: string, kind?: Notice["kind"]) => void;
  /** Zeigt einen Fehler lesbar an. */
  fail: (error: unknown) => void;
  dismiss: () => void;
}

let nextId = 1;

export const useUiStore = create<UiState>((set) => ({
  notice: null,
  notify: (message, kind = "info") =>
    set({ notice: { id: nextId++, kind, message } }),
  fail: (error) =>
    set({
      notice: { id: nextId++, kind: "error", message: errorMessage(error) },
    }),
  dismiss: () => set({ notice: null }),
}));
