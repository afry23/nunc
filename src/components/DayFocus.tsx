import { useState } from "react";
import { useT } from "../i18n";

/** Längster Tagesfokus in Zeichen: ein Satz, kein Text (ADR 0032). */
export const FOCUS_MAX = 200;

/**
 * Tagesfokus: ein kurzer Satz, was heute das Wichtigste ist. Gespeichert wird beim Verlassen des Feldes und mit Enter,
 * aber nur, wenn sich der Text geändert hat; ein leerer Text entfernt den Fokus. Beginnt mit der gespeicherten Fassung.
 */
export function DayFocus({
  focus,
  onSave,
}: {
  focus: string;
  onSave: (text: string) => void;
}) {
  const t = useT();
  const [draft, setDraft] = useState(focus);
  const save = (): void => {
    if (draft.trim() !== focus.trim()) onSave(draft);
  };
  return (
    <label className="flex flex-col gap-1">
      <span className="label-md text-on-surface-variant">
        {t("today.focusLabel")}
      </span>
      <input
        type="text"
        value={draft}
        maxLength={FOCUS_MAX}
        autoComplete="off"
        placeholder={t("today.focusPlaceholder")}
        onChange={(event) => setDraft(event.target.value)}
        onBlur={save}
        onKeyDown={(event) => {
          if (event.key === "Enter") save();
        }}
        className="min-h-12 w-full rounded-lg bg-surface-container-highest px-4 text-lg font-semibold placeholder:font-normal placeholder:text-on-surface-variant focus:bg-secondary-container"
      />
    </label>
  );
}
