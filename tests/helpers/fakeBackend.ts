import { emit } from "@tauri-apps/api/event";
import type {
  BackupEntry,
  BackupInfo,
  BackupStatus,
  ConnectionTestResult,
  GroupBy,
  KnownTags,
  TimelineBlock,
  SyncResult,
  SyncStatus,
  WebDavSettings,
  Settings,
  StatsResponse,
  TimerMode,
  TimerSnapshot,
  Todo,
} from "../../src/bindings";
import { addDays, parseIsoDate, toIsoDate } from "../../src/lib/format";
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
  language: "de",
  apiEnabled: false,
  apiPort: 42820,
  syncIntervalMin: 15,
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
    parentId: null,
  };
}

type TodoFilterArg = {
  projects?: string[];
  contexts?: string[];
  priorities?: string[];
  text?: string;
  completed?: boolean | null;
} | null;

export type TimesheetSeed = { group: string | null; minutes: number[] };

export interface FakeBackend extends BackendMock {
  settings: Settings;
  todos: Todo[];
  timer: TimerSnapshot;
  stats: StatsResponse;
  suggestedBreak: {
    sessionType: "short_break" | "long_break";
    minutes: number;
  } | null;
  /** Systemstand der Benachrichtigungs-Berechtigung (mobil); `request_…` entscheidet nach `permissionAnswer`. */
  notificationPermission: "granted" | "denied" | "prompt";
  permissionAnswer: "granted" | "denied";
  /** Ergebnis von `save_export_to_file` (`null` = Dialog abgebrochen). */
  exportPath: string | null;
  /** Ergebnis von `import_todotxt_file` (`null` = Dialog abgebrochen) und von `save_todotxt_file`. */
  todotxtImport: { added: number; duplicates: number } | null;
  todotxtExportPath: string | null;
  /** Fehler, den `get_api_status` meldet, obwohl die Schnittstelle laufen soll (`null` = läuft). */
  apiError: string | null;
  /** Der Server startet noch im Hintergrund: `get_api_status` meldet ihn als aus, ohne Fehler. */
  apiStarting: boolean;
  /** Token der lokalen Schnittstelle; `regenerate_api_token` setzt `apiNextToken`. */
  apiToken: string;
  apiNextToken: string;
  /** Backups im Backup-Ordner (neueste zuerst); Status und Liste leiten sich daraus ab. */
  backups: BackupEntry[];
  /** Todo-Zeilen je Backup-Pfad, die ein Restore einspielt. */
  backupContents: Record<string, string[]>;
  /** Ergebnis von `trigger_backup`; `"hold"` hält den Aufruf an, bis `releaseBackup` ruft. */
  backupResult: BackupInfo;
  /** Meldet der Status „veraltet“ (die Zeitrechnung selbst gehört dem Backend). */
  backupsOutdated: boolean;
  holdBackup: boolean;
  releaseBackup: () => void;
  /** Ergebnisse der Datei- und Ordner-Dialoge (`null` = abgebrochen). */
  restoreFile: string | null;
  backupDirChoice: string | null;
  /** Ziel des „Speichern unter“-Dialogs beim Export (`null` = abgebrochen). */
  exportTarget: string | null;
  /** Gespeicherte WebDAV-Einstellungen und das Ergebnis des Verbindungstests. */
  webdav: WebDavSettings;
  webdavTest: ConnectionTestResult;
  /** Stand der Synchronisierung und was der nächste `sync_now` tut (`null` = nichts zu synchronisieren). */
  syncStatus: SyncStatus;
  syncOutcome: SyncResult | { error: string } | null;
  /** Im Browser geöffnete Adressen. */
  openedUrls: string[];
  /** Fehler, den `set_sync_provider` meldet (`null` = Erfolg). */
  providerError: { code: string; message: string } | null;
  /** Fehler, den `restore_from_backup` meldet (`null` = Erfolg). */
  restoreError: { code: string; message: string } | null;
  /** Zeilen von `get_timesheet` je Gruppierung: Minuten für Montag bis Sonntag. */
  timesheet: Partial<Record<GroupBy, TimesheetSeed[]>>;
  /** Blöcke von `get_day_timeline` je lokalem Tag. */
  timeline: Record<string, TimelineBlock[]>;
  /** Vorschläge, die `get_known_tags` liefert. */
  knownTags: KnownTags;
  /** Fehler, den `create_manual_session` und `update_session` melden (`null` = Erfolg). */
  sessionError: { code: string; message: string } | null;
  /** Letzte Argumente je Command. */
  last: (cmd: string) => Record<string, unknown> | undefined;
  /** Anzahl der Aufrufe eines Commands. */
  count: (cmd: string) => number;
  addTodo: (rawText: string, over?: Partial<Todo>) => Todo;
}

type SessionInputArg = {
  startedAt: string;
  endedAt: string;
  todoId: number | null;
  note: string;
  projectTags?: string[];
  contextTags?: string[];
};

function sessionOf(block: TimelineBlock) {
  const minutes = Math.round(
    (new Date(block.end).getTime() - new Date(block.start).getTime()) / 60_000,
  );
  return {
    id: block.sessionId,
    mode: "flowtime" as const,
    sessionType: block.sessionType,
    status: block.status,
    startedAt: block.start,
    endedAt: block.end,
    plannedMin: null,
    actualMin: minutes,
    todoId: block.todoId,
    breakMin: null,
  };
}

function backupStatus(
  backups: BackupEntry[],
  isOutdated: boolean,
): BackupStatus {
  return {
    lastBackupAt: backups[0]?.createdAt ?? null,
    backupCount: backups.length,
    totalSizeBytes: backups.reduce((sum, b) => sum + b.sizeBytes, 0),
    isOutdated,
  };
}

const IDLE: TimerSnapshot = {
  state: "idle",
  mode: "pomodoro",
  sessionType: "work",
  remaining: 0,
  total: 0,
  todoId: null,
  labels: { projectTags: [], contextTags: [], note: "" },
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
    todotxtImport: null as { added: number; duplicates: number } | null,
    todotxtExportPath: null as string | null,
    apiError: null as string | null,
    apiStarting: false,
    apiToken: "test-token",
    apiNextToken: "new-token",
    notificationPermission: "granted",
    permissionAnswer: "granted",
    backups: [],
    backupContents: {},
    backupResult: {
      path: "/data/backups/nunc-backup-2026-10-04.db",
      sizeBytes: 2048,
      deleted: [],
      rotationError: null,
    },
    holdBackup: false,
    releaseBackup: () => undefined,
    backupsOutdated: false,
    restoreFile: null,
    backupDirChoice: null,
    restoreError: null,
    timesheet: {},
    timeline: {},
    sessionError: null,
    knownTags: { projects: [], contexts: [] },
    providerError: null,
    openedUrls: [],
    webdav: {
      config: {
        url: "",
        authMethod: "basic",
        username: "",
        remotePath: "/nunc/",
      },
      hasPassword: false,
    },
    webdavTest: { success: true, message: "dav.example.org" },
    syncStatus: { state: "idle", lastSyncAt: null, errorMessage: null },
    syncOutcome: { importedTodos: 0, importedSessions: 0, conflictedTodos: 0 },
    exportTarget: "/storage/Download/nunc-backup.db",
  } as unknown as FakeBackend;
  let releaseHeld: (() => void) | undefined;
  fake.releaseBackup = () => releaseHeld?.();
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
    labels: TimerSnapshot["labels"] = {
      projectTags: [],
      contextTags: [],
      note: "",
    },
  ): TimerSnapshot => {
    const total = mode === "pomodoro" ? fake.settings.pomodoroMin * 60 : 0;
    return {
      state: "running",
      mode,
      sessionType: "work",
      remaining: total,
      total,
      todoId,
      labels:
        todoId === null
          ? labels
          : { projectTags: [], contextTags: [], note: "" },
    };
  };

  const handlers: Record<string, (args: never) => unknown> = {
    app_info: () => ({ name: "nunc", version: "1.2.3" }),
    get_settings: () => fake.settings,
    get_notification_status: () => ({
      permission: fake.notificationPermission,
    }),
    request_notification_permission: () => {
      if (fake.notificationPermission === "prompt")
        fake.notificationPermission = fake.permissionAnswer;
      return { permission: fake.notificationPermission };
    },
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
    add_subtask: ({
      parentId,
      rawText,
    }: {
      parentId: number;
      rawText: string;
    }) => {
      find(parentId);
      return fake.addTodo(rawText, { parentId });
    },
    set_todo_parent: ({
      id,
      parentId,
    }: {
      id: number;
      parentId: number | null;
    }) => {
      const todo = find(id);
      if (parentId !== null) find(parentId);
      todo.parentId = parentId;
      return todo;
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
      labels,
    }: {
      mode: TimerMode;
      todoId: number | null;
      labels: TimerSnapshot["labels"] | null;
    }) => (fake.timer = startSnapshot(mode, todoId, labels ?? undefined)),
    get_known_tags: () => fake.knownTags,
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
        labels: { projectTags: [], contextTags: [], note: "" },
      });
    },
    pause_timer: () => (fake.timer = { ...fake.timer, state: "paused" }),
    resume_timer: () => (fake.timer = { ...fake.timer, state: "running" }),
    stop_timer: () => (fake.timer = { ...IDLE }),
    get_timer_tick: () => fake.timer,
    get_suggested_break: () => fake.suggestedBreak,
    get_stats: () => fake.stats,
    get_timesheet: ({
      startDate,
      endDate,
      groupBy,
    }: {
      startDate: string;
      endDate: string;
      groupBy: GroupBy;
    }) => {
      const days: string[] = [];
      for (
        let day = parseIsoDate(startDate);
        toIsoDate(day) <= endDate;
        day = addDays(day, 1)
      )
        days.push(toIsoDate(day));
      const rows = (fake.timesheet[groupBy] ?? []).map((seed) => {
        const minutes = days.map((_, index) => seed.minutes[index] ?? 0);
        return {
          group: seed.group,
          todoId: null,
          minutes,
          total: minutes.reduce((sum, m) => sum + m, 0),
        };
      });
      const dayTotals = days.map((_, index) =>
        rows.reduce((sum, row) => sum + row.minutes[index], 0),
      );
      return {
        days,
        rows,
        dayTotals,
        total: dayTotals.reduce((sum, m) => sum + m, 0),
      };
    },
    get_day_timeline: ({ date }: { date: string }) => ({
      date,
      blocks: fake.timeline[date] ?? [],
      gaps: [],
    }),
    create_manual_session: ({ input }: { input: SessionInputArg }) => {
      if (fake.sessionError) throw fake.sessionError;
      const date = toIsoDate(new Date(input.startedAt));
      const block: TimelineBlock = {
        sessionId: 1000 + nextId++,
        start: input.startedAt,
        end: input.endedAt,
        sessionType: "work",
        status: "completed",
        todoId: input.todoId,
        note: input.note,
        origin: "manual",
        projectTags: input.todoId === null ? (input.projectTags ?? []) : [],
        contextTags: input.todoId === null ? (input.contextTags ?? []) : [],
        overlaps: false,
      };
      fake.timeline[date] = [...(fake.timeline[date] ?? []), block];
      return { session: sessionOf(block), overlaps: [] };
    },
    update_session: ({ id, input }: { id: number; input: SessionInputArg }) => {
      if (fake.sessionError) throw fake.sessionError;
      for (const blocks of Object.values(fake.timeline)) {
        const block = blocks.find((b) => b.sessionId === id);
        if (block) {
          Object.assign(block, {
            start: input.startedAt,
            end: input.endedAt,
            todoId: input.todoId,
            note: input.note,
            projectTags: input.todoId === null ? (input.projectTags ?? []) : [],
            contextTags: input.todoId === null ? (input.contextTags ?? []) : [],
          });
          return { session: sessionOf(block), overlaps: [] };
        }
      }
      return fail("not_found", `Session ${id} nicht gefunden`);
    },
    delete_session: ({ id }: { id: number }) => {
      for (const [date, blocks] of Object.entries(fake.timeline))
        fake.timeline[date] = blocks.filter((b) => b.sessionId !== id);
      return null;
    },
    save_export_to_file: () => fake.exportPath,
    get_api_status: () => ({
      running:
        fake.settings.apiEnabled && fake.apiError === null && !fake.apiStarting,
      port: fake.settings.apiEnabled ? fake.settings.apiPort : null,
      error: fake.settings.apiEnabled ? fake.apiError : null,
    }),
    get_api_token: () => fake.apiToken,
    regenerate_api_token: () => {
      fake.apiToken = fake.apiNextToken;
      return fake.apiToken;
    },
    import_todotxt_file: () => fake.todotxtImport,
    save_todotxt_file: () => fake.todotxtExportPath,
    show_main_window: () => null,
    get_backup_status: () => backupStatus(fake.backups, fake.backupsOutdated),
    list_backups: () => fake.backups,
    trigger_backup: async () => {
      if (fake.holdBackup)
        await new Promise<void>((resolve) => (releaseHeld = resolve));
      return fake.backupResult;
    },
    select_backup_dir: () => {
      if (fake.backupDirChoice !== null)
        fake.settings = { ...fake.settings, backupDir: fake.backupDirChoice };
      return fake.backupDirChoice;
    },
    select_restore_file: () => fake.restoreFile,
    export_backup: () => fake.exportTarget,
    get_webdav_config: () => fake.webdav,
    save_webdav_config: ({
      config,
      password,
    }: {
      config: WebDavSettings["config"];
      password: string | null;
    }) => {
      fake.webdav = {
        config,
        hasPassword: password !== null || fake.webdav.hasPassword,
      };
      return null;
    },
    test_webdav_connection: () => fake.webdavTest,
    get_sync_status: () => fake.syncStatus,
    sync_in_background: () => null,
    "plugin:opener|open_url": ({ url }: { url: string }) => {
      fake.openedUrls.push(url);
      return null;
    },
    sync_now: async () => {
      const outcome = fake.syncOutcome;
      if (outcome === null) return null;
      const last = fake.syncStatus.lastSyncAt;
      await emit("sync:start");
      await emit("sync:status", {
        state: "syncing",
        lastSyncAt: last,
        errorMessage: null,
      });
      if ("error" in outcome) {
        fake.syncStatus = {
          state: "error",
          lastSyncAt: last,
          errorMessage: outcome.error,
        };
        await emit("sync:status", fake.syncStatus);
        fail("network", outcome.error);
      }
      fake.syncStatus = {
        state: "idle",
        lastSyncAt: "2026-10-04T10:00:00Z",
        errorMessage: null,
      };
      await emit("sync:status", fake.syncStatus);
      await emit("sync:complete", outcome);
      return outcome;
    },
    set_sync_provider: ({
      provider,
    }: {
      provider: Settings["syncProvider"];
    }) => {
      if (fake.providerError) throw fake.providerError;
      fake.settings = { ...fake.settings, syncProvider: provider };
      return null;
    },
    restore_from_backup: async ({ sourcePath }: { sourcePath: string }) => {
      if (fake.restoreError) throw fake.restoreError;
      const lines = fake.backupContents[sourcePath] ?? [];
      fake.todos = [];
      lines.forEach((line) => fake.addTodo(line));
      // wie das echte Backend: nach dem Austausch melden
      setTimeout(() => void emit("app:reload"), 0);
      return null;
    },
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
