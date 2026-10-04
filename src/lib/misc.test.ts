import { describe, expect, it, vi } from "vitest";
import { ApiError, errorMessage, unwrap } from "./api";
import { contrastRatio } from "./contrast";
import { isMobile } from "./platform";
import { SOUND_NAMES, playSound, toneRecipe } from "./sound";

describe("unwrap", () => {
  it("returns data of ok results", async () => {
    expect(
      await unwrap(Promise.resolve({ status: "ok" as const, data: 7 })),
    ).toBe(7);
  });
  it("throws ApiError with code and message", async () => {
    const failing = Promise.resolve({
      status: "error" as const,
      error: { code: "not_found", message: "Todo 7 nicht gefunden" },
    });
    await expect(unwrap(failing)).rejects.toMatchObject({
      name: "ApiError",
      code: "not_found",
      message: "Todo 7 nicht gefunden",
    });
    await expect(unwrap(failing)).rejects.toBeInstanceOf(ApiError);
  });
});

describe("errorMessage", () => {
  it("reads errors and falls back", () => {
    expect(errorMessage(new Error("kaputt"))).toBe("kaputt");
    expect(errorMessage("x")).toBe("Unbekannter Fehler");
  });
});

describe("contrastRatio", () => {
  it("matches known values", () => {
    expect(contrastRatio("#000000", "#ffffff")).toBeCloseTo(21, 0);
    expect(contrastRatio("#ffffff", "#ffffff")).toBe(1);
    expect(contrastRatio("#777777", "#ffffff")).toBeCloseTo(4.48, 1);
  });
  it("is symmetric", () => {
    expect(contrastRatio("#4de082", "#131313")).toBe(
      contrastRatio("#131313", "#4de082"),
    );
  });
});

describe("design tokens meet WCAG AA", () => {
  const surface = "#131313";
  const cases: [string, string, string, number][] = [
    ["Text auf Fläche", "#e5e2e1", surface, 4.5],
    ["Sekundärtext auf Fläche", "#c6c6cd", surface, 4.5],
    ["Sekundärtext auf Karte", "#c6c6cd", "#1c1b1b", 4.5],
    ["Sekundärtext auf Eingabe", "#c6c6cd", "#353534", 4.5],
    ["Primär auf Fläche", "#4de082", surface, 4.5],
    ["Text auf Primär", "#003919", "#4de082", 4.5],
    ["Text auf Primärverlauf (Ende)", "#e5e2e1", "#00944b", 3],
    ["Pomodoro-Rot auf Fläche", "#ef4444", surface, 3],
    ["Flowtime-Violett auf Fläche", "#a855f7", surface, 3],
    ["Pause-Grün auf Fläche", "#22c55e", surface, 3],
    ["Lange Pause auf Fläche", "#3b82f6", surface, 3],
    ["Chip-Text", "#263143", "#bcc7de", 4.5],
  ];
  it.each(cases)("%s", (_name, fg, bg, minimum) => {
    expect(contrastRatio(fg, bg)).toBeGreaterThanOrEqual(minimum);
  });
});

describe("isMobile", () => {
  it("detects touch platforms by user agent", () => {
    expect(isMobile("Mozilla/5.0 (Linux; Android 15)")).toBe(true);
    expect(isMobile("Mozilla/5.0 (iPhone; CPU iPhone OS 18_0)")).toBe(true);
    expect(isMobile("Mozilla/5.0 (Macintosh; Intel Mac OS X)")).toBe(false);
  });
  it("defaults to the current user agent", () =>
    expect(isMobile()).toBe(false));
});

describe("sound", () => {
  it("has a recipe per name with a decaying envelope", () => {
    for (const name of SOUND_NAMES) {
      const tones = toneRecipe(name);
      expect(tones.length).toBeGreaterThan(0);
      expect(tones.every((t) => t.freq > 0 && t.decay > 0 && t.gain > 0)).toBe(
        true,
      );
    }
    expect(toneRecipe("bell")).toHaveLength(4);
    expect(toneRecipe("digital").map((t) => t.delay)).toEqual([0, 0.18, 0.36]);
    expect(toneRecipe("unknown")).toEqual(toneRecipe("ding"));
  });
  it("plays without Audio support and never throws", () => {
    expect(() => playSound("bell", 80)).not.toThrow();
  });
  it("drives the audio graph when available", () => {
    const gain = {
      setValueAtTime: vi.fn(),
      linearRampToValueAtTime: vi.fn(),
      exponentialRampToValueAtTime: vi.fn(),
    };
    const osc = {
      connect: vi.fn(),
      start: vi.fn(),
      stop: vi.fn(),
      frequency: { setValueAtTime: vi.fn() },
      type: "",
    };
    class FakeContext {
      state = "suspended";
      currentTime = 1;
      destination = {};
      resume = vi.fn();
      createOscillator = () => osc;
      createGain = () => ({ connect: vi.fn(), gain });
    }
    vi.stubGlobal("AudioContext", FakeContext);
    playSound("digital", 50);
    expect(osc.start).toHaveBeenCalledTimes(3);
    expect(gain.linearRampToValueAtTime).toHaveBeenCalledWith(
      0.35 * 0.5,
      expect.any(Number),
    );
    vi.unstubAllGlobals();
  });
});
