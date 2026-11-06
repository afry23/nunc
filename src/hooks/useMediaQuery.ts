import { useSyncExternalStore } from "react";

/** Folgt einer Media-Query. Ohne `matchMedia` (z. B. Tests) gilt `fallback`. */
export function useMediaQuery(query: string, fallback = true): boolean {
  return useSyncExternalStore(
    (notify) => {
      if (typeof window.matchMedia !== "function") return () => undefined;
      const list = window.matchMedia(query);
      list.addEventListener("change", notify);
      return () => list.removeEventListener("change", notify);
    },
    () =>
      typeof window.matchMedia === "function"
        ? window.matchMedia(query).matches
        : fallback,
    () => fallback,
  );
}

/** Ab 768 px Breite nutzen die Ansichten ihre zweispaltigen Layouts (Navigation: siehe [`useNavMode`]). */
export function useIsWide(): boolean {
  return useMediaQuery("(min-width: 768px)");
}

/**
 * Navigation nach Breite: `tabs` (unter 600 px, Tab-Leiste), `rail` (600 bis 1023 px, schmale Leiste mit Icons) und
 * `sidebar` (ab 1024 px, einklappbare Seitenleiste).
 */
export function useNavMode(): "tabs" | "rail" | "sidebar" {
  const medium = useMediaQuery("(min-width: 600px)");
  const expanded = useMediaQuery("(min-width: 1024px)");
  if (expanded) return "sidebar";
  return medium ? "rail" : "tabs";
}

/** Innenmaße des Fensters, aktualisiert beim Ändern der Größe. */
export function useViewportSize(): { width: number; height: number } {
  const subscribe = (notify: () => void): (() => void) => {
    window.addEventListener("resize", notify);
    return () => window.removeEventListener("resize", notify);
  };
  const width = useSyncExternalStore(
    subscribe,
    () => window.innerWidth,
    () => 264,
  );
  const height = useSyncExternalStore(
    subscribe,
    () => window.innerHeight,
    () => 280,
  );
  return { width, height };
}
