import { describe, expect, it } from "vitest";
import {
  PULL_THRESHOLD,
  SWIPE_DISTANCE,
  SWIPE_TOLERANCE,
  isSwipeRight,
  pullDistance,
  shouldRefresh,
  swipeOffset,
} from "./gestures";

const at = (x: number, y = 0) => ({ x, y });

describe("isSwipeRight", () => {
  it("accepts a long horizontal move to the right", () =>
    expect(isSwipeRight(at(10), at(10 + SWIPE_DISTANCE))).toBe(true));
  it("rejects a short move", () =>
    expect(isSwipeRight(at(10), at(10 + SWIPE_DISTANCE - 1))).toBe(false));
  it("rejects a move to the left", () =>
    expect(isSwipeRight(at(200), at(200 - SWIPE_DISTANCE))).toBe(false));
  it("rejects a mostly vertical move", () =>
    expect(isSwipeRight(at(0), at(SWIPE_DISTANCE, SWIPE_TOLERANCE + 1))).toBe(
      false,
    ));
  it("rejects a diagonal move even within the tolerance", () =>
    expect(
      isSwipeRight(at(0), at(SWIPE_DISTANCE, SWIPE_DISTANCE / 2 + 1)),
    ).toBe(false));
  it("accepts the largest tolerated deviation", () =>
    expect(isSwipeRight(at(0, 100), at(SWIPE_DISTANCE, 100 - 30))).toBe(true));
});

describe("swipeOffset", () => {
  it("follows the finger to the right", () =>
    expect(swipeOffset(at(0), at(40))).toBe(40));
  it("is capped", () =>
    expect(swipeOffset(at(0), at(1000))).toBe(SWIPE_DISTANCE * 1.5));
  it("ignores moves to the left", () =>
    expect(swipeOffset(at(50), at(10))).toBe(0));
  it("ignores vertical scrolling", () =>
    expect(swipeOffset(at(0), at(40, SWIPE_TOLERANCE + 1))).toBe(0));
});

describe("pull to refresh", () => {
  it("dampens the finger movement", () =>
    expect(pullDistance(100, 200)).toBe(50));
  it("is zero when moving up", () => expect(pullDistance(200, 100)).toBe(0));
  it("triggers at the threshold", () => {
    expect(shouldRefresh(PULL_THRESHOLD)).toBe(true);
    expect(shouldRefresh(PULL_THRESHOLD - 1)).toBe(false);
  });
});
