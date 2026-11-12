import { Link } from "@tanstack/react-router";
import { useEffect, useId, useRef, useState } from "react";
import { useT } from "../i18n";
import type { JiraIssueHit } from "../lib/api";
import { useJiraStore } from "../stores/jiraStore";
import { useUiStore } from "../stores/uiStore";
import { Button } from "./ui";

/** Wartezeit nach dem letzten Tastendruck, bevor gesucht wird. */
const DEBOUNCE_MS = 300;

/** Dialog „Aus Jira“ (ADR 0038): Ticket suchen und als Aufgabe übernehmen. */
export function JiraPickerDialog() {
  const open = useUiStore((s) => s.jiraPickerOpen);
  return open ? <Picker /> : null;
}

type View =
  | { state: "loading" }
  | { state: "hits"; hits: JiraIssueHit[] }
  | { state: "error"; message: string; notConfigured: boolean };

function Picker() {
  const t = useT();
  const titleId = useId();
  const field = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [view, setView] = useState<View>({ state: "loading" });
  const [busy, setBusy] = useState(false);
  const close = useUiStore((s) => s.closeJiraPicker);
  const notify = useUiStore((s) => s.notify);
  useEffect(() => field.current?.focus(), []);

  // Die erste Suche (leer = eigene offene Tickets) läuft sofort, spätere nach einer kurzen Pause. Eine
  // veraltete Antwort wird verworfen, wenn inzwischen weiter getippt wurde.
  useEffect(() => {
    let current = true;
    const run = async (): Promise<void> => {
      const result = await useJiraStore.getState().search(query.trim());
      if (!current) return;
      setView(
        result.ok
          ? { state: "hits", hits: result.hits }
          : {
              state: "error",
              message: result.message,
              notConfigured: result.notConfigured,
            },
      );
    };
    const timer = setTimeout(() => void run(), query === "" ? 0 : DEBOUNCE_MS);
    return () => {
      current = false;
      clearTimeout(timer);
    };
  }, [query]);

  const choose = async (hit: JiraIssueHit): Promise<void> => {
    if (busy) return;
    setBusy(true);
    const ok = await useJiraStore.getState().addTodo(hit.issue);
    setBusy(false);
    if (ok) {
      close();
      notify(t("todos.created"));
    }
  };

  return (
    <div className="fixed inset-0 z-40 flex items-stretch justify-center bg-black/60 sm:items-center sm:p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            close();
          }
        }}
        className="glass shadow-ambient flex h-full w-full min-w-0 flex-col gap-3 overflow-hidden p-4 sm:h-auto sm:max-h-[90vh] sm:max-w-lg sm:rounded-2xl sm:p-6"
      >
        <h2 id={titleId} className="headline-md">
          {t("jira.picker.title")}
        </h2>
        <input
          ref={field}
          type="search"
          aria-label={t("jira.picker.search")}
          placeholder={t("jira.picker.placeholder")}
          autoComplete="off"
          autoCapitalize="none"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          className="w-full min-w-0 rounded-lg bg-surface-container-high px-3 py-3 text-on-surface"
        />
        <div className="min-h-0 flex-1 overflow-y-auto" aria-live="polite">
          {view.state === "loading" && (
            <p className="p-2 text-on-surface-variant">
              {t("jira.picker.loading")}
            </p>
          )}
          {view.state === "error" && (
            <div role="alert" className="flex flex-col gap-2 p-2 text-red-300">
              <p>{view.message}</p>
              {view.notConfigured && (
                <Link
                  to="/settings"
                  onClick={close}
                  className="font-semibold text-primary underline"
                >
                  {t("jira.picker.setup")}
                </Link>
              )}
            </div>
          )}
          {view.state === "hits" && view.hits.length === 0 && (
            <p className="p-2 text-on-surface-variant">
              {t("jira.picker.empty")}
            </p>
          )}
          {view.state === "hits" && view.hits.length > 0 && (
            <ul className="flex flex-col gap-1">
              {view.hits.map((hit) => (
                <li key={hit.issue.key}>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => void choose(hit)}
                    className={`flex min-h-11 w-full min-w-0 flex-col items-start rounded-lg px-3 py-2 text-left hover:bg-surface-container-high ${hit.imported ? "opacity-60" : ""}`}
                  >
                    <span className="w-full min-w-0 truncate">
                      {hit.issue.key} · {hit.issue.summary}
                    </span>
                    {hit.imported && (
                      <>
                        {" "}
                        <span className="text-sm text-on-surface-variant">
                          {t("jira.picker.imported")}
                        </span>
                      </>
                    )}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="flex justify-end">
          <Button variant="ghost" onClick={close}>
            {t("common.cancel")}
          </Button>
        </div>
      </div>
    </div>
  );
}
