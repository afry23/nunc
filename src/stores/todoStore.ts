import { create } from "zustand";
import {
  commands,
  unwrap,
  type SortDirection,
  type Todo,
  type TodoSort,
} from "../lib/api";
import { t } from "../i18n";
import {
  buildTree,
  describeImport,
  indentParent,
  mergeOrder,
  moveItem,
  outdentParent,
} from "../lib/todos";
import { useUiStore } from "./uiStore";

export type TodoFilterState = {
  projects: string[];
  contexts: string[];
  priorities: string[];
  text: string;
};

export const NO_FILTER: TodoFilterState = {
  projects: [],
  contexts: [],
  priorities: [],
  text: "",
};

interface TodoStoreState {
  /** Alle aktiven Todos in manueller Reihenfolge (Grundlage für Tag-Listen und Umsortieren). */
  all: Todo[];
  /** Gefilterte und sortierte Ansicht. */
  visible: Todo[];
  filter: TodoFilterState;
  /** Eingeklappte Todos (zeigen ihre Unteraufgaben nicht). */
  collapsed: ReadonlySet<number>;
  showCompleted: boolean;
  sort: TodoSort;
  direction: SortDirection;
  load: () => Promise<void>;
  add: (rawText: string) => Promise<boolean>;
  /** Legt eine Unteraufgabe unter `parentId` an. */
  addSubtask: (parentId: number, rawText: string) => Promise<boolean>;
  update: (id: number, rawText: string) => Promise<boolean>;
  toggleCollapsed: (id: number) => void;
  /** Rückt das Todo unter sein vorheriges Geschwister ein. */
  indent: (id: number) => Promise<void>;
  /** Rückt das Todo eine Ebene aus. */
  outdent: (id: number) => Promise<void>;
  remove: (id: number) => Promise<void>;
  toggle: (id: number) => Promise<void>;
  /** Verschiebt ein Todo in der sichtbaren Reihenfolge um `delta` Plätze (nur bei manueller Sortierung). */
  move: (id: number, delta: number) => Promise<void>;
  /** Setzt die sichtbare Reihenfolge neu (Drag & Drop). */
  reorderVisible: (orderedIds: number[]) => Promise<void>;
  setText: (text: string) => Promise<void>;
  toggleTag: (
    kind: "projects" | "contexts" | "priorities",
    tag: string,
  ) => Promise<void>;
  clearFilters: () => Promise<void>;
  setShowCompleted: (show: boolean) => Promise<void>;
  setSort: (sort: TodoSort, direction?: SortDirection) => Promise<void>;
  /** Fragt nach einer todo.txt-Datei und legt ihre neuen Zeilen an; zeigt das Ergebnis als Hinweis. */
  importFile: () => Promise<void>;
  /** Speichert alle Aufgaben in eine todo.txt-Datei; zeigt den Pfad als Hinweis. */
  exportFile: () => Promise<void>;
}

export const useTodoStore = create<TodoStoreState>((set, get) => {
  const guard = async (action: () => Promise<void>): Promise<void> => {
    try {
      await action();
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };

  const fetchVisible = async (): Promise<Todo[]> => {
    const { filter, showCompleted, sort, direction } = get();
    return unwrap(
      commands.getTodos(
        {
          projects: filter.projects,
          contexts: filter.contexts,
          priorities: filter.priorities,
          text: filter.text,
          completed: showCompleted ? null : false,
        },
        sort,
        direction,
      ),
    );
  };

  const reload = async (): Promise<void> => {
    const [all, visible] = await Promise.all([
      unwrap(commands.getTodos(null, "manual", "ascending")),
      fetchVisible(),
    ]);
    set({ all, visible });
  };

  const refilter = (change: Partial<TodoStoreState>): Promise<void> => {
    set(change);
    return guard(async () => set({ visible: await fetchVisible() }));
  };

  return {
    all: [],
    visible: [],
    filter: NO_FILTER,
    collapsed: new Set(),
    showCompleted: false,
    sort: "manual",
    direction: "ascending",
    load: () => guard(reload),
    importFile: () =>
      guard(async () => {
        const result = await unwrap(commands.importTodotxtFile());
        if (!result) return;
        useUiStore.getState().notify(describeImport(result));
        await reload();
      }),
    exportFile: () =>
      guard(async () => {
        const path = await unwrap(commands.saveTodotxtFile());
        if (path)
          useUiStore.getState().notify(t("todos.exportSaved", { path }));
      }),
    add: async (rawText) => {
      try {
        await unwrap(commands.addTodo(rawText));
        await reload();
        return true;
      } catch (error) {
        useUiStore.getState().fail(error);
        return false;
      }
    },
    addSubtask: async (parentId, rawText) => {
      try {
        await unwrap(commands.addSubtask(parentId, rawText));
        set({
          collapsed: new Set(
            [...get().collapsed].filter((c) => c !== parentId),
          ),
        });
        await reload();
        return true;
      } catch (error) {
        useUiStore.getState().fail(error);
        return false;
      }
    },
    toggleCollapsed: (id) => {
      const next = new Set(get().collapsed);
      if (!next.delete(id)) next.add(id);
      set({ collapsed: next });
    },
    indent: (id) =>
      guard(async () => {
        const rows = buildTree(get().visible);
        const index = rows.findIndex((row) => row.todo.id === id);
        const parent = indentParent(rows, index);
        if (parent === null) return;
        await unwrap(commands.setTodoParent(id, parent));
        await reload();
      }),
    outdent: (id) =>
      guard(async () => {
        const rows = buildTree(get().visible);
        const index = rows.findIndex((row) => row.todo.id === id);
        const parent = outdentParent(rows, index);
        if (parent === undefined) return;
        await unwrap(commands.setTodoParent(id, parent));
        await reload();
      }),
    update: async (id, rawText) => {
      try {
        await unwrap(commands.updateTodo(id, rawText));
        await reload();
        return true;
      } catch (error) {
        useUiStore.getState().fail(error);
        return false;
      }
    },
    remove: (id) =>
      guard(async () => {
        await unwrap(commands.deleteTodo(id));
        await reload();
      }),
    toggle: (id) =>
      guard(async () => {
        const todo = get().all.find((t) => t.id === id);
        if (!todo) return;
        await unwrap(commands.setTodoCompleted(id, !todo.completed));
        await reload();
      }),
    move: (id, delta) =>
      guard(async () => {
        const { visible } = get();
        const ids = visible.map((t) => t.id);
        // nur unter Geschwistern verschieben: das Ziel ist das nächste Geschwister in die Richtung
        const self = visible.find((t) => t.id === id);
        const siblings = visible
          .filter((t) => (t.parentId ?? null) === (self?.parentId ?? null))
          .map((t) => t.id);
        const target = siblings[siblings.indexOf(id) + delta];
        if (target === undefined) return;
        await get().reorderVisible(
          moveItem(ids, ids.indexOf(id), ids.indexOf(target)),
        );
      }),
    reorderVisible: (orderedIds) =>
      guard(async () => {
        const allIds = get().all.map((t) => t.id);
        const next = mergeOrder(allIds, orderedIds);
        if (next.every((id, index) => id === allIds[index])) return;
        await unwrap(commands.reorderTodos(next));
        await reload();
      }),
    setText: (text) => refilter({ filter: { ...get().filter, text } }),
    toggleTag: (kind, tag) => {
      const current = get().filter[kind];
      const next = current.includes(tag)
        ? current.filter((t) => t !== tag)
        : [...current, tag];
      return refilter({ filter: { ...get().filter, [kind]: next } });
    },
    clearFilters: () => refilter({ filter: NO_FILTER }),
    setShowCompleted: (showCompleted) => refilter({ showCompleted }),
    setSort: (sort, direction) =>
      refilter({ sort, direction: direction ?? get().direction }),
  };
});
