import { SectionLabel } from "../components/ui";
import { useT, type MessageKey } from "../i18n";
import { isMobile } from "../lib/platform";
import { SHORTCUTS } from "../lib/shortcuts";

const SECTIONS: {
  id: string;
  title: MessageKey;
  body: MessageKey;
  desktopOnly?: boolean;
}[] = [
  { id: "timer", title: "help.timer.title", body: "help.timer.body" },
  { id: "todos", title: "help.todos.title", body: "help.todos.body" },
  { id: "stats", title: "help.stats.title", body: "help.stats.body" },
  { id: "sync", title: "help.sync.title", body: "help.sync.body" },
  { id: "backup", title: "help.backup.title", body: "help.backup.body" },
  {
    id: "api",
    title: "help.api.title",
    body: "help.api.body",
    desktopOnly: true,
  },
];

export function HelpView() {
  const t = useT();
  const mobile = isMobile();

  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-10 px-6 pt-10 pb-10">
      <header>
        <SectionLabel>{t("help.eyebrow")}</SectionLabel>
        <h1 className="headline-md">{t("help.title")}</h1>
      </header>

      {SECTIONS.filter((s) => !(s.desktopOnly && mobile)).map((s) => (
        <section
          key={s.id}
          aria-labelledby={`help-${s.id}`}
          className="flex flex-col gap-3"
        >
          <h2 id={`help-${s.id}`} className="text-xl font-semibold">
            {t(s.title)}
          </h2>
          <p className="rounded-xl bg-surface-container-low p-4 text-on-surface-variant">
            {t(s.body)}
          </p>
        </section>
      ))}

      {!mobile && (
        <section
          aria-labelledby="help-shortcuts"
          className="flex flex-col gap-3"
        >
          <h2 id="help-shortcuts" className="text-xl font-semibold">
            {t("help.shortcuts.title")}
          </h2>
          <p className="text-on-surface-variant">{t("help.shortcuts.hint")}</p>
          <ul className="flex flex-col gap-2 rounded-xl bg-surface-container-low p-4">
            {SHORTCUTS.map((s) => (
              <li key={s.keys} className="flex items-center gap-4">
                <kbd className="min-w-12 rounded-md bg-surface-container-high px-2 py-1 text-center font-mono text-sm">
                  {s.keys}
                </kbd>
                <span>{t(s.label)}</span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
