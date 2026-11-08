import { useEffect, useId, useRef, useState, type FormEvent } from "react";
import { useT } from "../i18n";
import { useTodoStore } from "../stores/todoStore";
import { useUiStore } from "../stores/uiStore";
import { Button } from "./ui";

/** Globaler Dialog „Neue Aufgabe“ (Seitenleiste): eine Zeile in todo.txt-Syntax, Escape schließt ohne Wirkung. */
export function NewTaskDialog() {
  const open = useUiStore((s) => s.newTaskOpen);
  return open ? <NewTaskForm /> : null;
}

function NewTaskForm() {
  const t = useT();
  const titleId = useId();
  const field = useRef<HTMLInputElement>(null);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const add = useTodoStore((s) => s.add);
  const close = useUiStore((s) => s.closeNewTask);
  const notify = useUiStore((s) => s.notify);
  useEffect(() => field.current?.focus(), []);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    const text = draft.trim();
    if (!text || busy) return;
    setBusy(true);
    const ok = await add(text);
    setBusy(false);
    if (ok) {
      close();
      notify(t("todos.created"));
    }
  };

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-4">
      <form
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onSubmit={(event) => void submit(event)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            close();
          }
        }}
        className="glass shadow-ambient w-full max-w-md rounded-2xl p-6"
      >
        <h2 id={titleId} className="headline-md">
          {t("todos.newTitle")}
        </h2>
        <input
          ref={field}
          aria-label={t("todos.new")}
          placeholder={t("todos.placeholder")}
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          className="mt-3 w-full rounded-lg bg-surface-container-high px-3 py-3 text-on-surface"
        />
        <div className="mt-6 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          <Button variant="ghost" onClick={close}>
            {t("common.cancel")}
          </Button>
          <Button
            type="submit"
            variant="primary"
            disabled={busy || !draft.trim()}
          >
            {t("todos.add")}
          </Button>
        </div>
      </form>
    </div>
  );
}
