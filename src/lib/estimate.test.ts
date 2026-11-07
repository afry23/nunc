import { describe, expect, it } from "vitest";
import type { EstimateCheck } from "./api";
import {
  estimateFromInput,
  estimateOf,
  formatDelta,
  formatEstimate,
  formatInUnit,
  onTargetPercent,
  progressPercent,
  splitMinutes,
  verdictLabel,
} from "./estimate";

const check = (
  unit: "minutes" | "pomodoros",
  value: number,
  actual: number,
): EstimateCheck => ({
  estimate: { unit, value },
  actual,
  delta: actual - value,
  verdict: "over",
});

describe("estimateOf", () => {
  it("liest Minuten und Pomodoros, Minuten gewinnen", () => {
    expect(estimateOf("Bericht")).toBeNull();
    expect(estimateOf("Bericht est:45")).toEqual({
      unit: "minutes",
      value: 45,
    });
    expect(estimateOf("Bericht estp:3")).toEqual({
      unit: "pomodoros",
      value: 3,
    });
    expect(estimateOf("Bericht estp:3 est:45")).toEqual({
      unit: "minutes",
      value: 45,
    });
  });
  it("ignoriert ungültige Werte", () => {
    expect(estimateOf("est:0 estp:0")).toBeNull();
    expect(estimateOf("est:10081")).toBeNull();
    expect(estimateOf("estp:1000")).toBeNull();
    expect(estimateOf("est:0 est:30")).toEqual({ unit: "minutes", value: 30 });
  });
});

describe("estimateFromInput", () => {
  const empty = { pomodoros: "", hours: "", minutes: "" };
  it("Pomodoros", () => {
    expect(
      estimateFromInput("pomodoros", { ...empty, pomodoros: "3" }),
    ).toEqual({
      unit: "pomodoros",
      value: 3,
    });
    for (const bad of ["", "0", "1000", "1.5", "-2"]) {
      expect(
        estimateFromInput("pomodoros", { ...empty, pomodoros: bad }),
      ).toBeNull();
    }
  });
  it("Stunden und Minuten", () => {
    expect(
      estimateFromInput("time", { ...empty, hours: "1", minutes: "30" }),
    ).toEqual({ unit: "minutes", value: 90 });
    expect(estimateFromInput("time", { ...empty, minutes: "45" })).toEqual({
      unit: "minutes",
      value: 45,
    });
    expect(estimateFromInput("time", { ...empty, hours: "2" })).toEqual({
      unit: "minutes",
      value: 120,
    });
    expect(estimateFromInput("time", empty)).toBeNull();
    expect(estimateFromInput("time", { ...empty, hours: "-1" })).toBeNull();
    expect(estimateFromInput("time", { ...empty, minutes: "-5" })).toBeNull();
    expect(estimateFromInput("time", { ...empty, hours: "1.5" })).toBeNull();
    expect(estimateFromInput("time", { ...empty, minutes: "1.5" })).toBeNull();
    expect(estimateFromInput("time", { ...empty, hours: "168" })).toEqual({
      unit: "minutes",
      value: 10080,
    });
    expect(estimateFromInput("time", { ...empty, hours: "169" })).toBeNull();
  });
});

describe("Anzeige", () => {
  it("zerlegt und formatiert", () => {
    expect(splitMinutes(90)).toEqual({ hours: 1, minutes: 30 });
    expect(formatEstimate({ unit: "pomodoros", value: 3 })).toBe("3 Pomodoros");
    expect(formatEstimate({ unit: "minutes", value: 90 })).toBe("1h 30m");
    expect(formatInUnit("pomodoros", 5)).toBe("5 Pomodoros");
    expect(formatInUnit("minutes", 25)).toBe("25m");
  });
  it("Abweichung", () => {
    expect(formatDelta(check("minutes", 60, 60))).toBe("genau getroffen");
    expect(formatDelta(check("minutes", 60, 85))).toBe("+25m");
    expect(formatDelta(check("pomodoros", 4, 3))).toBe("−1 Pomodoros");
  });
  it("Urteil und Anteile", () => {
    expect(verdictLabel("on_target")).toBe("im Rahmen");
    expect(onTargetPercent({ count: 0, onTarget: 0 })).toBeNull();
    expect(onTargetPercent({ count: 3, onTarget: 1 })).toBe(33);
    expect(progressPercent(check("minutes", 60, 30))).toBe(50);
    expect(progressPercent(check("minutes", 60, 200))).toBe(100);
  });
});
