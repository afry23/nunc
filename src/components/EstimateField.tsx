import { useState } from "react";
import { useT } from "../i18n";
import {
  estimateFromInput,
  splitMinutes,
  type EstimateMode,
} from "../lib/estimate";
import type { Estimate } from "../lib/api";
import { usePlanStore } from "../stores/planStore";
import { Button } from "./ui";

const inputClass =
  "w-20 rounded-lg bg-surface-container-highest p-2 text-on-surface tabular-nums focus:bg-secondary-container";

/** Zeitbedarf einer Aufgabe: in Pomodoros oder in Stunden und Minuten (ADR 0035). */
export function EstimateField({
  id,
  estimate,
}: {
  id: number;
  estimate: Estimate | null;
}) {
  const t = useT();
  const initial =
    estimate?.unit === "minutes" ? splitMinutes(estimate.value) : null;
  const [mode, setMode] = useState<EstimateMode>(
    estimate?.unit === "minutes" ? "time" : "pomodoros",
  );
  const [pomodoros, setPomodoros] = useState(
    estimate?.unit === "pomodoros" ? String(estimate.value) : "",
  );
  const [hours, setHours] = useState(
    initial && initial.hours > 0 ? String(initial.hours) : "",
  );
  const [minutes, setMinutes] = useState(
    initial && initial.minutes > 0 ? String(initial.minutes) : "",
  );

  const next = estimateFromInput(mode, { pomodoros, hours, minutes });
  const changed =
    next !== null &&
    (estimate === null ||
      next.unit !== estimate.unit ||
      next.value !== estimate.value);

  return (
    <form
      aria-label={t("estimate.label")}
      onSubmit={(event) => {
        event.preventDefault();
        if (next && changed) void usePlanStore.getState().setEffort(id, next);
      }}
      className="flex flex-col gap-2"
    >
      <h3 className="label-md text-on-surface-variant">
        {t("estimate.label")}
      </h3>
      <div
        role="radiogroup"
        aria-label={t("estimate.mode")}
        className="flex gap-4"
      >
        {(["pomodoros", "time"] as const).map((value) => (
          <label key={value} className="flex items-center gap-2">
            <input
              type="radio"
              name={`estimate-mode-${id}`}
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
            className={inputClass}
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
                className={inputClass}
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
                className={inputClass}
              />
              min
            </label>
          </>
        )}
        <Button type="submit" variant="primary" disabled={!changed}>
          {t("estimate.save")}
        </Button>
        {estimate !== null && (
          <Button
            type="button"
            onClick={() => void usePlanStore.getState().setEffort(id, null)}
          >
            {t("estimate.clear")}
          </Button>
        )}
      </div>
    </form>
  );
}
