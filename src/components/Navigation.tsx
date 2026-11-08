import {
  Link,
  useNavigate,
  useRouter,
  useRouterState,
} from "@tanstack/react-router";
import type { ReactNode } from "react";
import { useT, type MessageKey } from "../i18n";
import { useUiStore } from "../stores/uiStore";
import { isMobile } from "../lib/platform";
import { ActionMenu } from "./ActionMenu";
import {
  BarsIcon,
  CheckIcon,
  ClockIcon,
  DotsIcon,
  GearIcon,
  HelpIcon,
  PlusIcon,
  SidebarIcon,
  TimerIcon,
} from "./icons";

export const NAV_ITEMS: {
  to: "/" | "/tasks" | "/log" | "/review" | "/settings" | "/help";
  label: MessageKey;
  icon: ReactNode;
}[] = [
  { to: "/", label: "nav.today", icon: <TimerIcon /> },
  { to: "/tasks", label: "nav.tasks", icon: <CheckIcon /> },
  { to: "/log", label: "nav.log", icon: <ClockIcon /> },
  { to: "/review", label: "nav.review", icon: <BarsIcon /> },
  { to: "/settings", label: "nav.settings", icon: <GearIcon /> },
  { to: "/help", label: "nav.help", icon: <HelpIcon /> },
];

/** Die vier Hauptbereiche; Einstellungen und Hilfe stehen mobil im Menü der Kopfzeile. */
export const MAIN_ITEMS = NAV_ITEMS.slice(0, 4);

/** Seitenleiste ab 600 px Breite (bis 1023 px als Leiste); einklappbar auf reine Icons (Beschriftung bleibt für Screenreader und Tooltip). */
export function Sidebar({ rail = false }: { rail?: boolean }) {
  const stored = useUiStore((s) => s.sidebarCollapsed);
  const toggle = useUiStore((s) => s.toggleSidebar);
  // als Leiste (600 bis 1023 px) ist sie immer eingeklappt und hat keinen Umschalter
  const collapsed = rail || stored;
  const t = useT();
  const label = collapsed ? t("nav.expand") : t("nav.collapse");
  const openNewTask = useUiStore((s) => s.openNewTask);

  return (
    <nav
      aria-label={t("nav.main")}
      data-testid="sidebar"
      data-collapsed={collapsed}
      className={`sticky top-0 flex h-dvh shrink-0 flex-col gap-1 self-start overflow-y-auto bg-surface-container-lowest transition-[width] duration-200 ${
        collapsed ? "w-16 p-2" : "w-64 p-4"
      }`}
    >
      {!collapsed && (
        <p className="font-display mb-8 px-3 pt-2 text-2xl font-bold">nunc</p>
      )}
      {collapsed && <div className="h-6" />}
      <button
        type="button"
        data-testid="sidebar-new-task"
        onClick={openNewTask}
        title={collapsed ? t("todos.new") : undefined}
        className={`mb-2 flex min-h-12 items-center rounded-lg btn-primary font-semibold ${
          collapsed ? "justify-center" : "gap-3 px-3"
        }`}
      >
        <PlusIcon />
        <span className={collapsed ? "sr-only" : undefined}>
          {t("todos.new")}
        </span>
      </button>
      {NAV_ITEMS.map((item) => (
        <Link
          key={item.to}
          to={item.to}
          title={collapsed ? t(item.label) : undefined}
          activeOptions={{ exact: item.to === "/" }}
          activeProps={{
            "aria-current": "page",
            className: "bg-secondary-container text-on-surface",
          }}
          inactiveProps={{
            className:
              "text-on-surface-variant hover:bg-surface-container-high",
          }}
          className={`flex min-h-12 items-center rounded-lg font-semibold transition-colors ${
            collapsed ? "justify-center" : "gap-3 px-3"
          }`}
        >
          {item.icon}
          <span className={collapsed ? "sr-only" : undefined}>
            {t(item.label)}
          </span>
        </Link>
      ))}
      {!rail && (
        <button
          type="button"
          onClick={toggle}
          aria-label={label}
          aria-expanded={!collapsed}
          title={label}
          className={`mt-auto flex min-h-12 items-center rounded-lg text-on-surface-variant transition-colors hover:bg-surface-container-high hover:text-on-surface ${
            collapsed ? "justify-center" : "gap-3 px-3 font-semibold"
          }`}
        >
          <SidebarIcon />
          {!collapsed && <span>{t("nav.collapseShort")}</span>}
        </button>
      )}
    </nav>
  );
}

/**
 * Tab-Leiste unter 600 px Breite (Touch-Ziele ≥ 44 px, Safe-Area unten).
 *
 * Mobil gilt ein Zurück-Stapel mit „Heute“ als Startseite: Ein Tab ersetzt den vorigen statt sich
 * aufzustapeln, und „Heute“ geht einen Schritt zurück. So führt die Zurück-Taste (Android) vom Tab zuerst
 * zu „Heute“ und erst von dort aus der App, statt durch alle besuchten Tabs zu laufen.
 */
export function BottomTabs() {
  const t = useT();
  const router = useRouter();
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const onTab = pathname !== "/";

  return (
    <nav
      aria-label={t("nav.main")}
      data-testid="tabbar"
      className="safe-bottom fixed inset-x-0 bottom-0 z-30 flex bg-surface-container-lowest"
    >
      {MAIN_ITEMS.map((item) => (
        <Link
          key={item.to}
          to={item.to}
          replace={isMobile() && onTab && item.to !== "/"}
          onClick={(event) => {
            if (
              isMobile() &&
              onTab &&
              item.to === "/" &&
              router.history.canGoBack()
            ) {
              event.preventDefault();
              router.history.back();
            }
          }}
          activeOptions={{ exact: item.to === "/" }}
          activeProps={{ "aria-current": "page", className: "text-primary" }}
          inactiveProps={{ className: "text-on-surface-variant" }}
          className="flex min-h-14 min-w-0 flex-1 flex-col items-center justify-center gap-1 text-xs font-semibold"
        >
          {item.icon}
          {t(item.label)}
        </Link>
      ))}
    </nav>
  );
}

/** Kopfzeile unter 600 px: Menü mit den selteneren Zielen (Einstellungen, Hilfe). */
export function ShellMenu() {
  const t = useT();
  const navigate = useNavigate();
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  // wie bei den Tabs: außerhalb von „Heute“ ersetzt das Ziel den Eintrag, der Zurück-Stapel wächst nicht
  const replace = isMobile() && pathname !== "/";
  return (
    <div className="flex justify-end px-2 pt-1">
      <ActionMenu
        label={t("nav.menu")}
        icon={<DotsIcon />}
        actions={[
          {
            label: t("nav.settings"),
            onSelect: () => void navigate({ to: "/settings", replace }),
          },
          {
            label: t("nav.help"),
            onSelect: () => void navigate({ to: "/help", replace }),
          },
        ]}
      />
    </div>
  );
}
