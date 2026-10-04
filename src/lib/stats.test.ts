import { describe, expect, it } from "vitest";
import {
  barLayout,
  fillDays,
  heatmapCells,
  linePoints,
  niceMax,
  pathThrough,
  periodRange,
  periodTotals,
  pieSlices,
  slicePath,
} from "./stats";

describe("periodRange", () => {
  const today = new Date(2026, 9, 4, 15);
  it("ends today and covers the window", () => {
    expect(periodRange("day", today)).toEqual({
      startDate: "2026-10-04",
      endDate: "2026-10-04",
    });
    expect(periodRange("week", today)).toEqual({
      startDate: "2026-09-28",
      endDate: "2026-10-04",
    });
    expect(periodRange("month", today)).toEqual({
      startDate: "2026-09-05",
      endDate: "2026-10-04",
    });
    expect(periodRange("year", today)).toEqual({
      startDate: "2025-10-05",
      endDate: "2026-10-04",
    });
  });
});

describe("fillDays", () => {
  it("adds missing days with zeros", () => {
    const days = fillDays(
      [
        {
          date: "2026-10-02",
          pomodoroCount: 3,
          workMinutes: 75,
          breakMinutes: 10,
        },
      ],
      "2026-10-01",
      "2026-10-03",
    );
    expect(
      days.map((d) => [d.date, d.pomodoroCount, d.workMinutes, d.breakMinutes]),
    ).toEqual([
      ["2026-10-01", 0, 0, 0],
      ["2026-10-02", 3, 75, 10],
      ["2026-10-03", 0, 0, 0],
    ]);
  });
  it("is empty when the end precedes the start", () => {
    expect(fillDays([], "2026-10-03", "2026-10-01")).toEqual([]);
  });
});

describe("niceMax", () => {
  it("rounds up to 1, 2, 5 or 10 times a power of ten", () => {
    expect(niceMax(0)).toBe(1);
    expect(niceMax(1)).toBe(1);
    expect(niceMax(1.5)).toBe(2);
    expect(niceMax(3)).toBe(5);
    expect(niceMax(7)).toBe(10);
    expect(niceMax(130)).toBe(200);
    expect(niceMax(500)).toBe(500);
    expect(niceMax(501)).toBe(1000);
  });
});

describe("barLayout", () => {
  it("scales bars to the nice maximum", () => {
    const bars = barLayout([0, 5, 10], 100, 50, 10);
    expect(bars).toHaveLength(3);
    expect(bars[2].height).toBe(50);
    expect(bars[1].height).toBe(25);
    expect(bars[0].height).toBe(0);
    expect(bars[0].y).toBe(50);
    expect(bars[1].x).toBeCloseTo(bars[0].width + 10);
  });
  it("handles empty input and all zero", () => {
    expect(barLayout([], 100, 50)).toEqual([]);
    expect(barLayout([0, 0], 100, 50)[0].height).toBe(0);
  });
});

describe("line", () => {
  it("spreads points and flips the y axis", () => {
    const points = linePoints([0, 10], 100, 40);
    expect(points).toEqual([
      { x: 0, y: 40 },
      { x: 100, y: 0 },
    ]);
    expect(linePoints([5], 100, 40)[0].x).toBe(50);
    expect(linePoints([], 100, 40)).toEqual([]);
  });
  it("builds a path", () => {
    expect(
      pathThrough([
        { x: 0, y: 1 },
        { x: 2, y: 3.14159 },
      ]),
    ).toBe("M0.0,1.0 L2.0,3.1");
    expect(pathThrough([])).toBe("");
  });
});

describe("heatmapCells", () => {
  it("places days in week columns starting on Monday", () => {
    // 2026-10-04 ist ein Sonntag (Zeile 6), danach beginnt die nächste Woche
    const cells = heatmapCells([
      { date: "2026-10-04", pomodoroCount: 4, workMinutes: 0, breakMinutes: 0 },
      { date: "2026-10-05", pomodoroCount: 0, workMinutes: 0, breakMinutes: 0 },
      { date: "2026-10-06", pomodoroCount: 1, workMinutes: 0, breakMinutes: 0 },
    ]);
    expect(cells.map((c) => [c.column, c.row, c.level])).toEqual([
      [0, 6, 4],
      [1, 0, 0],
      [1, 1, 1],
    ]);
  });
  it("is empty without days", () => expect(heatmapCells([])).toEqual([]));
  it("scales levels to the maximum", () => {
    const day = (n: number, i: number) => ({
      date: `2026-10-0${i + 1}`,
      pomodoroCount: n,
      workMinutes: 0,
      breakMinutes: 0,
    });
    expect(
      heatmapCells([day(2, 0), day(4, 1), day(8, 2)]).map((c) => c.level),
    ).toEqual([1, 2, 4]);
  });
});

describe("pie", () => {
  it("computes shares and angles, skipping zero values", () => {
    const slices = pieSlices([
      { label: "a", value: 30 },
      { label: "z", value: 0 },
      { label: "b", value: 10 },
    ]);
    expect(slices.map((s) => s.label)).toEqual(["a", "b"]);
    expect(slices[0].share).toBe(0.75);
    expect(slices[0].start).toBe(0);
    expect(slices[1].end).toBeCloseTo(2 * Math.PI);
    expect(pieSlices([])).toEqual([]);
  });
  it("draws segments and a full circle", () => {
    expect(slicePath(50, 50, 40, 0, Math.PI / 2)).toBe(
      "M50,50 L50.00,10.00 A40,40 0 0 1 90.00,50.00 Z",
    );
    expect(slicePath(50, 50, 40, 0, (3 * Math.PI) / 2)).toContain(" 0 1 1 ");
    expect(slicePath(50, 50, 40, 0, 2 * Math.PI)).toContain("A40,40 0 1 1");
  });
});

describe("periodTotals", () => {
  const day = (pomodoroCount: number, workMinutes: number) => ({
    date: "2026-10-01",
    pomodoroCount,
    workMinutes,
    breakMinutes: 0,
  });
  it("sums the days and averages per day", () => {
    expect(
      periodTotals([day(4, 100), day(2, 50), day(0, 0), day(6, 150)]),
    ).toEqual({
      pomodoros: 12,
      workMinutes: 300,
      perDay: 3,
    });
  });
  it("is zero without days", () => {
    expect(periodTotals([])).toEqual({
      pomodoros: 0,
      workMinutes: 0,
      perDay: 0,
    });
  });
});
