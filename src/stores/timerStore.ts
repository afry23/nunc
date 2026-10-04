import { create } from "zustand";
import {
  commands,
  unwrap,
  type BreakSuggestion,
  type FinishPayload,
  type TimerMode,
  type TimerSnapshot,
} from "../lib/api";
import { useUiStore } from "./uiStore";

export const IDLE: TimerSnapshot = {
  state: "idle",
  mode: "pomodoro",
  sessionType: "work",
  remaining: 0,
  total: 0,
  todoId: null,
};

/** Wie die letzte Session endete: `completed` (Ablauf bzw. Flowtime-Ende) oder `stopped` (vorzeitig). */
export type Finish = { kind: "completed" | "stopped"; payload: FinishPayload };

interface TimerStoreState {
  /** Nur ein Abbild der Backend-Ereignisse; das Frontend besitzt keinen Timer-Zustand. */
  snapshot: TimerSnapshot;
  suggestedBreak: BreakSuggestion | null;
  /** Letztes Sessionende (für Pausenvorschlag und Signalton); `finishSeq` zählt Enden hoch. */
  finish: Finish | null;
  finishSeq: number;
  selectedTodoId: number | null;
  setSnapshot: (snapshot: TimerSnapshot) => void;
  setFinish: (finish: Finish) => void;
  dismissFinish: () => void;
  selectTodo: (id: number | null) => void;
  refresh: () => Promise<void>;
  start: (mode: TimerMode) => Promise<void>;
  startBreak: () => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  stop: () => Promise<void>;
}

export const useTimerStore = create<TimerStoreState>((set, get) => {
  /** Führt einen Timer-Command aus und übernimmt den gemeldeten Stand. */
  const run = async (
    command: () => Promise<
      | { status: "ok"; data: TimerSnapshot }
      | { status: "error"; error: { code: string; message: string } }
    >,
  ): Promise<void> => {
    try {
      set({ snapshot: await unwrap(command()) });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };

  return {
    snapshot: IDLE,
    suggestedBreak: null,
    finish: null,
    finishSeq: 0,
    selectedTodoId: null,
    setSnapshot: (snapshot) => set({ snapshot }),
    setFinish: (finish) =>
      set((state) => ({
        finish,
        finishSeq: state.finishSeq + 1,
        suggestedBreak: finish.payload.suggestedBreak,
      })),
    dismissFinish: () => set({ finish: null }),
    selectTodo: (selectedTodoId) => set({ selectedTodoId }),
    refresh: async () => {
      try {
        const [snapshot, suggestedBreak] = await Promise.all([
          unwrap(commands.getTimerTick()),
          unwrap(commands.getSuggestedBreak()),
        ]);
        set({ snapshot, suggestedBreak });
      } catch (error) {
        useUiStore.getState().fail(error);
      }
    },
    start: async (mode) => {
      set({ finish: null });
      await run(() => commands.startTimer(mode, get().selectedTodoId));
    },
    startBreak: async () => {
      set({ finish: null });
      await run(() => commands.startBreak());
    },
    pause: () => run(() => commands.pauseTimer()),
    resume: () => run(() => commands.resumeTimer()),
    stop: () => run(() => commands.stopTimer()),
  };
});
