import { useT } from "../i18n";
import { openExternal } from "../lib/links";
import { priorityTone, splitSegments } from "../lib/todos";

const TONE_CLASS = {
  high: "text-primary",
  medium: "text-orange-300",
  low: "text-on-surface-variant",
  none: "",
} as const;

/** Priorität „(A)“, nach Dringlichkeit eingefärbt. Ohne Priorität nichts. */
export function PriorityMark({ priority }: { priority: string | null }) {
  const t = useT();
  if (!priority) return null;
  return (
    <span
      className={`font-semibold ${TONE_CLASS[priorityTone(priority)]}`}
      aria-label={t("todos.item.priority", { priority })}
    >
      ({priority})
    </span>
  );
}

/** Beschreibung mit hervorgehobenen Projekten (+) und Kontexten (@), Links mit Anzeigetext, ohne Planungsfelder. */
export function DescriptionText({ description }: { description: string }) {
  return (
    <>
      {splitSegments(description).map((segment, index) => {
        switch (segment.kind) {
          case "field":
            return null;
          case "link": {
            const { href } = segment;
            return (
              // Kein <a>/<button>: steht in anklickbaren Zeilen (Details, Auswahl); Klick öffnet nur den Link.
              <span
                key={index}
                role="link"
                className="cursor-pointer text-primary underline"
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  void openExternal(href);
                }}
              >
                {segment.text}
              </span>
            );
          }
          case "project":
            return (
              <span key={index} className="text-primary">
                {segment.text}
              </span>
            );
          case "context":
            return (
              <span key={index} className="text-secondary-fixed-dim">
                {segment.text}
              </span>
            );
          case "plain":
            return <span key={index}>{segment.text}</span>;
        }
      })}
    </>
  );
}
