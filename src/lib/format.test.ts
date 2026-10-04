import { describe, expect, it } from "vitest";
import {
  addDays,
  formatBytes,
  formatClock,
  formatDateTime,
  formatTimeOfDay,
  formatUtcDate,
  formatMinutes,
  parseIsoDate,
  shortDate,
  toIsoDate,
  weekdayLabel,
} from "./format";

describe("formatClock", () => {
  it("formats minutes and seconds", () => {
    expect(formatClock(0)).toBe("00:00");
    expect(formatClock(90)).toBe("01:30");
    expect(formatClock(1500)).toBe("25:00");
    expect(formatClock(3599)).toBe("59:59");
  });
  it("adds hours from one hour on", () => {
    expect(formatClock(3600)).toBe("1:00:00");
    expect(formatClock(3725)).toBe("1:02:05");
  });
  it("clamps negatives and drops fractions", () => {
    expect(formatClock(-5)).toBe("00:00");
    expect(formatClock(59.9)).toBe("00:59");
  });
});

describe("formatMinutes", () => {
  it("formats minutes, hours and both", () => {
    expect(formatMinutes(0)).toBe("0m");
    expect(formatMinutes(45)).toBe("45m");
    expect(formatMinutes(120)).toBe("2h");
    expect(formatMinutes(65)).toBe("1h 5m");
    expect(formatMinutes(-3)).toBe("0m");
  });
});

describe("dates", () => {
  it("formats local dates, not UTC", () => {
    expect(toIsoDate(new Date(2026, 0, 5, 23, 59))).toBe("2026-01-05");
    expect(toIsoDate(new Date(2026, 11, 31, 0, 1))).toBe("2026-12-31");
  });
  it("roundtrips and adds days across month and year ends", () => {
    expect(toIsoDate(parseIsoDate("2026-10-04"))).toBe("2026-10-04");
    expect(toIsoDate(addDays(parseIsoDate("2026-12-30"), 3))).toBe(
      "2027-01-02",
    );
    expect(toIsoDate(addDays(parseIsoDate("2026-03-01"), -1))).toBe(
      "2026-02-28",
    );
  });
  it("labels weekdays and short dates", () => {
    expect(weekdayLabel("2026-10-04")).toBe("So");
    expect(weekdayLabel("2026-10-05")).toBe("Mo");
    expect(shortDate("2026-10-04")).toBe("04.10.");
  });
});

describe("formatBytes", () => {
  it("scales units", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2 KB");
    expect(formatBytes(3.4 * 1024 * 1024)).toBe("3,4 MB");
  });
});

import { ratioAnchor, ratioBadge } from "./format";

describe("flowtime ratio", () => {
  it("formats the badge without float noise", () => {
    expect(ratioBadge(0.2)).toBe("1 : 0.2");
    expect(ratioBadge(0.35)).toBe("1 : 0.35");
    expect(ratioBadge(0.1 + 0.2)).toBe("1 : 0.3");
  });
  it("names the anchor", () => {
    expect(ratioAnchor(0.1)).toBe("STRICT");
    expect(ratioAnchor(0.2)).toBe("STRICT");
    expect(ratioAnchor(0.25)).toBe("BALANCED");
    expect(ratioAnchor(0.35)).toBe("BALANCED");
    expect(ratioAnchor(0.4)).toBe("RELAXED");
    expect(ratioAnchor(0.5)).toBe("RELAXED");
  });
});

import { monthLabel } from "./format";

describe("monthLabel", () => {
  it("names the month in German", () => {
    expect(monthLabel(2026, 1)).toBe("Januar 2026");
    expect(monthLabel(2026, 3)).toBe("März 2026");
    expect(monthLabel(2025, 12)).toBe("Dezember 2025");
  });
  it("falls back to the number for invalid months", () => {
    expect(monthLabel(2026, 13)).toBe("13 2026");
  });
});

describe("formatDateTime / formatUtcDate", () => {
  it("formats local date and time", () => {
    const local = new Date(2026, 9, 4, 8, 5).toISOString();
    expect(formatDateTime(local)).toBe("04.10.2026, 08:05");
  });
  it("formats the UTC calendar day", () => {
    expect(formatUtcDate("2026-10-04T12:00:00Z")).toBe("04.10.2026");
    expect(formatUtcDate("2026-01-09T23:59:00Z")).toBe("09.01.2026");
  });
  it("returns unreadable input unchanged", () => {
    expect(formatDateTime("gestern")).toBe("gestern");
    expect(formatUtcDate("gestern")).toBe("gestern");
  });
});

describe("formatTimeOfDay", () => {
  it("shows the local time of day with two digits", () => {
    expect(formatTimeOfDay(new Date(2026, 9, 4, 8, 5).toISOString())).toBe(
      "08:05",
    );
    expect(formatTimeOfDay(new Date(2026, 9, 4, 23, 59).toISOString())).toBe(
      "23:59",
    );
  });
  it("returns unreadable input unchanged", () => {
    expect(formatTimeOfDay("gestern")).toBe("gestern");
  });
});
