import { useEffect, useRef, useState } from "react";
import { useT } from "../i18n";
import { commands, unwrap } from "../lib/api";
import { sessionLabel } from "../lib/timer";
import { useTimerStore, type Finish } from "../stores/timerStore";
import { useUiStore } from "../stores/uiStore";
import { Button } from "./ui";

/**
 * Nach einer beendeten Arbeitssession: optional Ergebnis und nächste Schritte als Notiz festhalten und, wenn das
 * Backend eine Pause empfiehlt, diese starten oder auf später verschieben.
 */
export function SessionEndDialog() {
  const finish = useTimerStore((s) => s.finish);
  const session = finish?.payload.session;
  if (
    !finish ||
    !session ||
    session.sessionType !== "work" ||
    (session.actualMin ?? 0) <= 0
  )
    return null;
  // neuer Schlüssel je Session: das Formular beginnt mit der Notiz dieser Session
  return <EndForm key={session.id} finish={finish} />;
}

function EndForm({ finish }: { finish: Finish }) {
  const t = useT();
  const dismiss = useTimerStore((s) => s.dismissFinish);
  const startBreak = useTimerStore((s) => s.startBreak);
  const primary = useRef<HTMLButtonElement>(null);
  const field = useRef<HTMLTextAreaElement>(null);
  const [initial, setInitial] = useState("");
  const [note, setNote] = useState("");
  const session = finish.payload.session;
  const suggestion =
    finish.kind === "completed" ? finish.payload.suggestedBreak : null;
  const sessionId = session.id;

  // die vorhandene Notiz (z. B. aus einer freien Tätigkeit) bleibt erhalten und wird ergänzt
  useEffect(() => {
    let cancelled = false;
    unwrap(commands.getSessionNote(sessionId)).then(
      (text) => {
        if (cancelled) return;
        setInitial(text);
        setNote(text);
      },
      () => undefined,
    );
    field.current?.focus();
    return () => {
      cancelled = true;
    };
  }, [sessionId]);

  const changed = note.trim() !== initial.trim();

  const save = async (): Promise<void> => {
    if (!changed) return;
    try {
      await unwrap(commands.setSessionNote(session.id, note));
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };
  const close = async (): Promise<void> => {
    await save();
    dismiss();
  };

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="session-end-title"
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            dismiss();
          }
        }}
        className="glass shadow-ambient w-full max-w-sm rounded-2xl p-6"
      >
        <h2 id="session-end-title" className="headline-md text-center">
          {suggestion
            ? session.mode === "flowtime"
              ? t("timer.flowDone")
              : t("timer.pomodoroDone")
            : t("timer.ended")}
        </h2>
        {suggestion && (
          <p className="mt-3 text-center text-on-surface-variant">
            {t("timer.suggested")}{" "}
            <strong className="text-on-surface">
              {t("timer.minutes", { count: suggestion.minutes })}
            </strong>{" "}
            ({sessionLabel(session.mode, suggestion.sessionType)})
          </p>
        )}
        <label className="mt-4 flex flex-col gap-1">
          <span className="label-md text-on-surface-variant">
            {t("timer.result")}
          </span>
          <textarea
            ref={field}
            value={note}
            rows={3}
            onChange={(event) => setNote(event.target.value)}
            className="w-full min-w-0 rounded-lg bg-surface-container-highest p-3 text-on-surface focus:bg-secondary-container"
          />
        </label>
        <div className="mt-6 flex flex-col gap-2">
          {suggestion ? (
            <>
              <Button
                ref={primary}
                variant="primary"
                onClick={() => void save().then(() => startBreak())}
              >
                {t("timer.startBreak")}
              </Button>
              <Button variant="ghost" onClick={() => void close()}>
                {t("timer.later")}
              </Button>
            </>
          ) : (
            <>
              <Button
                ref={primary}
                variant="primary"
                onClick={() => void close()}
              >
                {t("timer.result.save")}
              </Button>
              <Button variant="ghost" onClick={dismiss}>
                {t("timer.result.skip")}
              </Button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
