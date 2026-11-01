import { useEffect, useId, useState, type ReactNode } from "react";
import { AboutSection } from "../components/AboutSection";
import { ApiSection } from "../components/ApiSection";
import { BackupSection } from "../components/BackupSection";
import { InfoIcon } from "../components/icons";
import { NotificationHint } from "../components/NotificationHint";
import { SyncSection } from "../components/SyncSection";
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
    <div className="mx-auto grid max-w-6xl gap-x-12 px-4 sm:px-6 lg:grid-cols-[12rem_minmax(0,1fr)]">
      <SectionJumpList />
      <div className="flex min-w-0 max-w-3xl flex-col gap-12 pb-10">
        <SettingsForm key={JSON.stringify(stored)} stored={stored} />
        <SyncSection />
        <BackupSection />
        <AboutSection />
      </div>
    </div>
  );
}

type JumpGroup = { title: string; items: { id: string; label: string }[] };

/** Sprungliste (nur breit): springt zu einem Bereich und markiert den, der gerade im Bild ist. */
function SectionJumpList() {
  const t = useT();
  const mobile = isMobile();
  const groups: JumpGroup[] = [
    {
      title: t("settings.group.timer"),
      items: [
        { id: "settings-pomodoro", label: t("settings.pomodoro") },
        { id: "settings-flowtime", label: t("settings.flowtime") },
      ],
    },
    {
      title: t("settings.group.app"),
      items: [
        { id: "settings-sound", label: t("settings.sound") },
        { id: "settings-language", label: t("language.section") },
        ...(mobile ? [] : [{ id: "settings-api", label: t("api.title") }]),
        ...(mobile
          ? []
          : [{ id: "settings-system", label: t("settings.system") }]),
      ],
    },
    {
      title: t("settings.group.data"),
      items: [
        { id: "settings-sync", label: t("sync.title") },
        { id: "settings-backup", label: t("backup.title") },
        { id: "settings-about", label: t("about.title") },
      ],
    },
  ];
  const ids = groups.flatMap((g) => g.items.map((i) => i.id)).join(",");
  const [active, setActive] = useState("settings-pomodoro");

  useEffect(() => {
    if (typeof IntersectionObserver !== "function") return;
    const observer = new IntersectionObserver(
      (entries) => {
        const hit = entries.find((e) => e.isIntersecting);
        if (hit) setActive(hit.target.id);
      },
      // schmales Band knapp unter der Kopfleiste: der Bereich dort gilt als aktuell
      { rootMargin: "-96px 0px -70% 0px" },
    );
    for (const id of ids.split(",")) {
      const el = document.getElementById(id);
      if (el) observer.observe(el);
    }
    return () => observer.disconnect();
  }, [ids]);

  const jump = (id: string): void => {
    const reduce =
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    document
      .getElementById(id)
      ?.scrollIntoView?.({ behavior: reduce ? "auto" : "smooth" });
    setActive(id);
  };

  return (
    <aside
      role="group"
      aria-label={t("settings.jump")}
      className="sticky top-24 hidden max-h-[calc(100dvh-8rem)] flex-col gap-6 self-start overflow-y-auto pt-4 lg:flex"
    >
      {groups.map((group) => (
        <div key={group.title} className="flex flex-col gap-1">
          <SectionLabel>{group.title}</SectionLabel>
          {group.items.map((item) => (
            <button
              key={item.id}
              type="button"
              aria-current={active === item.id ? "location" : undefined}
              onClick={() => jump(item.id)}
              className={`min-h-11 rounded-lg px-3 text-left transition-colors ${
                active === item.id
                  ? "bg-surface-container-high font-semibold text-on-surface"
                  : "text-on-surface-variant hover:bg-surface-container-low hover:text-on-surface"
              }`}
            >
              {item.label}
            </button>
          ))}
        </div>
      ))}
    </aside>
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
      className="flex flex-col gap-12"
    >
      <header className="glass sticky top-0 z-20 -mx-4 flex items-center justify-between gap-3 px-4 py-3 sm:-mx-6 sm:px-6 lg:mx-0 lg:rounded-b-2xl">
        <div className="min-w-0">
          <div className="flex flex-wrap items-baseline gap-x-3">
            <SectionLabel>{t("settings.eyebrow")}</SectionLabel>
            <p role="status" className="label-md text-primary" hidden={!dirty}>
              {dirty ? t("settings.unsaved") : ""}
            </p>
          </div>
          <h1 id="settings-heading" className="headline-md truncate">
            {t("settings.title")}
          </h1>
        </div>
        <div className="flex shrink-0 gap-2">
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
          className="-mt-6 rounded-xl bg-surface-container-low p-4 text-red-300"
        >
          {lastError}
        </p>
      )}

      <Section id="settings-pomodoro" title={t("settings.pomodoro")}>
        <div className="grid grid-cols-2 gap-3 sm:gap-4 xl:grid-cols-4">
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
          <NumberCard
            label={t("settings.longBreakAfter")}
            unit={t("settings.unit.pomodoros")}
            min={0}
            max={20}
            value={draft.longBreakAfter}
            onChange={(v) => set("longBreakAfter", v)}
          />
        </div>
        <Card>
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
        </Card>
      </Section>

      <Section id="settings-flowtime" title={t("settings.flowtime")}>
        <p className="max-w-prose text-on-surface-variant">
          {t("settings.flowtime.intro")}
        </p>
        <p className="flex items-start gap-2 rounded-xl bg-surface-container-low p-4 text-orange-300">
          <InfoIcon />
          {t("settings.flowtime.dynamic")}
        </p>
        <Card>
          <RatioSlider
            value={draft.flowtimeRatio ?? 0.2}
            onChange={(v) => set("flowtimeRatio", v)}
          />
          <NumberCard
            flat
            label={t("settings.flowtime.minFlow")}
            unit={t("settings.unit.min")}
            hint={t("settings.flowtime.minFlowHint")}
            min={1}
            max={60}
            value={draft.flowtimeMinFlowStateMin}
            onChange={(v) => set("flowtimeMinFlowStateMin", v)}
          />
        </Card>
      </Section>

      <Section id="settings-sound" title={t("settings.sound")}>
        <NotificationHint withRequest />
        <Card>
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
          <Field label={t("settings.volume", { volume: draft.soundVolume })}>
            {({ id }) => (
              <input
                id={id}
                type="range"
                min={0}
                max={100}
                step={5}
                value={draft.soundVolume}
                disabled={!draft.soundEnabled}
                onChange={(event) =>
                  set("soundVolume", Number(event.target.value))
                }
                className="h-11 w-full accent-[var(--color-primary)]"
              />
            )}
          </Field>
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
        </Card>
      </Section>

      <Section id="settings-language" title={t("language.section")}>
        <Card>
          <Field label={t("language.label")} hint={t("language.hint")}>
            {({ id, describedBy }) => (
              <select
                id={id}
                aria-describedby={describedBy}
                value={draft.language}
                onChange={(event) =>
                  set("language", event.target.value as LanguageSetting)
                }
                className="min-h-11 w-full rounded-lg bg-surface-container-highest px-3"
              >
                {LANGUAGES.map(({ value, native }) => (
                  <option key={value} value={value}>
                    {native ?? t("language.system")}
                  </option>
                ))}
              </select>
            )}
          </Field>
        </Card>
      </Section>

      {!isMobile() && (
        <Section id="settings-api" title={t("api.title")}>
          <ApiSection
            enabled={draft.apiEnabled}
            port={draft.apiPort}
            storedEnabled={stored.apiEnabled}
            storedPort={stored.apiPort}
            onEnabled={(value) => set("apiEnabled", value)}
            onPort={(value) => set("apiPort", value)}
          />
        </Section>
      )}

      {!isMobile() && (
        <Section id="settings-system" title={t("settings.system")}>
          <Card>
            <Toggle
              label={t("settings.tray")}
              description={t("settings.tray.hint")}
              checked={draft.systemTrayEnabled}
              onChange={(v) => set("systemTrayEnabled", v)}
            />
          </Card>
        </Section>
      )}
    </form>
  );
}

function Section({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: ReactNode;
}) {
  const headingId = useId();
  return (
    <section
      id={id}
      aria-labelledby={headingId}
      className="flex scroll-mt-24 flex-col gap-4"
    >
      <h2 id={headingId} className="font-display text-xl font-bold">
        {title}
      </h2>
      {children}
    </section>
  );
}

/** Tonale Fläche für zusammengehörige Einstellungen; Abstand statt Trennlinien. */
function Card({ children }: { children: ReactNode }) {
  return (
    <div className="flex flex-col gap-5 rounded-2xl bg-surface-container-low p-4 sm:p-5">
      {children}
    </div>
  );
}

/** Beschriftetes Steuerelement: schmal untereinander, ab `sm` Text links und Steuerung rechts. */
function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: (props: { id: string; describedBy?: string }) => ReactNode;
}) {
  const id = useId();
  const hintId = useId();
  return (
    <div className="flex flex-col gap-2 sm:grid sm:grid-cols-[minmax(0,1fr)_minmax(0,16rem)] sm:items-center sm:gap-6">
      <div className="flex flex-col">
        <label htmlFor={id} className="font-semibold">
          {label}
        </label>
        {hint && (
          <span id={hintId} className="text-sm text-on-surface-variant">
            {hint}
          </span>
        )}
      </div>
      {children({ id, describedBy: hint ? hintId : undefined })}
    </div>
  );
}

function NumberCard({
  label,
  unit,
  hint,
  min,
  max,
  value,
  flat = false,
  onChange,
}: {
  label: string;
  unit: string;
  hint?: string;
  min: number;
  max: number;
  value: number;
  /** ohne eigene Fläche, wenn die Karte schon in einer `Card` steht */
  flat?: boolean;
  onChange: (value: number) => void;
}) {
  return (
    <label
      className={`flex min-w-0 flex-col gap-2 ${
        flat ? "" : "rounded-2xl bg-surface-container-low p-4 sm:p-5"
      }`}
    >
      <span className="text-sm font-semibold text-on-surface-variant">
        {label}
      </span>
      <span className="flex flex-wrap items-center gap-x-3 gap-y-1">
        <input
          type="number"
          aria-label={label}
          inputMode="numeric"
          min={min}
          max={max}
          step={1}
          value={Number.isNaN(value) ? "" : value}
          onChange={(event) => onChange(event.target.valueAsNumber)}
          className="font-display h-12 w-20 rounded-sm bg-surface-container-highest px-3 text-2xl font-bold focus:bg-secondary-container sm:w-24"
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
  const anchorLabel = (ratio: number): string =>
    t(
      `settings.ratio.${ratioAnchor(ratio).toLowerCase() as Lowercase<ReturnType<typeof ratioAnchor>>}`,
    );
  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between gap-3">
        <label htmlFor="flowtime-ratio" className="font-semibold">
          {t("settings.ratio")}
        </label>
        <span
          data-testid="ratio-badge"
          className="shrink-0 rounded-sm bg-surface-container-highest px-2 py-1 font-semibold text-primary"
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
        aria-valuetext={`${ratioBadge(value)} (${anchorLabel(value)})`}
        onChange={(event) =>
          onChange(Math.round(Number(event.target.value) * 100) / 100)
        }
        className="h-11 w-full accent-[var(--color-primary)]"
      />
      <div
        className="label-md flex flex-wrap justify-between gap-x-3 text-on-surface-variant"
        aria-hidden="true"
      >
        <span>{t("settings.ratio.strict")} (1 : 0.1)</span>
        <span>{t("settings.ratio.balanced")}</span>
        <span>{t("settings.ratio.relaxed")} (1 : 0.5)</span>
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
  const id = useId();
  return (
    <div className="flex flex-col gap-2 sm:grid sm:grid-cols-[minmax(0,1fr)_minmax(0,16rem)] sm:items-center sm:gap-6">
      <label htmlFor={id} className="font-semibold">
        {label}
      </label>
      <div className="flex gap-2">
        <select
          id={id}
          value={value}
          disabled={disabled}
          onChange={(event) => onChange(event.target.value)}
          className="min-h-11 min-w-0 flex-1 rounded-lg bg-surface-container-highest px-3"
        >
          {SOUND_NAMES.map((name) => (
            <option key={name} value={name}>
              {soundLabel(name)}
            </option>
          ))}
        </select>
        <Button
          variant="ghost"
          disabled={disabled}
          onClick={() => playSound(value, volume)}
        >
          {t("settings.preview")}
        </Button>
      </div>
    </div>
  );
}
