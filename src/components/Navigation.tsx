import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { BarsIcon, CheckIcon, GearIcon, TimerIcon } from "./icons";

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

/** Seitenleiste ab 768 px Breite. */
export function Sidebar() {
  return (
    <nav
      aria-label="Hauptnavigation"
      data-testid="sidebar"
      className="flex w-64 shrink-0 flex-col gap-1 bg-surface-container-lowest p-4"
    >
      <p className="font-display mb-8 px-3 pt-2 text-2xl font-bold">nunc</p>
      {NAV_ITEMS.map((item) => (
        <Link
          key={item.to}
          to={item.to}
          activeOptions={{ exact: item.to === "/" }}
          activeProps={{
            "aria-current": "page",
            className: "bg-secondary-container text-on-surface",
          }}
          inactiveProps={{
            className:
              "text-on-surface-variant hover:bg-surface-container-high",
          }}
          className="flex min-h-12 items-center gap-3 rounded-lg px-3 font-semibold transition-colors"
        >
          {item.icon}
          {item.label}
        </Link>
      ))}
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
