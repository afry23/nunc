import { describe, expect, it } from "vitest";
import { shortcutFor, type ShortcutEvent } from "./shortcuts";

const press = (
  key: string,
  over: Partial<ShortcutEvent> = {},
): ShortcutEvent => ({
  key,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  target: document.body,
  ...over,
});

describe("shortcutFor", () => {
  it("maps keys to actions", () => {
    expect(shortcutFor(press(" "))).toBe("toggle-timer");
    expect(shortcutFor(press("Escape"))).toBe("stop");
    expect(shortcutFor(press("n"))).toBe("new-todo");
    expect(shortcutFor(press("N"))).toBe("new-todo");
    expect(shortcutFor(press("1"))).toBe("go-timer");
    expect(shortcutFor(press("2"))).toBe("go-todos");
    expect(shortcutFor(press("3"))).toBe("go-stats");
    expect(shortcutFor(press(","))).toBe("go-settings");
    expect(shortcutFor(press("x"))).toBeNull();
  });

  it("ignores modifiers and open dialogs", () => {
    expect(shortcutFor(press("1", { ctrlKey: true }))).toBeNull();
    expect(shortcutFor(press("1", { metaKey: true }))).toBeNull();
    expect(shortcutFor(press("1", { altKey: true }))).toBeNull();
    expect(shortcutFor(press("1"), true)).toBeNull();
  });

  it("ignores typing in form fields", () => {
    for (const tag of ["input", "textarea", "select"]) {
      expect(
        shortcutFor(press("1", { target: document.createElement(tag) })),
      ).toBeNull();
    }
    const editable = document.createElement("div");
    Object.defineProperty(editable, "isContentEditable", { value: true });
    expect(shortcutFor(press("n", { target: editable }))).toBeNull();
  });

  it("lets buttons and links keep the space key", () => {
    expect(
      shortcutFor(press(" ", { target: document.createElement("button") })),
    ).toBeNull();
    expect(
      shortcutFor(press(" ", { target: document.createElement("a") })),
    ).toBeNull();
    const roleButton = document.createElement("div");
    roleButton.setAttribute("role", "button");
    expect(shortcutFor(press(" ", { target: roleButton }))).toBeNull();
    // andere Tasten bleiben auf Buttons aktiv
    expect(
      shortcutFor(press("2", { target: document.createElement("button") })),
    ).toBe("go-todos");
  });

  it("works without a target", () => {
    expect(shortcutFor(press(" ", { target: null }))).toBe("toggle-timer");
  });
});
