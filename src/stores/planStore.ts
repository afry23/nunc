import { create } from "zustand";
import { commands, unwrap, type DayPlan } from "../lib/api";
import { useTodoStore } from "./todoStore";
import { useUiStore } from "./uiStore";

interface PlanStoreState {
  /** Plan für heute; `null`, bis er geladen ist. */
  plan: DayPlan | null;
  load: () => Promise<void>;
  /** Plant das Todo für `day` ein; `null` nimmt es vom Plan. */
  planTodo: (id: number, day: string | null) => Promise<void>;
  /** Setzt die Schätzung in Minuten; `null` entfernt sie. */
  setEstimate: (id: number, minutes: number | null) => Promise<void>;
}

export const usePlanStore = create<PlanStoreState>((set) => {
  const guard = async (action: () => Promise<void>): Promise<void> => {
    try {
      await action();
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };
  /** Die Planung steckt in der Beschreibung: die Todo-Liste lädt neu, die Ansicht „Heute“ folgt ihr. */
  const reloadTodos = (): Promise<void> => useTodoStore.getState().load();

  return {
    plan: null,
    load: () =>
      guard(async () => set({ plan: await unwrap(commands.getDayPlan()) })),
    planTodo: (id, day) =>
      guard(async () => {
        await unwrap(commands.planTodo(id, day));
        await reloadTodos();
      }),
    setEstimate: (id, minutes) =>
      guard(async () => {
        await unwrap(commands.setTodoEstimate(id, minutes));
        await reloadTodos();
      }),
  };
});
