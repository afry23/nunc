import { useRef, type ComponentProps } from "react";
import { useT } from "../i18n";
import { openExternal } from "../lib/links";
import {
  priorityTone,
  rawSegments,
  splitSegments,
  type RawSegment,
} from "../lib/todos";

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

/** Farbe eines Abschnitts der Eingabezeile, abgestimmt auf `DescriptionText` und `PriorityMark`. */
function rawClass(segment: RawSegment): string {
  switch (segment.kind) {
    case "priority":
      return `font-semibold ${TONE_CLASS[priorityTone(segment.priority)]}`;
    case "project":
      return "text-primary";
    case "context":
      return "text-secondary-fixed-dim";
    case "field":
      return "text-on-surface-variant";
    case "plain":
      return "";
  }
}

type TodoTextInputProps = Omit<
  ComponentProps<"input">,
  "value" | "className"
> & {
  value: string;
  /** Hintergrund, Rundung und Außenabstände (liegen am umgebenden Element). */
  className?: string;
  /** Innenabstand und Höhe; gelten für Feld und hervorgehobene Ebene gleichermaßen. */
  fieldClassName?: string;
};

/**
 * Eingabefeld für eine Zeile in todo.txt-Syntax, das Projekte, Kontexte, Priorität und Planungsfelder beim Tippen
 * einfärbt. Der Text des Feldes ist transparent; darunter liegt eine deckungsgleiche, eingefärbte Ebene, die beim
 * waagerechten Scrollen mitläuft.
 */
export function TodoTextInput({
  value,
  className = "",
  fieldClassName = "",
  ref,
  onScroll,
  onSelect,
  ...props
}: TodoTextInputProps) {
  const mirror = useRef<HTMLDivElement>(null);
  const follow = (input: HTMLInputElement): void => {
    if (mirror.current) mirror.current.scrollLeft = input.scrollLeft;
  };
  return (
    <div className={`relative ${className}`}>
      <div
        ref={mirror}
        aria-hidden="true"
        data-highlight
        className={`pointer-events-none absolute inset-0 flex items-center overflow-hidden whitespace-pre text-on-surface ${fieldClassName}`}
      >
        {rawSegments(value).map((segment, index) => (
          <span key={index} className={rawClass(segment)}>
            {segment.text}
          </span>
        ))}
      </div>
      <input
        {...props}
        ref={ref}
        value={value}
        onScroll={(event) => {
          follow(event.currentTarget);
          onScroll?.(event);
        }}
        onSelect={(event) => {
          follow(event.currentTarget);
          onSelect?.(event);
        }}
        className={`relative w-full bg-transparent text-transparent caret-on-surface placeholder:text-on-surface-variant selection:bg-primary/30 ${fieldClassName}`}
      />
    </div>
  );
}
