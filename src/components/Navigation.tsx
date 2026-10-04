import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { useUiStore } from "../stores/uiStore";
import { BarsIcon, CheckIcon, GearIcon, SidebarIcon, TimerIcon } from "./icons";

export const NAV_ITEMS: {
  to: "/" | "/todos" | "/stats" | "/settings";
  label: string;
  icon: ReactNode;
}[] = [
  { to: "/", label: "Timer", icon: <TimerIcon /> },
  { to: "/todos", label: "Aufgaben", icon: <CheckIcon /> },
  { to: "/stats", label: "Statistik", icon: <BarsIcon /> },
  { to: "/settings", label: "Einstellungen", icon: <GearIcon /> },
];

/** Seitenleiste ab 768 px Breite; einklappbar auf reine Icons (Beschriftung bleibt für Screenreader und Tooltip). */
export function Sidebar() {
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggle = useUiStore((s) => s.toggleSidebar);
  const label = collapsed
    ? "Seitenleiste ausklappen"
    : "Seitenleiste einklappen";

  return (
    <nav
      aria-label="Hauptnavigation"
      data-testid="sidebar"
      data-collapsed={collapsed}
      className={`flex shrink-0 flex-col gap-1 bg-surface-container-lowest transition-[width] duration-200 ${
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
          title={collapsed ? item.label : undefined}
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
            {item.label}
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
        {!collapsed && <span>Einklappen</span>}
      </button>
    </nav>
  );
}

/** Tab-Leiste unter 768 px Breite (Touch-Ziele ≥ 44 px, Safe-Area unten). */
export function BottomTabs() {
  return (
    <nav
      aria-label="Hauptnavigation"
      data-testid="tabbar"
      className="safe-bottom fixed inset-x-0 bottom-0 z-30 flex bg-surface-container-lowest"
    >
      {NAV_ITEMS.map((item) => (
        <Link
          key={item.to}
          to={item.to}
          activeOptions={{ exact: item.to === "/" }}
          activeProps={{ "aria-current": "page", className: "text-primary" }}
          inactiveProps={{ className: "text-on-surface-variant" }}
          className="flex min-h-14 flex-1 flex-col items-center justify-center gap-1 text-xs font-semibold"
        >
          {item.icon}
          {item.label}
        </Link>
      ))}
    </nav>
  );
}
