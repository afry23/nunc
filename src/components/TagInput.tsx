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
  const [text, setText] = useState("");

  const commit = (): void => {
    onChange(addTag(value, text, prefix));
    setText("");
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
      <input
        type="text"
        list={listId}
        value={text}
        placeholder={addLabel}
        aria-label={addLabel}
        autoComplete="off"
        onChange={(event) => setText(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === "," || event.key === " ") {
            event.preventDefault();
            commit();
          }
        }}
        onBlur={commit}
        className="min-h-11 w-full rounded-lg bg-surface-container-highest px-3 focus:bg-secondary-container"
      />
      <datalist id={listId}>
        {matchingSuggestions(suggestions, value, text).map((tag) => (
          <option key={tag} value={tag} />
        ))}
      </datalist>
    </div>
  );
}
