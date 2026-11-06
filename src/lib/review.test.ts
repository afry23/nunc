import { describe, expect, it } from "vitest";
import type { Todo } from "../bindings";
import {
  clampDay,
  deltaText,
  focusOnlyMinutes,
  isWeekReviewTime,
  openTasks,
  rangeLabel,
  reviewRange,
} from "./review";

describe("reviewRange", () => {
  // 4.10.2026 ist ein Sonntag
  const today = new Date(2026, 9, 4, 10);

  it("runs a week from Monday to Sunday", () => {
    expect(reviewRange("week", today, 0)).toEqual({
      from: "2026-09-28",
      to: "2026-10-04",
    });
    expect(reviewRange("week", today, 1)).toEqual({
      from: "2026-09-21",
      to: "2026-09-27",
    });
    expect(reviewRange("week", today, -1)).toEqual({
      from: "2026-10-05",
      to: "2026-10-11",
    });
  });

  it("covers a whole calendar month, also across years", () => {
    expect(reviewRange("month", today, 0)).toEqual({
      from: "2026-10-01",
      to: "2026-10-31",
    });
    expect(reviewRange("month", today, 10)).toEqual({
      from: "2025-12-01",
      to: "2025-12-31",
    });
    expect(reviewRange("month", today, 8)).toEqual({
      from: "2026-02-01",
      to: "2026-02-28",
    });
  });

  it("covers a whole calendar quarter, also across years", () => {
    expect(reviewRange("quarter", today, 0)).toEqual({
      from: "2026-10-01",
      to: "2026-12-31",
    });
    expect(reviewRange("quarter", today, 1)).toEqual({
      from: "2026-07-01",
      to: "2026-09-30",
    });
    expect(reviewRange("quarter", today, 4)).toEqual({
      from: "2025-10-01",
      to: "2025-12-31",
    });
  });
});

describe("rangeLabel", () => {
  it("shows both days and the year", () => {
    expect(rangeLabel("2026-03-02", "2026-03-08")).toBe("02.03. – 08.03. 2026");
  });
});

describe("deltaText", () => {
  it("signs the change against the previous period", () => {
    expect(deltaText({ focusMin: 85, previousFocusMin: 40 })).toBe("+45m");
    expect(deltaText({ focusMin: 40, previousFocusMin: 100 })).toBe("−1h");
    expect(deltaText({ focusMin: 30, previousFocusMin: 30 })).toBe("±0m");
  });
});

describe("isWeekReviewTime", () => {
  it("starts on Friday and lasts until Sunday", () => {
    // 28.9.2026 ist ein Montag
    const days = [28, 29, 30, 31, 32, 33, 34].map(
      (day) => new Date(2026, 8, day, 10),
    );
    expect(days.map(isWeekReviewTime)).toEqual([
      false,
      false,
      false,
      false,
      true,
      true,
      true,
    ]);
  });
});

describe("clampDay", () => {
  it("keeps a day inside the range and clamps one outside", () => {
    expect(clampDay("2026-03-04", "2026-03-02", "2026-03-08")).toBe(
      "2026-03-04",
    );
    expect(clampDay("2026-03-02", "2026-03-02", "2026-03-08")).toBe(
      "2026-03-02",
    );
    expect(clampDay("2026-03-08", "2026-03-02", "2026-03-08")).toBe(
      "2026-03-08",
    );
    expect(clampDay("2026-02-27", "2026-03-02", "2026-03-08")).toBe(
      "2026-03-02",
    );
    expect(clampDay("2026-03-20", "2026-03-02", "2026-03-08")).toBe(
      "2026-03-08",
    );
  });
});

describe("focusOnlyMinutes", () => {
  it("counts pomodoro and flowtime but not the stopwatch", () => {
    expect(
      focusOnlyMinutes([
        { mode: "pomodoro", sessions: 3, minutes: 60 },
        { mode: "flowtime", sessions: 1, minutes: 30 },
        { mode: "stopwatch", sessions: 2, minutes: 45 },
      ]),
    ).toBe(90);
  });

  it("is zero without sessions", () => {
    expect(focusOnlyMinutes([])).toBe(0);
  });
});

describe("openTasks", () => {
  const task = (todoId: number) => ({
    todoId,
    text: `Task ${todoId}`,
    minutes: 10,
    pomodoros: 0,
    sessions: 1,
  });
  const todo = (id: number, completed: boolean) =>
    ({ id, completed }) as unknown as Todo;

  it("keeps the tasks of open todos only", () => {
    expect(
      openTasks(
        [task(1), task(2), task(3)],
        [todo(1, false), todo(2, true)],
      ).map((entry) => entry.todoId),
    ).toEqual([1]);
  });

  it("drops tasks whose todo is gone", () => {
    expect(openTasks([task(9)], [])).toEqual([]);
  });
});
