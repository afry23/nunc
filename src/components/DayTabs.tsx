import { Link } from "@tanstack/react-router";
import { useT } from "../i18n";

const TAB =
  "inline-flex min-h-11 items-center rounded-lg px-6 font-semibold transition-colors";

/**
 * Umschalter oben in „Heute“ und „Timer“: die Tagesübersicht (`/`) und die Timer-Ansicht (`/focus`). Es sind zwei
 * Routen, damit Tray, Mini-Fenster und Aktiv-Leiste weiter direkt zum Timer führen.
 */
export function DayTabs({ current }: { current: "today" | "timer" }) {
  const t = useT();
  const tabs = [
    { id: "today", to: "/", label: t("today.tab.today") },
    { id: "timer", to: "/focus", label: t("today.tab.timer") },
  ] as const;
  return (
    <nav
      aria-label={t("today.tabs")}
      className="mx-auto inline-flex gap-1 rounded-xl bg-surface-container-lowest p-1"
    >
      {tabs.map((tab) => (
        <Link
          key={tab.id}
          to={tab.to}
          aria-current={tab.id === current ? "page" : undefined}
          className={`${TAB} ${
            tab.id === current
              ? "bg-surface-container-high text-on-surface"
              : "text-on-surface-variant hover:text-on-surface"
          }`}
        >
          {tab.label}
        </Link>
      ))}
    </nav>
  );
}
