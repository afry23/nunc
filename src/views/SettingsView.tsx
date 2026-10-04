import { useId, useState, type ReactNode } from "react";
import { InfoIcon } from "../components/icons";
import { NotificationHint } from "../components/NotificationHint";
import { Button, SectionLabel, Toggle } from "../components/ui";
import type { Settings } from "../lib/api";
import { ratioAnchor, ratioBadge } from "../lib/format";
import { isMobile } from "../lib/platform";
import { SOUND_LABELS, SOUND_NAMES, playSound } from "../lib/sound";
import { useSettingsStore } from "../stores/settingsStore";
import { useUiStore } from "../stores/uiStore";

export function SettingsView() {
  const stored = useSettingsStore((s) => s.settings);
  if (!stored)
    return <p className="p-10 text-on-surface-variant">Lade Einstellungen …</p>;
  // `key` setzt den Entwurf zurück, sobald andere Werte gespeichert sind (z. B. nach einem Restore)
  return <SettingsForm key={JSON.stringify(stored)} stored={stored} />;
}

function SettingsForm({ stored }: { stored: Settings }) {
  const save = useSettingsStore((s) => s.save);
  const lastError = useSettingsStore((s) => s.lastError);
  const notify = useUiStore((s) => s.notify);
  const [draft, setDraft] = useState<Settings>(stored);
  const dirty = JSON.stringify(draft) !== JSON.stringify(stored);
  const set = <K extends keyof Settings>(key: K, value: Settings[K]): void =>
    setDraft((d) => ({ ...d, [key]: value }));

  const submit = async (): Promise<void> => {
    if (await save(draft)) notify("Einstellungen gespeichert.");
  };

  return (
    <form
      aria-labelledby="settings-heading"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
      className="mx-auto flex max-w-4xl flex-col gap-10 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <SectionLabel>Konfiguration</SectionLabel>
          <h1 id="settings-heading" className="headline-md">
            Einstellungen
          </h1>
        </div>
        <div className="flex gap-2">
          <Button disabled={!dirty} onClick={() => setDraft(stored)}>
            Verwerfen
          </Button>
          <Button type="submit" variant="primary" disabled={!dirty}>
            Speichern
          </Button>
        </div>
      </header>

      {lastError && (
        <p
          role="alert"
          className="rounded-xl bg-surface-container-low p-4 text-red-300"
        >
          {lastError}
        </p>
      )}

      <Section title="Pomodoro">
        <div className="grid gap-4 sm:grid-cols-3">
          <NumberCard
            label="Arbeitszeit"
            unit="min"
            min={1}
            max={180}
            value={draft.pomodoroMin}
            onChange={(v) => set("pomodoroMin", v)}
          />
          <NumberCard
            label="Kurze Pause"
            unit="min"
            min={0}
            max={60}
            value={draft.shortBreakMin}
            onChange={(v) => set("shortBreakMin", v)}
          />
          <NumberCard
            label="Lange Pause"
            unit="min"
            min={0}
            max={120}
            value={draft.longBreakMin}
            onChange={(v) => set("longBreakMin", v)}
          />
        </div>
        <div className="grid gap-4 sm:grid-cols-3">
          <NumberCard
            label="Lange Pause nach"
            unit="Pomodoros"
            min={0}
            max={20}
            value={draft.longBreakAfter}
            onChange={(v) => set("longBreakAfter", v)}
          />
        </div>
        <div className="grid gap-4 rounded-2xl bg-surface-container-low p-5 sm:grid-cols-2">
          <Toggle
            label="Pausen automatisch starten"
            checked={draft.autoStartBreaks}
            onChange={(v) => set("autoStartBreaks", v)}
          />
          <Toggle
            label="Nächsten Pomodoro automatisch starten"
            checked={draft.autoStartNextPomodoro}
            onChange={(v) => set("autoStartNextPomodoro", v)}
          />
        </div>
      </Section>

      <Section title="Flowtime">
        <div className="grid gap-6 lg:grid-cols-2">
          <div className="flex flex-col gap-4 text-on-surface-variant">
            <p>
              Flowtime ist die flexible Alternative zum Pomodoro: Du arbeitest,
              solange der Fokus hält, und bekommst danach eine Pause passend zum
              Verhältnis.
            </p>
            <p className="flex items-center gap-2 rounded-xl bg-surface-container-low p-4 text-orange-300">
              <InfoIcon />
              Die Pausenlänge wird dynamisch aus der Arbeitszeit berechnet.
            </p>
          </div>
          <div className="flex flex-col gap-6 rounded-2xl bg-surface-container-low p-5">
            <RatioSlider
              value={draft.flowtimeRatio ?? 0.2}
              onChange={(v) => set("flowtimeRatio", v)}
            />
            <NumberCard
              label="Minimale Flow-Zeit"
              unit="min"
              hint="Davor wird keine Pause empfohlen."
              min={1}
              max={60}
              value={draft.flowtimeMinFlowStateMin}
              onChange={(v) => set("flowtimeMinFlowStateMin", v)}
            />
          </div>
        </div>
      </Section>

      <Section title="Benachrichtigungen und Ton">
        <NotificationHint withRequest />
        <div className="flex flex-col gap-4 rounded-2xl bg-surface-container-low p-5">
          <Toggle
            label="Benachrichtigungen"
            description="Hinweis, wenn der Timer endet"
            checked={draft.notificationsEnabled}
            onChange={(v) => set("notificationsEnabled", v)}
          />
          <Toggle
            label="Signalton"
            description="Ton beim Ende einer Session"
            checked={draft.soundEnabled}
            onChange={(v) => set("soundEnabled", v)}
          />
          <div className="grid gap-4 sm:grid-cols-2">
            <label className="flex flex-col gap-2">
              <span className="font-semibold">
                Lautstärke: {draft.soundVolume} %
              </span>
              <input
                type="range"
                min={0}
                max={100}
                step={5}
                value={draft.soundVolume}
                disabled={!draft.soundEnabled}
                onChange={(event) =>
                  set("soundVolume", Number(event.target.value))
                }
                className="h-11 accent-[var(--color-primary)]"
              />
            </label>
          </div>
          <div className="grid gap-4 sm:grid-cols-2">
            <SoundSelect
              label="Ton am Ende eines Pomodoros"
              value={draft.soundEndPomodoro}
              volume={draft.soundVolume}
              disabled={!draft.soundEnabled}
              onChange={(v) => set("soundEndPomodoro", v)}
            />
            <SoundSelect
              label="Ton am Ende einer Pause"
              value={draft.soundEndBreak}
              volume={draft.soundVolume}
              disabled={!draft.soundEnabled}
              onChange={(v) => set("soundEndBreak", v)}
            />
          </div>
        </div>
      </Section>

      {!isMobile() && (
        <Section title="System">
          <div className="rounded-2xl bg-surface-container-low p-5">
            <Toggle
              label="Symbol im Infobereich (Tray)"
              description="Das Fenster schließt in den Tray statt die App zu beenden"
              checked={draft.systemTrayEnabled}
              onChange={(v) => set("systemTrayEnabled", v)}
            />
          </div>
        </Section>
      )}
    </form>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  const id = useId();
  return (
    <section aria-labelledby={id} className="flex flex-col gap-4">
      <h2 id={id} className="font-display text-xl font-bold">
        {title}
      </h2>
      {children}
    </section>
  );
}

function NumberCard({
  label,
  unit,
  hint,
  min,
  max,
  value,
  onChange,
}: {
  label: string;
  unit: string;
  hint?: string;
  min: number;
  max: number;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="flex flex-col gap-2 rounded-2xl bg-surface-container-low p-5">
      <span className="label-md text-on-surface-variant">{label}</span>
      <span className="flex items-center gap-3">
        <input
          type="number"
          aria-label={label}
          inputMode="numeric"
          min={min}
          max={max}
          step={1}
          value={Number.isNaN(value) ? "" : value}
          onChange={(event) => onChange(event.target.valueAsNumber)}
          className="font-display h-12 w-24 rounded-sm bg-surface-container-highest px-3 text-2xl font-bold focus:bg-secondary-container"
        />
        <span className="text-on-surface-variant">{unit}</span>
      </span>
      {hint && <span className="text-sm text-on-surface-variant">{hint}</span>}
    </label>
  );
}

function RatioSlider({
  value,
  onChange,
}: {
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <label htmlFor="flowtime-ratio" className="font-semibold">
          Arbeit-zu-Pause-Verhältnis
        </label>
        <span
          data-testid="ratio-badge"
          className="rounded-sm bg-surface-container-highest px-2 py-1 font-semibold text-primary"
        >
          {ratioBadge(value)}
        </span>
      </div>
      <input
        id="flowtime-ratio"
        type="range"
        min={0.1}
        max={0.5}
        step={0.05}
        value={value}
        aria-valuetext={`${ratioBadge(value)} (${ratioAnchor(value)})`}
        onChange={(event) =>
          onChange(Math.round(Number(event.target.value) * 100) / 100)
        }
        className="h-11 accent-[var(--color-primary)]"
      />
      <div
        className="label-md flex justify-between text-on-surface-variant"
        aria-hidden="true"
      >
        <span>STRICT (1 : 0.1)</span>
        <span>BALANCED</span>
        <span>RELAXED (1 : 0.5)</span>
      </div>
    </div>
  );
}

function SoundSelect({
  label,
  value,
  volume,
  disabled,
  onChange,
}: {
  label: string;
  value: string;
  volume: number;
  disabled: boolean;
  onChange: (value: string) => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      <label className="flex flex-col gap-2">
        <span className="font-semibold">{label}</span>
        <select
          value={value}
          disabled={disabled}
          onChange={(event) => onChange(event.target.value)}
          className="min-h-11 rounded-lg bg-surface-container-highest px-3"
        >
          {SOUND_NAMES.map((name) => (
            <option key={name} value={name}>
              {SOUND_LABELS[name]}
            </option>
          ))}
        </select>
      </label>
      <Button
        variant="ghost"
        disabled={disabled}
        onClick={() => playSound(value, volume)}
      >
        Probehören
      </Button>
    </div>
  );
}
