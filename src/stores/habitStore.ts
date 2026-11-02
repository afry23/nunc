import { create } from "zustand";
import {
  commands,
  unwrap,
  type Goal,
  type HabitItem,
  type Schedule,
} from "../lib/api";
import { useUiStore } from "./uiStore";

interface HabitStoreState {
  /** Alle Gewohnheiten mit Stand für heute (auch archivierte). */
  items: HabitItem[];
  load: () => Promise<void>;
  /** Legt eine Gewohnheit an (`id` leer) oder ändert sie; `true`, wenn es geklappt hat. */
  save: (
    id: string | null,
    name: string,
    schedule: Schedule,
    goal: Goal,
  ) => Promise<boolean>;
  archive: (id: string, archived: boolean) => Promise<void>;
  remove: (id: string) => Promise<void>;
  /** Setzt oder entfernt den Haken für `day`. */
  setDone: (id: string, day: string, done: boolean) => Promise<void>;
}

export const useHabitStore = create<HabitStoreState>((set, get) => {
  /** Führt die Aktion aus und lädt danach neu; ein Fehler erscheint als Hinweis. */
  const change = async (action: () => Promise<unknown>): Promise<boolean> => {
    try {
      await action();
      await get().load();
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  };

  return {
    items: [],
    load: async () => {
      try {
        set({ items: await unwrap(commands.getHabitOverview()) });
      } catch (error) {
        useUiStore.getState().fail(error);
      }
    },
    save: (id, name, schedule, goal) =>
      change(() => unwrap(commands.saveHabit(id, name, schedule, goal))),
    archive: async (id, archived) => {
      await change(() => unwrap(commands.archiveHabit(id, archived)));
    },
    remove: async (id) => {
      await change(() => unwrap(commands.deleteHabit(id)));
    },
    setDone: async (id, day, done) => {
      await change(() => unwrap(commands.setHabitDone(id, day, done)));
    },
  };
});
