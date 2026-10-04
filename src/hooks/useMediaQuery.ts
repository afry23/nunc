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

/** Ab 768 px Breite gibt es die Seitenleiste, darunter die Tab-Leiste. */
export function useIsWide(): boolean {
  return useMediaQuery("(min-width: 768px)");
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
