import { useEffect, useRef } from "react";
import { sessionLabel } from "../lib/timer";
import { useTimerStore } from "../stores/timerStore";
import { Button } from "./ui";

/** Nach einer abgeschlossenen Arbeitssession: empfohlene Pause starten oder auf später verschieben. */
export function BreakDialog() {
  const finish = useTimerStore((s) => s.finish);
  const dismiss = useTimerStore((s) => s.dismissFinish);
  const startBreak = useTimerStore((s) => s.startBreak);
  const primary = useRef<HTMLButtonElement>(null);

  const suggestion =
    finish?.kind === "completed" ? finish.payload.suggestedBreak : null;

  useEffect(() => {
    if (suggestion) primary.current?.focus();
  }, [suggestion]);

  if (!finish || !suggestion) return null;
  const session = finish.payload.session;
  const kind = sessionLabel(session.mode, suggestion.sessionType);

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="break-title"
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.stopPropagation();
            dismiss();
          }
        }}
        className="glass shadow-ambient w-full max-w-sm rounded-2xl p-6 text-center"
      >
        <h2 id="break-title" className="headline-md">
          {session.mode === "flowtime" ? "Flow beendet" : "Pomodoro geschafft"}
        </h2>
        <p className="mt-3 text-on-surface-variant">
          Empfohlene Pause:{" "}
          <strong className="text-on-surface">
            {suggestion.minutes} Minuten
          </strong>{" "}
          ({kind})
        </p>
        <div className="mt-6 flex flex-col gap-2">
          <Button
            ref={primary}
            variant="primary"
            onClick={() => void startBreak()}
          >
            Pause starten
          </Button>
          <Button variant="ghost" onClick={dismiss}>
            Später
          </Button>
        </div>
      </div>
    </div>
  );
}
