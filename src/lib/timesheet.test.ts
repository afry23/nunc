import { describe, expect, it } from "vitest";
import type { TimesheetRow } from "../bindings";
import {
  blockColor,
  blockPlacement,
  dayToShow,
  defaultEntry,
  fromLocalInput,
  minuteOfDay,
  rowKey,
  rowLabel,
  timelineWindow,
  toLocalInput,
  weekLabel,
  weekRange,
} from "./timesheet";

const at = (hour: number, minute = 0) =>
  new Date(2026, 9, 4, hour, minute).toISOString();
const row = (over: Partial<TimesheetRow>): TimesheetRow => ({
  group: "+work",
  todoId: null,
  minutes: [],
  total: 0,
  ...over,
});

describe("weekRange", () => {
  it("runs from Monday to Sunday around today", () => {
    // 4.10.2026 ist ein Sonntag
    expect(weekRange(new Date(2026, 9, 4, 15), 0)).toEqual({
      startDate: "2026-09-28",
      endDate: "2026-10-04",
    });
    // 5.10.2026 ist ein Montag
    expect(weekRange(new Date(2026, 9, 5, 15), 0)).toEqual({
      startDate: "2026-10-05",
      endDate: "2026-10-11",
    });
  });
  it("moves back and forward by whole weeks", () => {
    const today = new Date(2026, 9, 7, 12);
    expect(weekRange(today, 1).startDate).toBe("2026-09-28");
    expect(weekRange(today, -1).startDate).toBe("2026-10-12");
  });
});

describe("weekLabel", () => {
  it("shows both ends", () => {
    expect(weekLabel("2026-09-28", "2026-10-04")).toBe("28.09. – 04.10.");
  });
});

describe("dayToShow", () => {
  const range = { startDate: "2026-09-28", endDate: "2026-10-04" };
  it("is today inside the week, including both ends", () => {
    expect(dayToShow(range, new Date(2026, 9, 1, 12))).toBe("2026-10-01");
    expect(dayToShow(range, new Date(2026, 8, 28, 12))).toBe("2026-09-28");
    expect(dayToShow(range, new Date(2026, 9, 4, 12))).toBe("2026-10-04");
  });
  it("is Monday outside the week", () => {
    expect(dayToShow(range, new Date(2026, 9, 5, 12))).toBe("2026-09-28");
    expect(dayToShow(range, new Date(2026, 8, 27, 12))).toBe("2026-09-28");
  });
});

describe("rowLabel and rowKey", () => {
  it("names untagged rows", () => {
    expect(rowLabel(row({ group: "+work" }))).toBe("+work");
    expect(rowLabel(row({ group: null }))).toBe("ohne Zuordnung");
  });
  it("keeps keys of different rows apart", () => {
    expect(rowKey(row({ group: null }))).toBe("|");
    expect(rowKey(row({ group: "a", todoId: 3 }))).toBe("3|a");
    expect(rowKey(row({ group: "a", todoId: 4 }))).not.toBe(
      rowKey(row({ group: "a", todoId: 3 })),
    );
  });
});

describe("minuteOfDay", () => {
  it("counts local minutes since midnight", () => {
    expect(minuteOfDay(at(0), "2026-10-04")).toBe(0);
    expect(minuteOfDay(at(9, 30), "2026-10-04")).toBe(570);
    expect(minuteOfDay(at(23, 59), "2026-10-04")).toBe(1439);
  });
  it("clamps other days to the day", () => {
    expect(minuteOfDay(at(9), "2026-10-05")).toBe(0);
    expect(minuteOfDay(at(9), "2026-10-03")).toBe(1440);
  });
});

describe("timelineWindow", () => {
  const block = (from: [number, number], to: [number, number]) => ({
    start: at(...from),
    end: at(...to),
  });
  it("has a working-day default without blocks", () => {
    expect(timelineWindow([], "2026-10-04")).toEqual({
      fromHour: 8,
      toHour: 18,
    });
  });
  it("adds an hour of margin around the blocks", () => {
    expect(
      timelineWindow(
        [block([9, 30], [10, 0]), block([13, 0], [14, 15])],
        "2026-10-04",
      ),
    ).toEqual({ fromHour: 8, toHour: 16 });
  });
  it("is at least four hours long", () => {
    expect(timelineWindow([block([10, 0], [10, 30])], "2026-10-04")).toEqual({
      fromHour: 9,
      toHour: 13,
    });
  });
  it("stays inside the day", () => {
    expect(timelineWindow([block([0, 0], [0, 30])], "2026-10-04")).toEqual({
      fromHour: 0,
      toHour: 4,
    });
    expect(timelineWindow([block([23, 0], [23, 59])], "2026-10-04")).toEqual({
      fromHour: 20,
      toHour: 24,
    });
  });
});

describe("blockPlacement", () => {
  const window = { fromHour: 8, toHour: 18 };
  it("places a block in percent of the window", () => {
    expect(
      blockPlacement({ start: at(9), end: at(10, 30) }, "2026-10-04", window),
    ).toEqual({ top: 10, height: 15 });
  });
  it("keeps tiny blocks tappable", () => {
    const placed = blockPlacement(
      { start: at(9), end: at(9, 1) },
      "2026-10-04",
      window,
    );
    expect(placed.height).toBe(2);
  });
  it("clips blocks that stick out of the window", () => {
    expect(
      blockPlacement({ start: at(6), end: at(20) }, "2026-10-04", window),
    ).toEqual({ top: 0, height: 100 });
  });
});

describe("blockColor", () => {
  it("has one token per session type", () => {
    expect(blockColor("work")).toBe("bg-pomodoro");
    expect(blockColor("short_break")).toBe("bg-shortbrk");
    expect(blockColor("long_break")).toBe("bg-longbrk");
  });
});

describe("local input values", () => {
  it("round-trips a moment", () => {
    const iso = at(9, 5);
    expect(toLocalInput(iso)).toBe("2026-10-04T09:05");
    expect(fromLocalInput("2026-10-04T09:05")).toBe(iso);
  });
  it("rejects empty and broken values", () => {
    expect(fromLocalInput("")).toBeNull();
    expect(fromLocalInput("gestern")).toBeNull();
  });
  it("offers one hour in the morning for a new entry", () => {
    expect(defaultEntry("2026-10-04")).toEqual({
      start: "2026-10-04T09:00",
      end: "2026-10-04T10:00",
    });
  });
});
