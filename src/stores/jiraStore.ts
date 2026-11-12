import { create } from "zustand";
import {
  ApiError,
  commands,
  errorMessage,
  unwrap,
  type ConnectionTestResult,
  type JiraConfig,
  type JiraIssue,
  type JiraIssueHit,
  type JiraSettings,
} from "../lib/api";
import { useTodoStore } from "./todoStore";
import { useUiStore } from "./uiStore";

/** Ergebnis einer Suche; Fehler werden als Text zurückgegeben, damit der Dialog sie selbst zeigt. */
export type JiraSearch =
  | { ok: true; hits: JiraIssueHit[] }
  | { ok: false; message: string; notConfigured: boolean };

interface JiraState {
  settings: JiraSettings | null;
  load: () => Promise<void>;
  /** Speichert den Zugang; `token: null` lässt das gespeicherte Token unverändert. */
  save: (config: JiraConfig, token: string | null) => Promise<boolean>;
  /** Prüft die Verbindung mit den übergebenen Werten; `null` bei einem Fehler des Befehls selbst. */
  test: (
    config: JiraConfig,
    token: string | null,
  ) => Promise<ConnectionTestResult | null>;
  /** Entfernt Einstellungen und Token. */
  clear: () => Promise<boolean>;
  /** Sucht Tickets; leer = eigene offene Tickets. */
  search: (query: string) => Promise<JiraSearch>;
  /** Legt aus dem Ticket eine Aufgabe an und lädt die Aufgaben neu. */
  addTodo: (issue: JiraIssue) => Promise<boolean>;
}

export const useJiraStore = create<JiraState>((set, get) => ({
  settings: null,
  load: async () => {
    try {
      set({ settings: await unwrap(commands.getJiraConfig()) });
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  },
  save: async (config, token) => {
    try {
      await unwrap(commands.saveJiraConfig(config, token));
      await get().load();
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  },
  test: async (config, token) => {
    try {
      return await unwrap(commands.testJiraConnection(config, token));
    } catch (error) {
      useUiStore.getState().fail(error);
      return null;
    }
  },
  clear: async () => {
    try {
      await unwrap(commands.clearJiraConfig());
      await get().load();
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  },
  search: async (query) => {
    try {
      return { ok: true, hits: await unwrap(commands.searchJiraIssues(query)) };
    } catch (error) {
      return {
        ok: false,
        message: errorMessage(error),
        notConfigured:
          error instanceof ApiError && error.code === "jira_not_configured",
      };
    }
  },
  addTodo: async (issue) => {
    try {
      await unwrap(commands.addTodoFromJira(issue));
      await useTodoStore.getState().load();
      return true;
    } catch (error) {
      useUiStore.getState().fail(error);
      return false;
    }
  },
}));
