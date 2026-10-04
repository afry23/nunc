import { useEffect, useId, useRef } from "react";
import { useSyncStore } from "../stores/syncStore";
import { Button } from "./ui";

function countText(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

/**
 * Angebot auf einem Gerät ohne Daten: den Stand aus Google Drive übernehmen. Zeigt während des Imports einen
 * Fortschrittsbalken und danach eine Zusammenfassung.
 */
export function ImportDialog() {
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
          Vorhandene Daten von Google Drive importieren?
        </h2>
        <div className="mt-3 flex flex-col gap-3 text-on-surface-variant">
          {offer.phase === "offer" && (
            <>
              <p>
                Auf Google Drive liegen{" "}
                {countText(info.todoCount, "Aufgabe", "Aufgaben")} und{" "}
                {countText(info.sessionCount, "Session", "Sessions")}.
              </p>
              <p className="text-sm">
                Mit „Leer starten“ werden die Daten erst beim nächsten Abgleich
                übernommen.
              </p>
            </>
          )}
          {offer.phase === "running" && (
            <div
              role="progressbar"
              aria-label="Import läuft"
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
                Leer starten
              </Button>
              <Button
                ref={first}
                variant="primary"
                onClick={() => void runImport()}
              >
                Importieren
              </Button>
            </>
          )}
          {offer.phase === "running" && (
            <Button variant="primary" disabled>
              Importiere …
            </Button>
          )}
          {offer.phase === "done" && (
            <Button ref={first} variant="primary" onClick={closeImport}>
              Fertig
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}
