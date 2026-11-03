import { useState } from "react";
import { useT } from "../i18n";
import { Button } from "./ui";

/** Tagesnotiz mit Speichern und Einstieg in den Tagesabschluss (ADR 0027). Beginnt mit der gespeicherten `note`. */
export function DayNote({
  note,
  onSave,
  onClose,
}: {
  note: string;
  onSave: (text: string) => void;
  onClose: () => void;
}) {
  const t = useT();
  const [draft, setDraft] = useState(note);

  return (
    <div className="flex flex-col gap-2 rounded-xl bg-surface-container-low p-4">
      <label className="flex flex-col gap-1">
        <span className="label-md text-on-surface-variant">
          {t("today.note")}
        </span>
        <textarea
          value={draft}
          maxLength={4000}
          rows={3}
          onChange={(event) => setDraft(event.target.value)}
          className="w-full min-w-0 rounded-lg bg-surface-container-high p-3"
        />
      </label>
      <p className="text-sm text-on-surface-variant">{t("today.noteHint")}</p>
      <div className="flex flex-wrap gap-2">
        <Button
          disabled={draft.trim() === note.trim()}
          onClick={() => onSave(draft)}
        >
          {t("today.noteSave")}
        </Button>
        <Button variant="primary" onClick={onClose}>
          {t("today.close")}
        </Button>
      </div>
    </div>
  );
}
