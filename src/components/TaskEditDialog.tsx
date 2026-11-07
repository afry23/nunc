import { useEffect, useState } from "react";
import { useT } from "../i18n";
import type { Todo } from "../lib/api";
import {
  estimateFromInput,
  estimateOf,
  splitMinutes,
  withEstimate,
  withoutEstimate,
  type EstimateMode,
} from "../lib/estimate";
import { withoutTags, withTags } from "../lib/tags";
import { useTimerStore } from "../stores/timerStore";
import { useTodoStore } from "../stores/todoStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { TagInput } from "./TagInput";
import { TaskInfo } from "./TaskInfo";

const FIELD =
  "min-h-11 w-24 rounded-lg bg-surface-container-highest px-3 text-on-surface tabular-nums focus:bg-secondary-container";

/**
 * Aufgabe bearbeiten (modal): Text, Projekte und Kontexte, Priorität und Datumsangaben bleiben in der Zeile; dazu der
 * Zeitbedarf in Pomodoros oder Stunden und Minuten (ADR 0035). Beim Speichern entsteht daraus eine Zeile; ein einziger `update_todo`-Aufruf.
 */
export function TaskEditDialog({
  todo,
  onClose,
}: {
  todo: Todo;
  onClose: () => void;
}) {
  const t = useT();
  const initial = estimateOf(todo.description);
  const parts =
    initial?.unit === "minutes" ? splitMinutes(initial.value) : null;
  const knownTags = useTimerStore((state) => state.knownTags);
  useEffect(() => {
    void useTimerStore.getState().loadKnownTags();
  }, []);
  const [line, setLine] = useState(withoutTags(withoutEstimate(todo.rawText)));
  const [projectTags, setProjectTags] = useState(todo.projectTags);
  const [contextTags, setContextTags] = useState(todo.contextTags);
  const [mode, setMode] = useState<EstimateMode>(
    initial?.unit === "minutes" ? "time" : "pomodoros",
  );
  const [pomodoros, setPomodoros] = useState(
    initial?.unit === "pomodoros" ? String(initial.value) : "",
  );
  const [hours, setHours] = useState(
    parts && parts.hours > 0 ? String(parts.hours) : "",
  );
  const [minutes, setMinutes] = useState(
    parts && parts.minutes > 0 ? String(parts.minutes) : "",
  );
  const [busy, setBusy] = useState(false);

  const input = { pomodoros, hours, minutes };
  const estimate = estimateFromInput(mode, input);
  const filled =
    mode === "pomodoros" ? pomodoros !== "" : hours !== "" || minutes !== "";
  // gefüllte, aber ungültige Eingabe sperrt das Speichern; leere Felder heißen „keine Schätzung“
  const valid = line.trim() !== "" && (!filled || estimate !== null);

  const save = async (): Promise<void> => {
    setBusy(true);
    await useTodoStore
      .getState()
      .update(
        todo.id,
        withEstimate(
          withTags(line, projectTags, contextTags),
          filled ? estimate : null,
        ),
      );
    onClose();
  };

  return (
    <ConfirmDialog
      title={t("taskEdit.title")}
      confirmLabel={t("task.save")}
      busy={busy || !valid}
      wide
      onCancel={onClose}
      onConfirm={() => void save()}
    >
      <TaskInfo todo={todo} />
      <label className="flex flex-col gap-1">
        <span className="label-md">{t("task.raw")}</span>
        <textarea
          value={line}
          rows={3}
          onChange={(event) => setLine(event.target.value)}
          className="w-full min-w-0 rounded-lg bg-surface-container-highest p-3 font-mono text-sm text-on-surface focus:bg-secondary-container"
        />
      </label>
      <TagInput
        label={t("labels.projects")}
        addLabel={t("labels.addProject")}
        prefix="+"
        value={projectTags}
        suggestions={knownTags.projects}
        onChange={setProjectTags}
      />
      <TagInput
        label={t("labels.contexts")}
        addLabel={t("labels.addContext")}
        prefix="@"
        value={contextTags}
        suggestions={knownTags.contexts}
        onChange={setContextTags}
      />
      <fieldset className="flex flex-col gap-2">
        <legend className="label-md">{t("estimate.label")}</legend>
        <div
          role="radiogroup"
          aria-label={t("estimate.mode")}
          className="flex gap-4"
        >
          {(["pomodoros", "time"] as const).map((value) => (
            <label key={value} className="flex items-center gap-2">
              <input
                type="radio"
                name="estimate-mode"
                checked={mode === value}
                onChange={() => setMode(value)}
              />
              {t(`estimate.mode.${value}`)}
            </label>
          ))}
        </div>
        <div className="flex flex-wrap items-center gap-3">
          {mode === "pomodoros" ? (
            <input
              type="number"
              min={1}
              max={999}
              inputMode="numeric"
              aria-label={t("estimate.pomodoroInput")}
              value={pomodoros}
              onChange={(event) => setPomodoros(event.target.value)}
              className={FIELD}
            />
          ) : (
            <>
              <label className="flex items-center gap-2">
                <input
                  type="number"
                  min={0}
                  max={168}
                  inputMode="numeric"
                  aria-label={t("estimate.hours")}
                  value={hours}
                  onChange={(event) => setHours(event.target.value)}
                  className={FIELD}
                />
                h
              </label>
              <label className="flex items-center gap-2">
                <input
                  type="number"
                  min={0}
                  max={59}
                  inputMode="numeric"
                  aria-label={t("estimate.minutes")}
                  value={minutes}
                  onChange={(event) => setMinutes(event.target.value)}
                  className={FIELD}
                />
                min
              </label>
            </>
          )}
        </div>
        <p className="text-sm">{t("estimate.hint")}</p>
      </fieldset>
    </ConfirmDialog>
  );
}
