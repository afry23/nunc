import { useId, useState, type ReactNode } from "react";
import { BackupSection } from "../components/BackupSection";
import { InfoIcon } from "../components/icons";
import { NotificationHint } from "../components/NotificationHint";
import { SyncSection } from "../components/SyncSection";
import { TodoistSection } from "../components/TodoistSection";
import { Button, SectionLabel, Toggle } from "../components/ui";
import {
  LANGUAGES,
  t as translateNow,
  useT,
  type LanguageSetting,
} from "../i18n";
import type { Settings } from "../lib/api";
import { ratioAnchor, ratioBadge } from "../lib/format";
import { isMobile } from "../lib/platform";
import { soundLabel, SOUND_NAMES, playSound } from "../lib/sound";
import { useSettingsStore } from "../stores/settingsStore";
import { useUiStore } from "../stores/uiStore";

export function SettingsView() {
  const t = useT();
  const stored = useSettingsStore((s) => s.settings);
  if (!stored)
    return (
      <p className="p-10 text-on-surface-variant">{t("settings.loading")}</p>
    );
  // `key` setzt den Entwurf zurück, sobald andere Werte gespeichert sind (z. B. nach einem Restore)
  return (
    <>
      <SettingsForm key={JSON.stringify(stored)} stored={stored} />
      <div className="mx-auto flex max-w-4xl flex-col gap-10 px-6 pb-10">
        <SyncSection />
        {!isMobile() && <TodoistSection />}
        <BackupSection />
      </div>
    </>
  );
}

function SettingsForm({ stored }: { stored: Settings }) {
  const save = useSettingsStore((s) => s.save);
  const lastError = useSettingsStore((s) => s.lastError);
  const notify = useUiStore((s) => s.notify);
  const t = useT();
  const [draft, setDraft] = useState<Settings>(stored);
  const dirty = JSON.stringify(draft) !== JSON.stringify(stored);
  const set = <K extends keyof Settings>(key: K, value: Settings[K]): void =>
    setDraft((d) => ({ ...d, [key]: value }));

  const submit = async (): Promise<void> => {
    // nach dem Speichern gilt eventuell eine neue Sprache: den aktuellen Stand benutzen, nicht den alten `t`
    if (await save(draft)) notify(translateNow("settings.saved"));
  };

  return (
    <form
      aria-labelledby="settings-heading"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
      className="mx-auto flex max-w-4xl flex-col gap-10 px-6 pt-10 pb-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <SectionLabel>{t("settings.eyebrow")}</SectionLabel>
          <h1 id="settings-heading" className="headline-md">
            {t("settings.title")}
          </h1>
        </div>
        <div className="flex gap-2">
          <Button disabled={!dirty} onClick={() => setDraft(stored)}>
            {t("settings.discard")}
          </Button>
          <Button type="submit" variant="primary" disabled={!dirty}>
            {t("common.save")}
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

      <Section title={t("settings.pomodoro")}>
        <div className="grid gap-4 sm:grid-cols-3">
          <NumberCard
            label={t("settings.work")}
            unit={t("settings.unit.min")}
            min={1}
            max={180}
            value={draft.pomodoroMin}
            onChange={(v) => set("pomodoroMin", v)}
          />
          <NumberCard
            label={t("settings.shortBreak")}
            unit={t("settings.unit.min")}
            min={0}
            max={60}
            value={draft.shortBreakMin}
            onChange={(v) => set("shortBreakMin", v)}
          />
          <NumberCard
            label={t("settings.longBreak")}
            unit={t("settings.unit.min")}
            min={0}
            max={120}
            value={draft.longBreakMin}
            onChange={(v) => set("longBreakMin", v)}
          />
        </div>
        <div className="grid gap-4 sm:grid-cols-3">
          <NumberCard
            label={t("settings.longBreakAfter")}
            unit={t("settings.unit.pomodoros")}
            min={0}
            max={20}
            value={draft.longBreakAfter}
            onChange={(v) => set("longBreakAfter", v)}
          />
        </div>
        <div className="grid gap-4 rounded-2xl bg-surface-container-low p-5 sm:grid-cols-2">
          <Toggle
            label={t("settings.autoBreaks")}
            checked={draft.autoStartBreaks}
            onChange={(v) => set("autoStartBreaks", v)}
          />
          <Toggle
            label={t("settings.autoNext")}
            checked={draft.autoStartNextPomodoro}
            onChange={(v) => set("autoStartNextPomodoro", v)}
          />
        </div>
      </Section>

      <Section title={t("settings.flowtime")}>
        <div className="grid gap-6 lg:grid-cols-2">
          <div className="flex flex-col gap-4 text-on-surface-variant">
            <p>{t("settings.flowtime.intro")}</p>
            <p className="flex items-center gap-2 rounded-xl bg-surface-container-low p-4 text-orange-300">
              <InfoIcon />
              {t("settings.flowtime.dynamic")}
            </p>
          </div>
          <div className="flex flex-col gap-6 rounded-2xl bg-surface-container-low p-5">
            <RatioSlider
              value={draft.flowtimeRatio ?? 0.2}
              onChange={(v) => set("flowtimeRatio", v)}
            />
            <NumberCard
              label={t("settings.flowtime.minFlow")}
              unit={t("settings.unit.min")}
              hint={t("settings.flowtime.minFlowHint")}
              min={1}
              max={60}
              value={draft.flowtimeMinFlowStateMin}
              onChange={(v) => set("flowtimeMinFlowStateMin", v)}
            />
          </div>
        </div>
      </Section>

      <Section title={t("settings.sound")}>
        <NotificationHint withRequest />
        <div className="flex flex-col gap-4 rounded-2xl bg-surface-container-low p-5">
          <Toggle
            label={t("settings.notifications")}
            description={t("settings.notifications.hint")}
            checked={draft.notificationsEnabled}
            onChange={(v) => set("notificationsEnabled", v)}
          />
          <Toggle
            label={t("settings.beep")}
            description={t("settings.beep.hint")}
            checked={draft.soundEnabled}
            onChange={(v) => set("soundEnabled", v)}
          />
          <div className="grid gap-4 sm:grid-cols-2">
            <label className="flex flex-col gap-2">
              <span className="font-semibold">
                {t("settings.volume", { volume: draft.soundVolume })}
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
              label={t("settings.sound.pomodoro")}
              value={draft.soundEndPomodoro}
              volume={draft.soundVolume}
              disabled={!draft.soundEnabled}
              onChange={(v) => set("soundEndPomodoro", v)}
            />
            <SoundSelect
              label={t("settings.sound.break")}
              value={draft.soundEndBreak}
              volume={draft.soundVolume}
              disabled={!draft.soundEnabled}
              onChange={(v) => set("soundEndBreak", v)}
            />
          </div>
        </div>
      </Section>

      <Section title={t("language.section")}>
        <div className="flex flex-col gap-2 rounded-2xl bg-surface-container-low p-5">
          <label className="flex flex-col gap-2">
            <span className="font-semibold">{t("language.label")}</span>
            <select
              value={draft.language}
              onChange={(event) =>
                set("language", event.target.value as LanguageSetting)
              }
              className="min-h-11 rounded-lg bg-surface-container-highest px-3"
            >
              {LANGUAGES.map(({ value, native }) => (
                <option key={value} value={value}>
                  {native ?? t("language.system")}
                </option>
              ))}
            </select>
          </label>
          <span className="text-sm text-on-surface-variant">
            {t("language.hint")}
          </span>
        </div>
      </Section>

      {!isMobile() && (
        <Section title={t("settings.system")}>
          <div className="rounded-2xl bg-surface-container-low p-5">
            <Toggle
              label={t("settings.tray")}
              description={t("settings.tray.hint")}
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
  const t = useT();
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <label htmlFor="flowtime-ratio" className="font-semibold">
          {t("settings.ratio")}
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
  const t = useT();
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
              {soundLabel(name)}
            </option>
          ))}
        </select>
      </label>
      <Button
        variant="ghost"
        disabled={disabled}
        onClick={() => playSound(value, volume)}
      >
        {t("settings.preview")}
      </Button>
    </div>
  );
}
