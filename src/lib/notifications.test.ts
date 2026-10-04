import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  loadNotificationInfo,
  openExactAlarmSettings,
  openNotificationSettings,
  requestNotificationPermission,
} from "./notifications";

const ANDROID = "Mozilla/5.0 (Linux; Android 15; Pixel) AppleWebKit/537.36";
const IPHONE = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X)";

function bridge(exactAlarms: boolean) {
  return {
    canScheduleExactAlarms: vi.fn(() => exactAlarms),
    openExactAlarmSettings: vi.fn(),
    openNotificationSettings: vi.fn(),
  };
}

beforeEach(() => {
  mockIPC((cmd) => {
    if (cmd === "get_notification_status") return { permission: "prompt" };
    if (cmd === "request_notification_permission")
      return { permission: "granted" };
    throw new Error(cmd);
  });
});

afterEach(() => {
  clearMocks();
  Reflect.deleteProperty(window, "NuncAndroid");
  vi.restoreAllMocks();
});

describe("notification info", () => {
  it("reads the permission and the exact alarm state on Android", async () => {
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(ANDROID);
    window.NuncAndroid = bridge(false);
    expect(await loadNotificationInfo()).toEqual({
      permission: "prompt",
      exactAlarms: false,
    });
  });

  it("requests the permission", async () => {
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(ANDROID);
    window.NuncAndroid = bridge(true);
    expect(await requestNotificationPermission()).toEqual({
      permission: "granted",
      exactAlarms: true,
    });
  });

  it("assumes exact alarms where there is no Android bridge", async () => {
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(IPHONE);
    window.NuncAndroid = bridge(false);
    expect((await loadNotificationInfo()).exactAlarms).toBe(true);
    Reflect.deleteProperty(window, "NuncAndroid");
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(ANDROID);
    expect((await loadNotificationInfo()).exactAlarms).toBe(true);
  });
});

describe("settings shortcuts", () => {
  it("open the Android settings through the bridge", () => {
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(ANDROID);
    const android = bridge(true);
    window.NuncAndroid = android;
    openNotificationSettings();
    openExactAlarmSettings();
    expect(android.openNotificationSettings).toHaveBeenCalledOnce();
    expect(android.openExactAlarmSettings).toHaveBeenCalledOnce();
  });

  it("do nothing without the bridge", () => {
    vi.spyOn(navigator, "userAgent", "get").mockReturnValue(IPHONE);
    expect(() => {
      openNotificationSettings();
      openExactAlarmSettings();
    }).not.toThrow();
  });
});
