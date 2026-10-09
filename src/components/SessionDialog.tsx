import { useEffect, useId, useRef, useState } from "react";
import { useT } from "../i18n";
import {
  commands,
  errorMessage,
  unwrap,
  type SessionInput,
  type Todo,
} from "../lib/api";
import { formatTimeOfDay } from "../lib/format";
import { defaultEntry, fromLocalInput, toLocalInput } from "../lib/timesheet";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button } from "./ui";

/** Ein bestehender Eintrag, der bearbeitet wird. */
export type EditedSession = {
  id: number;
  start: string;
  end: string;
  todoId: number | null;
  note: string;
};

const FIELD =
  "min-h-11 w-full rounded-lg bg-surface-container-highest px-3 focus:bg-secondary-container";

/**
 * Dialog zum Nachtragen auf `date` (`session === null`) oder Bearbeiten einer Arbeitssession. Fehler des Backends
 * (Ende vor Beginn, zu lang, in der Zukunft) erscheinen im Dialog; er bleibt dann offen.
 * „Löschen“ fragt nach, bevor es die Session entfernt.
 */
export function SessionDialog({
  session,
  date,
  todos,
  onClose,
  onSaved,
}: {
  session: EditedSession | null;
  date: string;
  todos: Todo[];
  onClose: () => void;
  /** Nach dem Speichern oder Löschen; `overlaps` zählt die überlappten Sessions. */
  onSaved: (overlaps: number) => void;
}) {
  const t = useT();
  const titleId = useId();
  const first = useRef<HTMLInputElement>(null);
  const sessionId = session?.id ?? null;
  const [draft, setDraft] = useState(() =>
    session
      ? {
          start: toLocalInput(session.start),
          end: toLocalInput(session.end),
          todoId: session.todoId,
          note: session.note,
        }
      : { ...defaultEntry(date), todoId: null as number | null, note: "" },
  );
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirming, setConfirming] = useState(false);
  useEffect(() => first.current?.focus(), []);

  const save = async (): Promise<void> => {
    const startedAt = fromLocalInput(draft.start);
    const endedAt = fromLocalInput(draft.end);
    if (startedAt === null || endedAt === null) {
      setError(t("entry.missingTime"));
      return;
    }
    const input: SessionInput = {
      startedAt,
      endedAt,
      todoId: draft.todoId,
      note: draft.note.trim(),
    };
    setBusy(true);
    try {
      const saved = await unwrap(
        sessionId === null
          ? commands.createManualSession(input)
          : commands.updateSession(sessionId, input),
      );
      onSaved(saved.overlaps.length);
    } catch (failure) {
      setError(errorMessage(failure));
      setBusy(false);
    }
  };

  const remove = async (): Promise<void> => {
    if (sessionId === null) return;
    setBusy(true);
    try {
      await unwrap(commands.deleteSession(sessionId));
      onSaved(0);
    } catch (failure) {
      setConfirming(false);
      setError(errorMessage(failure));
      setBusy(false);
    }
  };

  if (confirming && session)
    return (
      <ConfirmDialog
        title={t("entry.deleteTitle")}
        confirmLabel={t("entry.delete")}
        danger
        busy={busy}
        onConfirm={() => void remove()}
        onCancel={() => setConfirming(false)}
      >
        <p>
          {t("entry.deleteBody", {
            start: formatTimeOfDay(session.start),
            end: formatTimeOfDay(session.end),
          })}
        </p>
      </ConfirmDialog>
    );

  const known = todos.some((todo) => todo.id === draft.todoId);
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-4">
      <form
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onSubmit={(event) => {
          event.preventDefault();
          void save();
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            onClose();
          }
        }}
        className="glass shadow-ambient flex max-h-full w-full max-w-md flex-col gap-4 overflow-y-auto rounded-2xl p-6"
      >
        <h2 id={titleId} className="headline-md">
          {sessionId === null ? t("entry.titleNew") : t("entry.titleEdit")}
        </h2>
        <label className="flex flex-col gap-1">
          <span className="label-md text-on-surface-variant">
            {t("entry.start")}
          </span>
          <input
            ref={first}
            type="datetime-local"
            value={draft.start}
            onChange={(event) =>
              setDraft({ ...draft, start: event.target.value })
            }
            className={FIELD}
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="label-md text-on-surface-variant">
            {t("entry.end")}
          </span>
          <input
            type="datetime-local"
            value={draft.end}
            onChange={(event) =>
              setDraft({ ...draft, end: event.target.value })
            }
            className={FIELD}
          />
        </label>
        <label className="flex flex-col gap-1">
          <span className="label-md text-on-surface-variant">
            {t("entry.todo")}
          </span>
          <select
            value={draft.todoId ?? ""}
            onChange={(event) =>
              setDraft({
                ...draft,
                todoId:
                  event.target.value === "" ? null : Number(event.target.value),
              })
            }
            className={FIELD}
          >
            <option value="">{t("entry.noTodo")}</option>
            {!known && draft.todoId !== null && (
              <option value={draft.todoId}>#{draft.todoId}</option>
            )}
            {todos.map((todo) => (
              <option key={todo.id} value={todo.id}>
                {todo.description}
              </option>
            ))}
          </select>
        </label>
        <label className="flex flex-col gap-1">
          <span className="label-md text-on-surface-variant">
            {t("entry.note")}
          </span>
          <input
            type="text"
            value={draft.note}
            onChange={(event) =>
              setDraft({ ...draft, note: event.target.value })
            }
            autoComplete="off"
            className={FIELD}
          />
        </label>
        {error && (
          <p
            role="alert"
            className="rounded-xl bg-surface-container-low p-3 text-red-300"
          >
            {error}
          </p>
        )}
        <div className="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          {sessionId !== null && (
            <Button
              variant="danger"
              disabled={busy}
              onClick={() => setConfirming(true)}
              className="sm:mr-auto"
            >
              {t("entry.delete")}
            </Button>
          )}
          <Button variant="ghost" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" variant="primary" disabled={busy}>
            {t("entry.save")}
          </Button>
        </div>
      </form>
    </div>
  );
}
