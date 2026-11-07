import { create } from "zustand";
import {
  commands,
  unwrap,
  type CloseAction,
  type DayPlan,
  type TimelineBlock,
} from "../lib/api";
import { trackedMinutes } from "../lib/dayPlan";
import { useTodoStore } from "./todoStore";
import { useUiStore } from "./uiStore";

interface PlanStoreState {
  /** Plan für heute; `null`, bis er geladen ist. */
  plan: DayPlan | null;
  /** Erfasste Arbeit des Plan-Tages in Minuten (aus den Sessions, nicht aus dem Plan). */
  tracked: number;
  /** Arbeitsblöcke des Plan-Tages (für die freien Tätigkeiten ohne Aufgabe). */
  blocks: TimelineBlock[];
  /** Gespeicherte Tagesnotiz von heute (leer, wenn es keine gibt). */
  note: string;
  load: () => Promise<void>;
  /** Lädt die Tagesnotiz für `day`. */
  loadNote: (day: string) => Promise<void>;
  /** Speichert die Tagesnotiz für `day`; ein leerer Text löscht sie. */
  saveNote: (day: string, text: string) => Promise<void>;
  /** Tagesabschluss; `note` `null` lässt die Notiz, wie sie ist. `true`, wenn es geklappt hat. */
  closeDay: (
    day: string,
    action: CloseAction,
    note: string | null,
  ) => Promise<boolean>;
  /** Plant das Todo für `day` ein; `null` nimmt es vom Plan. */
  planTodo: (id: number, day: string | null) => Promise<void>;
  /** Setzt die Schätzung in Minuten; `null` entfernt sie. */
  setEstimate: (id: number, minutes: number | null) => Promise<void>;
}

export const usePlanStore = create<PlanStoreState>((set, get) => {
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
    tracked: 0,
    note: "",
    blocks: [],
    load: () =>
      guard(async () => {
        const plan = await unwrap(commands.getDayPlan());
        const timeline = await unwrap(commands.getDayTimeline(plan.day));
        set({
          plan,
          tracked: trackedMinutes(timeline.blocks),
          blocks: timeline.blocks.filter((b) => b.sessionType === "work"),
        });
      }),
    loadNote: (day) =>
      guard(async () => set({ note: await unwrap(commands.getDayNote(day)) })),
    saveNote: (day, text) =>
      guard(async () => {
        await unwrap(commands.saveJournalNote("day_note", day, text));
        set({ note: text.trim() });
      }),
    closeDay: async (day, action, note) => {
      let done = false;
      await guard(async () => {
        await unwrap(commands.closeDay(action, note));
        done = true;
        await Promise.all([get().load(), get().loadNote(day), reloadTodos()]);
      });
      return done;
    },
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
