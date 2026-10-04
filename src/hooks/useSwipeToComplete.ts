import { useRef, useState, type TouchEvent } from "react";
import { isSwipeRight, swipeOffset, type Point } from "../lib/gestures";

const pointOf = (event: TouchEvent): Point => {
  const touch = event.touches[0] ?? event.changedTouches[0];
  return { x: touch?.clientX ?? 0, y: touch?.clientY ?? 0 };
};

/**
 * Wisch nach rechts erledigt eine Aufgabe. Liefert Touch-Handler für das Zeilenelement und den
 * aktuellen Versatz für die Rückmeldung während des Wischens.
 */
export function useSwipeToComplete(onSwipe: () => void) {
  const start = useRef<Point | null>(null);
  const [offset, setOffset] = useState(0);

  return {
    offset,
    handlers: {
      onTouchStart: (event: TouchEvent) => {
        start.current = pointOf(event);
        setOffset(0);
      },
      onTouchMove: (event: TouchEvent) => {
        if (start.current)
          setOffset(swipeOffset(start.current, pointOf(event)));
      },
      onTouchEnd: (event: TouchEvent) => {
        const from = start.current;
        start.current = null;
        setOffset(0);
        if (from && isSwipeRight(from, pointOf(event))) onSwipe();
      },
      onTouchCancel: () => {
        start.current = null;
        setOffset(0);
      },
    },
  };
}
