import { beforeEach, describe, expect, it, vi } from "vitest";

const plugin = vi.hoisted(() => ({ platform: vi.fn() }));
vi.mock("@tauri-apps/plugin-os", () => plugin);

// Die Setup-Datei lädt `platform` schon vor dem Mock; ein frischer Import nimmt ihn.
let isAndroid: (userAgent?: string) => boolean;
let isMobile: (userAgent?: string) => boolean;

beforeEach(async () => {
  vi.resetModules();
  ({ isAndroid, isMobile } = await import("./platform"));
});

describe("platform from plugin-os", () => {
  it.each([
    ["android", true, true],
    ["ios", true, false],
    ["macos", false, false],
    ["linux", false, false],
  ] as const)("%s", (name, mobile, android) => {
    plugin.platform.mockReturnValue(name);
    expect([isMobile(), isAndroid()]).toEqual([mobile, android]);
  });

  it("prefers an explicit user agent", () => {
    plugin.platform.mockReturnValue("macos");
    expect(isMobile("Mozilla/5.0 (Linux; Android 15)")).toBe(true);
    expect(isAndroid("Mozilla/5.0 (iPhone)")).toBe(false);
  });

  it("falls back to the user agent without the plugin", () => {
    plugin.platform.mockImplementation(() => {
      throw new Error("not initialised");
    });
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(
      "Mozilla/5.0 (Linux; Android 15)",
    );
    expect([isMobile(), isAndroid()]).toEqual([true, true]);
    vi.restoreAllMocks();
  });
});
