import { describe, expect, it } from "vitest";
import type { ActivityEntry, TaskTotals, Todo } from "../bindings";
import {
  bucketDays,
  clampDay,
  deltaText,
  filterActivities,
  filterTasks,
  granularityFor,
  hourRange,
  projectOptions,
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
    completed: false,
    completedAt: null,
    projects: [],
    estimateCheck: null,
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

describe("reviewRange for day and year", () => {
  const today = new Date(2026, 9, 4, 10);
  it("covers one day, also in the past", () => {
    expect(reviewRange("day", today, 0)).toEqual({
      from: "2026-10-04",
      to: "2026-10-04",
    });
    expect(reviewRange("day", today, 2)).toEqual({
      from: "2026-10-02",
      to: "2026-10-02",
    });
  });
  it("covers a whole calendar year", () => {
    expect(reviewRange("year", today, 0)).toEqual({
      from: "2026-01-01",
      to: "2026-12-31",
    });
    expect(reviewRange("year", today, 1)).toEqual({
      from: "2025-01-01",
      to: "2025-12-31",
    });
  });
});

describe("bucketDays", () => {
  const day = (date: string, minutes: number, pomodoros = 0) => ({
    date,
    minutes,
    pomodoros,
    pomodoroMin: minutes,
    flowtimeMin: 0,
    stopwatchMin: 0,
  });
  it("picks the granularity from the period", () => {
    expect(
      ["day", "week", "month", "quarter", "year"].map((kind) =>
        granularityFor(kind as never),
      ),
    ).toEqual(["day", "day", "day", "week", "month"]);
  });
  it("fills every day with zero", () => {
    const points = bucketDays(
      [day("2026-03-03", 30, 1)],
      "2026-03-02",
      "2026-03-04",
      "day",
    );
    expect(points.map((p) => [p.date, p.workMinutes, p.pomodoroCount])).toEqual(
      [
        ["2026-03-02", 0, 0],
        ["2026-03-03", 30, 1],
        ["2026-03-04", 0, 0],
      ],
    );
  });
  it("sums weeks starting Monday", () => {
    const points = bucketDays(
      [day("2026-03-01", 10), day("2026-03-02", 20), day("2026-03-08", 5)],
      "2026-03-01",
      "2026-03-09",
      "week",
    );
    expect(
      points.map((p) => [p.date, p.workMinutes, p.pomodoroMinutes]),
    ).toEqual([
      ["2026-02-23", 10, 10],
      ["2026-03-02", 25, 25],
      ["2026-03-09", 0, 0],
    ]);
  });
  it("sums months", () => {
    const points = bucketDays(
      [day("2026-01-31", 10), day("2026-02-01", 20)],
      "2026-01-01",
      "2026-02-28",
      "month",
    );
    expect(points.map((p) => [p.date, p.workMinutes])).toEqual([
      ["2026-01-01", 10],
      ["2026-02-01", 20],
    ]);
  });
});

describe("project filter", () => {
  const task = (id: number, projects: string[]) =>
    ({ todoId: id, projects }) as unknown as TaskTotals;
  const activity = (id: number, projects: string[]) =>
    ({ sessionId: id, projects }) as unknown as ActivityEntry;
  const none = "ohne";
  const tasks = [task(1, ["+a"]), task(2, []), task(3, ["+b", "+a"])];
  const activities = [activity(1, []), activity(2, ["+c"])];
  it("keeps everything without a selection", () => {
    expect(filterTasks(tasks, [], none)).toHaveLength(3);
    expect(filterActivities(activities, [], none)).toHaveLength(2);
  });
  it("matches any selected project", () => {
    expect(filterTasks(tasks, ["+a"], none).map((t) => t.todoId)).toEqual([
      1, 3,
    ]);
    expect(filterTasks(tasks, ["+b", "+z"], none).map((t) => t.todoId)).toEqual(
      [3],
    );
  });
  it("matches work without a project via the none entry", () => {
    expect(filterTasks(tasks, [none], none).map((t) => t.todoId)).toEqual([2]);
    expect(filterActivities(activities, [none, "+c"], none)).toHaveLength(2);
    expect(filterActivities(activities, ["+c"], none)).toHaveLength(1);
  });
  it("lists the projects sorted, with the none entry first when needed", () => {
    expect(projectOptions(tasks, activities, none)).toEqual([
      none,
      "+a",
      "+b",
      "+c",
    ]);
    expect(projectOptions([task(1, ["+b"])], [], none)).toEqual(["+b"]);
  });
});

describe("hourRange", () => {
  it("spans the first to the last hour with work", () => {
    const hours = Array<number>(24).fill(0);
    hours[9] = 5;
    hours[14] = 1;
    expect(hourRange(hours)).toEqual({ first: 9, last: 14 });
  });
  it("is null without work", () => {
    expect(hourRange(Array<number>(24).fill(0))).toBeNull();
  });
});
