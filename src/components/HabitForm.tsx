import { useState } from "react";
import { useT, type MessageKey } from "../i18n";
import {
  EMPTY_DRAFT,
  isDraftValid,
  toggleDay,
  type HabitDraft,
} from "../lib/habits";
import { Button, Chip } from "./ui";

const FIELD =
  "h-11 rounded-sm bg-surface-container-highest px-3 focus:bg-secondary-container";

/** Formular zum Anlegen oder Ändern einer Gewohnheit. `onSave` meldet, ob das Speichern geklappt hat. */
export function HabitForm({
  initial = EMPTY_DRAFT,
  onSave,
  onCancel,
}: {
  initial?: HabitDraft;
  onSave: (draft: HabitDraft) => Promise<boolean>;
  onCancel: () => void;
}) {
  const t = useT();
  const [draft, setDraft] = useState(initial);
  const [busy, setBusy] = useState(false);
  const patch = (change: Partial<HabitDraft>): void =>
    setDraft((current) => ({ ...current, ...change }));

  const submit = async (): Promise<void> => {
    setBusy(true);
    const saved = await onSave(draft);
    setBusy(false);
    if (saved) onCancel();
  };

  return (
    <form
      aria-label={t("habits.form.title")}
      onSubmit={(event) => {
        event.preventDefault();
        if (isDraftValid(draft)) void submit();
      }}
      className="flex flex-col gap-4 rounded-xl bg-surface-container-low p-4"
    >
      <label className="flex flex-col gap-1">
        <span className="text-sm text-on-surface-variant">
          {t("habits.form.name")}
        </span>
        <input
          type="text"
          value={draft.name}
          maxLength={100}
          onChange={(event) => patch({ name: event.target.value })}
          className={FIELD}
        />
      </label>

      <label className="flex flex-col gap-1">
        <span className="text-sm text-on-surface-variant">
          {t("habits.form.schedule")}
        </span>
        <select
          value={draft.schedule}
          onChange={(event) =>
            patch({ schedule: event.target.value as HabitDraft["schedule"] })
          }
          className={FIELD}
        >
          <option value="daily">{t("habits.form.schedule.daily")}</option>
          <option value="days">{t("habits.form.schedule.days")}</option>
          <option value="weekly">{t("habits.form.schedule.weekly")}</option>
        </select>
      </label>

      {draft.schedule === "days" && (
        <div
          role="group"
          aria-label={t("habits.form.days")}
          className="flex flex-wrap gap-2"
        >
          {[1, 2, 3, 4, 5, 6, 7].map((day) => (
            <Chip
              key={day}
              selected={draft.days.includes(day)}
              onClick={() => patch({ days: toggleDay(draft.days, day) })}
            >
              {t(`weekday.short.${day % 7}` as MessageKey)}
            </Chip>
          ))}
        </div>
      )}

      {draft.schedule === "weekly" && (
        <label className="flex flex-col gap-1">
          <span className="text-sm text-on-surface-variant">
            {t("habits.form.perWeek")}
          </span>
          <input
            type="number"
            inputMode="numeric"
            min={1}
            max={7}
            value={draft.perWeek}
            onChange={(event) =>
              patch({ perWeek: event.target.valueAsNumber || 0 })
            }
            className={`${FIELD} w-24 tabular-nums`}
          />
        </label>
      )}

      <label className="flex flex-col gap-1">
        <span className="text-sm text-on-surface-variant">
          {t("habits.form.goal")}
        </span>
        <select
          value={draft.goal}
          onChange={(event) =>
            patch({ goal: event.target.value as HabitDraft["goal"] })
          }
          className={FIELD}
        >
          <option value="check">{t("habits.form.goal.check")}</option>
          <option value="minutes">{t("habits.form.goal.minutes")}</option>
        </select>
      </label>

      {draft.goal === "minutes" && (
        <div className="flex flex-wrap gap-4">
          <label className="flex flex-col gap-1">
            <span className="text-sm text-on-surface-variant">
              {t("habits.form.minutes")}
            </span>
            <input
              type="number"
              inputMode="numeric"
              min={1}
              max={1440}
              value={draft.minutes}
              onChange={(event) =>
                patch({ minutes: event.target.valueAsNumber || 0 })
              }
              className={`${FIELD} w-24 tabular-nums`}
            />
          </label>
          <label className="flex min-w-0 flex-1 flex-col gap-1">
            <span className="text-sm text-on-surface-variant">
              {t("habits.form.tag")}
            </span>
            <input
              type="text"
              value={draft.tag}
              onChange={(event) => patch({ tag: event.target.value })}
              className={FIELD}
            />
          </label>
        </div>
      )}

      <div className="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
        <Button variant="ghost" onClick={onCancel}>
          {t("common.cancel")}
        </Button>
        <Button
          type="submit"
          variant="primary"
          disabled={busy || !isDraftValid(draft)}
        >
          {t("habits.form.save")}
        </Button>
      </div>
    </form>
  );
}
