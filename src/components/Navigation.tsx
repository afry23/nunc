import { Link, useRouter, useRouterState } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { useT, type MessageKey } from "../i18n";
import { useUiStore } from "../stores/uiStore";
import { isMobile } from "../lib/platform";
import {
  BarsIcon,
  CheckIcon,
  ClockIcon,
  GearIcon,
  HelpIcon,
  SidebarIcon,
  TimerIcon,
} from "./icons";

export const NAV_ITEMS: {
  to: "/" | "/todos" | "/times" | "/stats" | "/settings" | "/help";
  label: MessageKey;
  icon: ReactNode;
}[] = [
  { to: "/", label: "nav.timer", icon: <TimerIcon /> },
  { to: "/todos", label: "nav.todos", icon: <CheckIcon /> },
  { to: "/times", label: "nav.times", icon: <ClockIcon /> },
  { to: "/stats", label: "nav.stats", icon: <BarsIcon /> },
  { to: "/settings", label: "nav.settings", icon: <GearIcon /> },
  { to: "/help", label: "nav.help", icon: <HelpIcon /> },
];

/** Seitenleiste ab 768 px Breite; einklappbar auf reine Icons (Beschriftung bleibt für Screenreader und Tooltip). */
export function Sidebar() {
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggle = useUiStore((s) => s.toggleSidebar);
  const t = useT();
  const label = collapsed ? t("nav.expand") : t("nav.collapse");

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
    </nav>
  );
}

/**
 * Tab-Leiste unter 768 px Breite (Touch-Ziele ≥ 44 px, Safe-Area unten).
 *
 * Mobil gilt ein Zurück-Stapel mit dem Timer als Startseite: Ein Tab ersetzt den vorigen statt sich
 * aufzustapeln, und „Timer“ geht einen Schritt zurück. So führt die Zurück-Taste (Android) vom Tab zuerst
 * zum Timer und erst von dort aus der App, statt durch alle besuchten Tabs zu laufen.
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
      {NAV_ITEMS.map((item) => (
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
