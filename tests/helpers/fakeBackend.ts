import type {
  Settings,
  StatsResponse,
  TimerMode,
  TimerSnapshot,
  Todo,
} from "../../src/bindings";
import { mockBackend, type BackendMock } from "./ipc";

/** Fehler eines Commands, wie ihn das Backend liefert (`{ code, message }`, kein `Error`). */
export function fail(code: string, message: string): never {
  throw { code, message };
}

export const DEFAULT_SETTINGS: Settings = {
  pomodoroMin: 25,
  shortBreakMin: 5,
  longBreakMin: 25,
  longBreakAfter: 4,
  autoStartBreaks: false,
  autoStartNextPomodoro: false,
  notificationsEnabled: true,
  soundEnabled: true,
  soundVolume: 80,
  soundEndPomodoro: "bell",
  soundEndBreak: "ding",
  activeMode: "pomodoro",
  systemTrayEnabled: true,
  lastBackupDate: "",
  backupDir: "",
  syncEnabled: true,
  syncProvider: "none",
  flowtimeRatio: 0.2,
  flowtimeMinFlowStateMin: 15,
};

export const EMPTY_STATS: StatsResponse = {
  daily: [],
  projects: [],
  contexts: [],
  topTodos: [],
  weekdays: [],
  monthly: [],
  totalPomodoros: 0,
  totalWorkMin: 0,
  averagePomodoros7: 0,
  averagePomodoros30: 0,
  currentStreak: 0,
  longestStreak: 0,
};

/** Vereinfachtes todo.txt-Parsen (das echte steckt in Rust); reicht für die UI-Tests. */
export function parseTodo(id: number, position: number, rawText: string): Todo {
  const priority = /^\(([A-Z])\) /.exec(rawText)?.[1] ?? null;
  const description = rawText.replace(/^\([A-Z]\) /, "").replace(/^x \S+ /, "");
  const words = description.split(/\s+/);
  return {
    id,
    rawText,
    priority,
    completed: rawText.startsWith("x "),
    completedAt: null,
    createdAt: null,
    description,
    projectTags: words.filter((w) => w.length > 1 && w.startsWith("+")),
    contextTags: words.filter((w) => w.length > 1 && w.startsWith("@")),
    pomodoroCount: 0,
    position,
  };
}

type TodoFilterArg = {
  projects?: string[];
  contexts?: string[];
  priorities?: string[];
  text?: string;
  completed?: boolean | null;
} | null;

export interface FakeBackend extends BackendMock {
  settings: Settings;
  todos: Todo[];
  timer: TimerSnapshot;
  stats: StatsResponse;
  suggestedBreak: {
    sessionType: "short_break" | "long_break";
    minutes: number;
  } | null;
  /** Ergebnis von `save_export_to_file` (`null` = Dialog abgebrochen). */
  exportPath: string | null;
  /** Letzte Argumente je Command. */
  last: (cmd: string) => Record<string, unknown> | undefined;
  /** Anzahl der Aufrufe eines Commands. */
  count: (cmd: string) => number;
  addTodo: (rawText: string, over?: Partial<Todo>) => Todo;
}

const IDLE: TimerSnapshot = {
  state: "idle",
  mode: "pomodoro",
  sessionType: "work",
  remaining: 0,
  total: 0,
  todoId: null,
};

/** Zustandsbehaftetes Fake-Backend für `@ui`-Szenarien; unbekannte Commands werfen. */
export function createFakeBackend(): FakeBackend {
  const fake = {
    settings: { ...DEFAULT_SETTINGS },
    todos: [] as Todo[],
    timer: { ...IDLE },
    stats: { ...EMPTY_STATS },
    suggestedBreak: null,
    exportPath: "/tmp/export.csv" as string | null,
  } as FakeBackend;
  let nextId = 1;

  const resequence = (): void =>
    fake.todos.forEach((t, index) => (t.position = index));
  const find = (id: unknown): Todo => {
    const todo = fake.todos.find((t) => t.id === id);
    if (!todo) fail("not_found", `Todo ${String(id)} nicht gefunden`);
    return todo;
  };

  fake.addTodo = (rawText, over = {}) => {
    const todo = {
      ...parseTodo(nextId++, fake.todos.length, rawText),
      ...over,
    };
    fake.todos.push(todo);
    return todo;
  };

  const startSnapshot = (
    mode: TimerMode,
    todoId: number | null,
  ): TimerSnapshot => {
    const total = mode === "pomodoro" ? fake.settings.pomodoroMin * 60 : 0;
    return {
      state: "running",
      mode,
      sessionType: "work",
      remaining: total,
      total,
      todoId,
    };
  };

  const handlers: Record<string, (args: never) => unknown> = {
    get_settings: () => fake.settings,
    save_settings: ({ settings }: { settings: Settings }) => {
      if (settings.pomodoroMin < 1)
        fail(
          "invalid_settings",
          "Ungültige Einstellungen: pomodoroMin (erlaubt: ≥ 1)",
        );
      fake.settings = settings;
      return null;
    },
    get_todos: ({
      filter,
      sort,
      direction,
    }: {
      filter: TodoFilterArg;
      sort: string | null;
      direction: string | null;
    }) => {
      const f = filter ?? {};
      let result = fake.todos.filter(
        (t) =>
          (!f.projects?.length ||
            f.projects.some((p) => t.projectTags.includes(p))) &&
          (!f.contexts?.length ||
            f.contexts.some((c) => t.contextTags.includes(c))) &&
          (!f.priorities?.length ||
            (t.priority !== null && f.priorities.includes(t.priority))) &&
          (f.completed === undefined ||
            f.completed === null ||
            t.completed === f.completed) &&
          (f.text ?? "")
            .toLowerCase()
            .split(/\s+/)
            .filter(Boolean)
            .every((word) => t.description.toLowerCase().includes(word)),
      );
      result = [...result].sort((a, b) => {
        if (sort === "priority")
          return (a.priority ?? "~").localeCompare(b.priority ?? "~");
        if (sort === "alphabetical")
          return a.description.localeCompare(b.description);
        if (sort === "pomodoro_count") return a.pomodoroCount - b.pomodoroCount;
        return a.position - b.position;
      });
      return direction === "descending" ? result.reverse() : result;
    },
    add_todo: ({ rawText }: { rawText: string }) => {
      if (rawText.trim() === "")
        fail("invalid_input", "Der Text einer Aufgabe darf nicht leer sein.");
      return fake.addTodo(rawText);
    },
    update_todo: ({ id, rawText }: { id: number; rawText: string }) => {
      const todo = find(id);
      Object.assign(todo, parseTodo(todo.id, todo.position, rawText), {
        pomodoroCount: todo.pomodoroCount,
      });
      return todo;
    },
    delete_todo: ({ id }: { id: number }) => {
      find(id);
      fake.todos = fake.todos.filter((t) => t.id !== id);
      resequence();
      return null;
    },
    set_todo_completed: ({
      id,
      completed,
    }: {
      id: number;
      completed: boolean;
    }) => {
      const todo = find(id);
      todo.completed = completed;
      return todo;
    },
    reorder_todos: ({ ids }: { ids: number[] }) => {
      fake.todos = ids.map((id) => find(id));
      resequence();
      return null;
    },
    start_timer: ({
      mode,
      todoId,
    }: {
      mode: TimerMode;
      todoId: number | null;
    }) => (fake.timer = startSnapshot(mode, todoId)),
    start_break: () => {
      if (!fake.suggestedBreak)
        fail("no_break_suggested", "keine Pause vorgeschlagen");
      const total = fake.suggestedBreak.minutes * 60;
      return (fake.timer = {
        state: "running",
        mode: "pomodoro",
        sessionType: fake.suggestedBreak.sessionType,
        remaining: total,
        total,
        todoId: null,
      });
    },
    pause_timer: () => (fake.timer = { ...fake.timer, state: "paused" }),
    resume_timer: () => (fake.timer = { ...fake.timer, state: "running" }),
    stop_timer: () => (fake.timer = { ...IDLE }),
    get_timer_tick: () => fake.timer,
    get_suggested_break: () => fake.suggestedBreak,
    get_stats: () => fake.stats,
    save_export_to_file: () => fake.exportPath,
    show_main_window: () => null,
  };

  const mock = mockBackend(
    handlers as Record<string, (args: unknown) => unknown>,
  );
  fake.calls = mock.calls;
  fake.last = (cmd) =>
    mock.calls.filter((c) => c.cmd === cmd).at(-1)?.args as
      Record<string, unknown> | undefined;
  fake.count = (cmd) => mock.calls.filter((c) => c.cmd === cmd).length;
  return fake;
}
