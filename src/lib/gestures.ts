/** Reine Geometrie der Touch-Gesten (Swipe zum Erledigen, Pull-to-refresh). */

export type Point = { x: number; y: number };

/** Mindestweg nach rechts, ab dem ein Wisch eine Aufgabe erledigt. */
export const SWIPE_DISTANCE = 72;
/** Höchstens so viel vertikale Abweichung (px), sonst ist es ein Scrollen. */
export const SWIPE_TOLERANCE = 32;
/** Ab dieser (gedämpften) Zugstrecke löst Pull-to-refresh aus. */
export const PULL_THRESHOLD = 64;
/** Dämpfung: der Finger bewegt sich doppelt so weit wie die Anzeige. */
const PULL_RESISTANCE = 0.5;

/** Wisch nach rechts, überwiegend waagerecht und weit genug. */
export function isSwipeRight(start: Point, end: Point): boolean {
  const dx = end.x - start.x;
  const dy = Math.abs(end.y - start.y);
  return dx >= SWIPE_DISTANCE && dy <= SWIPE_TOLERANCE && dx > dy * 2;
}

/** Waagerechter Versatz während des Wischens: nur nach rechts, mit Obergrenze. */
export function swipeOffset(start: Point, current: Point): number {
  const dx = current.x - start.x;
  const dy = Math.abs(current.y - start.y);
  if (dy > SWIPE_TOLERANCE || dx <= 0) return 0;
  return Math.min(dx, SWIPE_DISTANCE * 1.5);
}

/** Gedämpfte Zugstrecke von Pull-to-refresh; 0 solange die Seite nicht oben steht. */
export function pullDistance(startY: number, currentY: number): number {
  return Math.max(0, (currentY - startY) * PULL_RESISTANCE);
}

export function shouldRefresh(distance: number): boolean {
  return distance >= PULL_THRESHOLD;
}
