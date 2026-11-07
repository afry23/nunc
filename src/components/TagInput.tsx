import { useId, useState } from "react";
import { useT } from "../i18n";
import {
  addTag,
  matchingSuggestions,
  removeTag,
  type TagPrefix,
} from "../lib/tags";

/**
 * Auswahl mehrerer Projekte (`+`) oder Kontexte (`@`): gewählte Tags als Chips, darunter ein Eingabefeld mit
 * Vorschlägen. Enter, Komma, Leerzeichen und Verlassen des Feldes übernehmen die Eingabe als neues Tag.
 */
export function TagInput({
  label,
  addLabel,
  prefix,
  value,
  suggestions,
  onChange,
}: {
  label: string;
  addLabel: string;
  prefix: TagPrefix;
  value: string[];
  suggestions: string[];
  onChange: (tags: string[]) => void;
}) {
  const t = useT();
  const listId = useId();
  const inputId = useId();
  const [text, setText] = useState("");
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const options = matchingSuggestions(suggestions, value, text);

  const commit = (): void => {
    onChange(addTag(value, text, prefix));
    setText("");
    setActive(-1);
  };

  const pick = (tag: string): void => {
    onChange(addTag(value, tag, prefix));
    setText("");
    setActive(-1);
    setOpen(false);
  };

  return (
    <div className="flex flex-col gap-2">
      <span className="label-md text-on-surface-variant">{label}</span>
      {value.length > 0 && (
        <ul aria-label={label} className="flex flex-wrap gap-2">
          {value.map((tag) => (
            <li key={tag}>
              <button
                type="button"
                aria-label={t("labels.remove", { tag })}
                onClick={() => onChange(removeTag(value, tag))}
                className="inline-flex min-h-9 items-center gap-1 rounded-lg bg-surface-container-high px-3 text-sm hover:bg-surface-bright"
              >
                {tag} <span aria-hidden="true">×</span>
              </button>
            </li>
          ))}
        </ul>
      )}
      <div className="relative">
        <input
          id={inputId}
          type="text"
          role="combobox"
          aria-expanded={open && options.length > 0}
          aria-controls={listId}
          aria-autocomplete="list"
          value={text}
          placeholder={addLabel}
          aria-label={addLabel}
          autoComplete="off"
          onChange={(event) => {
            setText(event.target.value);
            setActive(-1);
            setOpen(true);
          }}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown" || event.key === "ArrowUp") {
              event.preventDefault();
              setOpen(true);
              const step = event.key === "ArrowDown" ? 1 : -1;
              setActive((i) =>
                options.length === 0
                  ? -1
                  : (i + step + options.length) % options.length,
              );
            } else if (event.key === "Escape" && open) {
              event.stopPropagation();
              setOpen(false);
            } else if (
              event.key === "Tab" &&
              !event.shiftKey &&
              open &&
              options.length > 0 &&
              (active >= 0 || text.trim() !== "")
            ) {
              /* Tab übernimmt den markierten, sonst den ersten Vorschlag; ohne Treffer bleibt Tab ein Fokuswechsel */
              event.preventDefault();
              pick(options[Math.max(active, 0)]);
            } else if (
              event.key === "Enter" ||
              event.key === "," ||
              event.key === " "
            ) {
              event.preventDefault();
              if (event.key === "Enter" && options[active])
                pick(options[active]);
              else if (
                event.key === "Enter" &&
                open &&
                options.length === 1 &&
                text.trim() !== ""
              )
                pick(options[0]);
              else commit();
            }
          }}
          onBlur={() => {
            setOpen(false);
            commit();
          }}
          className="min-h-11 w-full rounded-lg bg-surface-container-highest px-3 focus:bg-secondary-container"
        />
        {open && options.length > 0 && (
          <ul
            id={listId}
            role="listbox"
            aria-labelledby={inputId}
            className="glass shadow-ambient absolute inset-x-0 top-full z-10 mt-1 max-h-48 overflow-y-auto rounded-xl p-1"
          >
            {options.map((tag, index) => (
              <li
                key={tag}
                role="option"
                aria-selected={index === active}
                /* mousedown statt click: das Feld soll den Fokus (und die Liste) nicht vorher verlieren */
                onMouseDown={(event) => {
                  event.preventDefault();
                  pick(tag);
                }}
                className={`min-h-9 cursor-pointer rounded-lg px-3 py-1.5 hover:bg-surface-bright ${index === active ? "bg-surface-bright" : ""}`}
              >
                {tag}
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
