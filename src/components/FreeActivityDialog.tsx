import { useEffect, useState } from "react";
import { useT } from "../i18n";
import type { Labels, TimerMode } from "../lib/api";
import { useSettingsStore } from "../stores/settingsStore";
import { NO_LABELS, useTimerStore } from "../stores/timerStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { TagInput } from "./TagInput";
import { Segmented } from "./ui";

/**
 * „Freie Tätigkeit starten“: Arbeit ohne Aufgabe (Meeting, Support, Abstimmung) mit Tätigkeit, Projekten, Kontexten
 * und Modus (ADR 0023). Startet die Session und meldet das mit `onStarted`.
 */
export function FreeActivityDialog({
  onCancel,
  onStarted,
}: {
  onCancel: () => void;
  onStarted: () => void;
}) {
  const t = useT();
  const knownTags = useTimerStore((s) => s.knownTags);
  const activeMode = useSettingsStore((s) => s.settings?.activeMode);
  const [mode, setMode] = useState<TimerMode>(activeMode ?? "pomodoro");
  const [labels, setLabels] = useState<Labels>(NO_LABELS);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void useTimerStore.getState().loadKnownTags();
  }, []);

  const start = async (): Promise<void> => {
    setBusy(true);
    const timer = useTimerStore.getState();
    timer.selectTodo(null);
    timer.setLabels(labels);
    if (mode !== activeMode)
      await useSettingsStore.getState().patch({ activeMode: mode });
    await timer.start(mode);
    onStarted();
  };

  return (
    <ConfirmDialog
      title={t("activity.title")}
      confirmLabel={t("activity.confirm")}
      busy={busy}
      onCancel={onCancel}
      onConfirm={() => void start()}
    >
      <p className="text-sm">{t("activity.hint")}</p>
      <label className="flex flex-col gap-1">
        <span className="label-md">{t("activity.what")}</span>
        <input
          type="text"
          value={labels.note}
          autoComplete="off"
          onChange={(event) =>
            setLabels({ ...labels, note: event.target.value })
          }
          className="min-h-11 w-full rounded-lg bg-surface-container-highest px-3 text-on-surface focus:bg-secondary-container"
        />
      </label>
      <TagInput
        label={t("labels.projects")}
        addLabel={t("labels.addProject")}
        prefix="+"
        value={labels.projectTags}
        suggestions={knownTags.projects}
        onChange={(projectTags) => setLabels({ ...labels, projectTags })}
      />
      <TagInput
        label={t("labels.contexts")}
        addLabel={t("labels.addContext")}
        prefix="@"
        value={labels.contextTags}
        suggestions={knownTags.contexts}
        onChange={(contextTags) => setLabels({ ...labels, contextTags })}
      />
      <Segmented
        label={t("timer.mode.label")}
        value={mode}
        onChange={setMode}
        options={[
          { value: "pomodoro", label: t("timer.mode.pomodoro") },
          { value: "flowtime", label: t("timer.mode.flowtime") },
          { value: "stopwatch", label: t("timer.mode.stopwatch") },
        ]}
      />
    </ConfirmDialog>
  );
}
