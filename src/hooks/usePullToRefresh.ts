import { useEffect, useState } from "react";
import { pullDistance, shouldRefresh } from "../lib/gestures";

/**
 * Pull-to-refresh: Wer am Seitenanfang nach unten zieht und weit genug loslässt, lädt die Daten neu.
 * Liefert die aktuelle (gedämpfte) Zugstrecke für die Anzeige.
 */
export function usePullToRefresh(onRefresh: () => void): number {
  const [distance, setDistance] = useState(0);

  useEffect(() => {
    let startY: number | null = null;
    let current = 0;

    const reset = (): void => {
      startY = null;
      current = 0;
      setDistance(0);
    };
    const onStart = (event: TouchEvent): void => {
      // nur vom Seitenanfang aus und nicht aus Dialogen/Eingabefeldern
      const atTop = window.scrollY <= 0;
      const blocked = document.querySelector("[role=dialog]") !== null;
      startY = atTop && !blocked ? (event.touches[0]?.clientY ?? null) : null;
    };
    const onMove = (event: TouchEvent): void => {
      if (startY === null) return;
      current = pullDistance(startY, event.touches[0]?.clientY ?? startY);
      setDistance(current);
    };
    const onEnd = (): void => {
      if (startY !== null && shouldRefresh(current)) onRefresh();
      reset();
    };

    document.addEventListener("touchstart", onStart, { passive: true });
    document.addEventListener("touchmove", onMove, { passive: true });
    document.addEventListener("touchend", onEnd);
    document.addEventListener("touchcancel", reset);
    return () => {
      document.removeEventListener("touchstart", onStart);
      document.removeEventListener("touchmove", onMove);
      document.removeEventListener("touchend", onEnd);
      document.removeEventListener("touchcancel", reset);
    };
  }, [onRefresh]);

  return distance;
}
