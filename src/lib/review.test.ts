import { describe, expect, it } from "vitest";
import { deltaText, rangeLabel, reviewRange } from "./review";

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
