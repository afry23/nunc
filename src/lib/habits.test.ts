import { describe, expect, it } from "vitest";
import type { HabitItem, HabitProgress } from "./api";
import {
  EMPTY_DRAFT,
  draftOf,
  goalOf,
  isAuto,
  isDraftValid,
  recentCells,
  scheduleOf,
  splitHabits,
  streakKey,
  todayHabits,
  toggleDay,
} from "./habits";

const progress = (extra: Partial<HabitProgress> = {}): HabitProgress => ({
  doneToday: false,
  openToday: false,
  currentStreak: 0,
  bestStreak: 0,
  unit: "days",
  weekDone: 0,
  weekTarget: 7,
  ...extra,
});

const habit = (id: string, extra: Partial<HabitItem> = {}): HabitItem => ({
  id,
  name: id,
  created: "2026-03-01",
  archived: false,
  schedule: { kind: "daily" },
  goal: { kind: "check" },
  progress: progress(),
  recent: [],
  ...extra,
});

describe("draftOf", () => {
  it("takes the fields of a habit and keeps the defaults for the rest", () => {
    expect(draftOf(habit("Read"))).toEqual({ ...EMPTY_DRAFT, name: "Read" });
    expect(
      draftOf(habit("Run", { schedule: { kind: "days", days: [2, 4] } })),
    ).toMatchObject({ schedule: "days", days: [2, 4], perWeek: 3 });
    expect(
      draftOf(habit("Run", { schedule: { kind: "weekly", perWeek: 5 } })),
    ).toMatchObject({ schedule: "weekly", perWeek: 5 });
    expect(
      draftOf(
        habit("Focus", {
          goal: { kind: "minutes", minutes: 45, tag: "+work" },
        }),
      ),
    ).toMatchObject({ goal: "minutes", minutes: 45, tag: "+work" });
  });

  it("falls back to the defaults for an encoding of a newer version", () => {
    expect(draftOf(habit("New", { schedule: null, goal: null }))).toEqual({
      ...EMPTY_DRAFT,
      name: "New",
    });
  });
});

describe("scheduleOf and goalOf", () => {
  it("builds the schedule of the chosen kind", () => {
    expect(scheduleOf(EMPTY_DRAFT)).toEqual({ kind: "daily" });
    expect(
      scheduleOf({ ...EMPTY_DRAFT, schedule: "days", days: [1, 3] }),
    ).toEqual({ kind: "days", days: [1, 3] });
    expect(
      scheduleOf({ ...EMPTY_DRAFT, schedule: "weekly", perWeek: 4 }),
    ).toEqual({ kind: "weekly", perWeek: 4 });
  });

  it("builds the goal and trims the tag", () => {
    expect(goalOf(EMPTY_DRAFT)).toEqual({ kind: "check" });
    expect(
      goalOf({ ...EMPTY_DRAFT, goal: "minutes", minutes: 20, tag: " @home " }),
    ).toEqual({ kind: "minutes", minutes: 20, tag: "@home" });
  });
});

describe("toggleDay", () => {
  it("adds and removes a day and keeps the order", () => {
    expect(toggleDay([1, 3], 2)).toEqual([1, 2, 3]);
    expect(toggleDay([1, 2, 3], 2)).toEqual([1, 3]);
    expect(toggleDay([], 7)).toEqual([7]);
  });
});

describe("isDraftValid", () => {
  it("needs a name, a weekday for weekdays and a tag for a minutes goal", () => {
    expect(isDraftValid({ ...EMPTY_DRAFT, name: "Read" })).toBe(true);
    expect(isDraftValid({ ...EMPTY_DRAFT, name: "  " })).toBe(false);
    const named = { ...EMPTY_DRAFT, name: "Read" };
    expect(isDraftValid({ ...named, schedule: "days", days: [] })).toBe(false);
    expect(isDraftValid({ ...named, schedule: "days", days: [1] })).toBe(true);
    expect(isDraftValid({ ...named, goal: "minutes", tag: " " })).toBe(false);
    expect(isDraftValid({ ...named, goal: "minutes", tag: "+work" })).toBe(
      true,
    );
  });
});

describe("isAuto", () => {
  it("is true only for a minutes goal", () => {
    expect(isAuto(habit("a"))).toBe(false);
    expect(isAuto(habit("b", { goal: null }))).toBe(false);
    expect(
      isAuto(habit("c", { goal: { kind: "minutes", minutes: 5, tag: "+x" } })),
    ).toBe(true);
  });
});

describe("streakKey", () => {
  it("picks singular and plural of days and weeks", () => {
    const key = (unit: "days" | "weeks", currentStreak: number) =>
      streakKey({ unit, currentStreak });
    expect(key("days", 1)).toBe("habits.streak.day");
    expect(key("days", 0)).toBe("habits.streak.days");
    expect(key("days", 3)).toBe("habits.streak.days");
    expect(key("weeks", 1)).toBe("habits.streak.week");
    expect(key("weeks", 2)).toBe("habits.streak.weeks");
  });
});

describe("recentCells", () => {
  it("lists the days up to today, oldest first, and marks the done ones", () => {
    const cells = recentCells(["2026-03-02", "2026-03-04"], "2026-03-04", 4);
    expect(cells).toEqual([
      { day: "2026-03-01", done: false },
      { day: "2026-03-02", done: true },
      { day: "2026-03-03", done: false },
      { day: "2026-03-04", done: true },
    ]);
    expect(recentCells([], "2026-03-04")).toHaveLength(14);
  });
});

describe("splitHabits", () => {
  it("separates archived habits and keeps the order", () => {
    const { active, archived } = splitHabits([
      habit("a"),
      habit("b", { archived: true }),
      habit("c"),
    ]);
    expect(active.map((h) => h.id)).toEqual(["a", "c"]);
    expect(archived.map((h) => h.id)).toEqual(["b"]);
  });
});

describe("todayHabits", () => {
  it("lists open habits first, then done ones, and skips the rest", () => {
    const items = [
      habit("done", { progress: progress({ doneToday: true }) }),
      habit("idle"),
      habit("open", { progress: progress({ openToday: true }) }),
      habit("old", {
        archived: true,
        progress: progress({ openToday: true }),
      }),
      habit("new", { progress: null }),
    ];
    expect(todayHabits(items).map((h) => h.id)).toEqual(["open", "done"]);
  });
});
