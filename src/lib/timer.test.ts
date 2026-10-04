import { describe, expect, it } from "vitest";
import type { TimerSnapshot } from "../bindings";
import {
  miniRingSize,
  ringDash,
  ringProgress,
  sessionColor,
  sessionLabel,
  spokenRemaining,
} from "./timer";

const snap = (over: Partial<TimerSnapshot>): TimerSnapshot => ({
  state: "running",
  mode: "pomodoro",
  sessionType: "work",
  remaining: 1500,
  total: 1500,
  todoId: null,
  ...over,
});

describe("sessionColor and label", () => {
  it("depends on mode and session type", () => {
    expect(sessionColor("pomodoro", "work")).toBe("var(--color-pomodoro)");
    expect(sessionColor("flowtime", "work")).toBe("var(--color-flowtime)");
    expect(sessionColor("pomodoro", "short_break")).toBe(
      "var(--color-shortbrk)",
    );
    expect(sessionColor("flowtime", "long_break")).toBe("var(--color-longbrk)");
    expect(sessionLabel("pomodoro", "work")).toBe("Fokus");
    expect(sessionLabel("flowtime", "work")).toBe("Flowtime");
    expect(sessionLabel("pomodoro", "short_break")).toBe("Kurze Pause");
    expect(sessionLabel("pomodoro", "long_break")).toBe("Lange Pause");
  });
});

describe("ringProgress", () => {
  it("is zero when idle", () =>
    expect(ringProgress(snap({ state: "idle" }))).toBe(0));
  it("is the elapsed share of a countdown", () => {
    expect(ringProgress(snap({ remaining: 1500 }))).toBe(0);
    expect(ringProgress(snap({ remaining: 750 }))).toBe(0.5);
    expect(ringProgress(snap({ remaining: 0 }))).toBe(1);
  });
  it("is clamped", () =>
    expect(ringProgress(snap({ remaining: 2000 }))).toBe(0));
  it("wraps hourly for a stopwatch", () => {
    expect(ringProgress(snap({ total: 0, remaining: 900 }))).toBe(0.25);
    expect(ringProgress(snap({ total: 0, remaining: 4500 }))).toBe(0.25);
  });
});

describe("ringDash", () => {
  it("splits the circumference", () => {
    const circumference = 2 * Math.PI * 100;
    const [visible, rest] = ringDash(100, 0.25).split(" ").map(Number);
    expect(visible).toBeCloseTo(circumference / 4, 1);
    expect(visible + rest).toBeCloseTo(circumference, 1);
    expect(ringDash(100, 2)).toBe(ringDash(100, 1));
    expect(ringDash(100, -1).startsWith("0.00")).toBe(true);
  });
});

describe("spokenRemaining", () => {
  it("announces per minute", () => {
    expect(spokenRemaining(snap({ state: "idle" }))).toBe("Timer bereit");
    expect(spokenRemaining(snap({ remaining: 1499 }))).toBe(
      "Fokus: noch 24 Minuten",
    );
    expect(spokenRemaining(snap({ remaining: 60 }))).toBe(
      "Fokus: noch 1 Minute",
    );
    expect(spokenRemaining(snap({ state: "paused", remaining: 600 }))).toBe(
      "Fokus: noch 10 Minuten, pausiert",
    );
    expect(
      spokenRemaining(snap({ mode: "flowtime", total: 0, remaining: 125 })),
    ).toBe("Flowtime: 2 Minuten gelaufen");
  });
});

describe("miniRingSize", () => {
  it("fits the window and stays within bounds", () => {
    expect(miniRingSize(264, 280)).toBe(170);
    expect(miniRingSize(160, 190)).toBe(90);
    expect(miniRingSize(100, 100)).toBe(90);
    expect(miniRingSize(900, 900)).toBe(200);
    expect(miniRingSize(264, 400)).toBe(200);
  });
});
