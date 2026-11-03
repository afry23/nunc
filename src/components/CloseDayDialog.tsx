import { useState } from "react";
import { useT } from "../i18n";
import type { CloseAction, DayPlan } from "../lib/api";
import { closeSummary } from "../lib/dayPlan";
import { ConfirmDialog } from "./ConfirmDialog";
import { Segmented } from "./ui";

/** Tagesabschluss (ADR 0027): eingeplante Einträge verschieben oder abnehmen, dazu eine Notiz zum Tag. */
export function CloseDayDialog({
  plan,
  note,
  onClose,
  onCancel,
}: {
  plan: DayPlan;
  /** Gespeicherte Tagesnotiz, mit der das Feld beginnt. */
  note: string;
  onClose: (action: CloseAction, note: string | null) => Promise<void>;
  onCancel: () => void;
}) {
  const t = useT();
  const { planned, fixed } = closeSummary(plan);
  const [action, setAction] = useState<CloseAction>("postpone");
  const [text, setText] = useState(note);
  const [busy, setBusy] = useState(false);

  return (
    <ConfirmDialog
      title={t("today.close.title")}
      confirmLabel={t("today.close.confirm")}
      busy={busy}
      onCancel={onCancel}
      onConfirm={() => {
        setBusy(true);
        // eine unveränderte Notiz wird nicht neu geschrieben
        const changed = text.trim() !== note.trim();
        void onClose(action, changed ? text : null).finally(() =>
          setBusy(false),
        );
      }}
    >
      <p>{t("today.close.summary", { planned, fixed })}</p>
      {planned > 0 && (
        <Segmented<CloseAction>
          label={t("today.close.action")}
          value={action}
          onChange={setAction}
          options={[
            { value: "postpone", label: t("today.close.postpone") },
            { value: "unplan", label: t("today.close.unplan") },
            { value: "keep", label: t("today.close.keep") },
          ]}
        />
      )}
      {fixed > 0 && <p className="text-sm">{t("today.close.fixedHint")}</p>}
      <label className="flex flex-col gap-1">
        <span className="label-md">{t("today.close.note")}</span>
        <textarea
          value={text}
          maxLength={4000}
          rows={3}
          onChange={(event) => setText(event.target.value)}
          className="w-full min-w-0 rounded-lg bg-surface-container-high p-3 text-on-surface"
        />
      </label>
    </ConfirmDialog>
  );
}
