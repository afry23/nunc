import { useEffect, useId, useRef } from "react";
import { useT } from "../i18n";
import { useSyncStore } from "../stores/syncStore";
import { Button } from "./ui";

/**
 * Angebot auf einem Gerät ohne Daten: den Stand aus Google Drive übernehmen. Zeigt während des Imports einen
 * Fortschrittsbalken und danach eine Zusammenfassung.
 */
export function ImportDialog() {
  const t = useT();
  const offer = useSyncStore((s) => s.importOffer);
  const { runImport, closeImport } = useSyncStore.getState();
  const titleId = useId();
  const first = useRef<HTMLButtonElement>(null);
  const phase = offer?.phase;
  useEffect(() => first.current?.focus(), [phase]);
  if (!offer) return null;
  const { info } = offer;

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/60 p-4">
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="glass shadow-ambient w-full max-w-md rounded-2xl p-6"
      >
        <h2 id={titleId} className="headline-md">
          {t("import.title")}
        </h2>
        <div className="mt-3 flex flex-col gap-3 text-on-surface-variant">
          {offer.phase === "offer" && (
            <>
              <p>
                {t("import.offer", {
                  todos: t("import.todos", { count: info.todoCount }),
                  sessions: t("import.sessions", { count: info.sessionCount }),
                })}
              </p>
              <p className="text-sm">{t("import.emptyHint")}</p>
            </>
          )}
          {offer.phase === "running" && (
            <div
              role="progressbar"
              aria-label={t("import.running")}
              className="h-2 overflow-hidden rounded-full bg-surface-container-highest"
            >
              <div className="h-full w-1/3 animate-pulse rounded-full bg-primary" />
            </div>
          )}
          {offer.phase === "done" && <p role="status">{offer.summary}</p>}
        </div>
        <div className="mt-6 flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
          {offer.phase === "offer" && (
            <>
              <Button variant="ghost" onClick={closeImport}>
                {t("import.startEmpty")}
              </Button>
              <Button
                ref={first}
                variant="primary"
                onClick={() => void runImport()}
              >
                {t("import.import")}
              </Button>
            </>
          )}
          {offer.phase === "running" && (
            <Button variant="primary" disabled>
              {t("import.importing")}
            </Button>
          )}
          {offer.phase === "done" && (
            <Button ref={first} variant="primary" onClick={closeImport}>
              {t("import.done")}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
