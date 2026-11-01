import { readFileSync } from "node:fs";
import { act, fireEvent, screen, waitFor } from "@testing-library/react";
import { expect, vi } from "vitest";
import { Given, Then, When } from "quickpickle";
import type { TimerSnapshot } from "../../src/bindings";
import type { UiWorld } from "./world";

const settle = (): Promise<void> =>
  act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 0));
  });

// ---------------------------------------------------------------------------------------------
// Safe Areas und Touch-Ziele

Then("the page extends below the display cutout", () => {
  const html = readFileSync("index.html", "utf8");
  expect(html).toMatch(/<meta[^>]*name="viewport"[^>]*viewport-fit=cover/s);
});

Then("the content keeps clear of the system bars", () => {
  expect(screen.getByTestId("app-shell").className).toContain("safe-top");
  expect(screen.getByTestId("tabbar").className).toContain("safe-bottom");
});

const CONTROLS =
  'button, a[href], select, input:not([type="hidden"]), [role="switch"], [role="radio"]';
const MIN_REM = 11; // Tailwind-Stufe 11 = 2,75 rem = 44 px

/** Höhe in Tailwind-Stufen aus `h-*`, `min-h-*` oder `size-*`; ohne Angabe 0. */
function heightStep(element: Element): number {
  let best = 0;
  for (const token of element.className.toString().split(/\s+/)) {
    const match = /^(?:min-h|h|size)-(\d+(?:\.\d+)?)$/.exec(token);
    if (match) best = Math.max(best, Number(match[1]));
  }
  return best;
}

Then("every control is at least 44 pixels high", () => {
  const offenders: string[] = [];
  for (const control of document.querySelectorAll(CONTROLS)) {
    if (control.closest(".sr-only, .sr-only *")) continue;
    // Kontrollkästchen und Radios haben ihre Trefferfläche im umgebenden Label
    const box =
      control instanceof HTMLInputElement &&
      ["checkbox", "radio"].includes(control.type)
        ? (control.closest("label") ?? control)
        : control;
    if (heightStep(box) < MIN_REM)
      offenders.push(
        `${control.tagName.toLowerCase()} "${control.getAttribute("aria-label") ?? control.textContent?.trim() ?? control.id}"`,
      );
  }
  expect(offenders).toEqual([]);
});

// ---------------------------------------------------------------------------------------------
// Wischen und Ziehen

type Touch = { clientX: number; clientY: number };
const touch = (x: number, y: number): Touch => ({ clientX: x, clientY: y });

When(
  "I swipe the todo {string} to the right by {int} pixels",
  async (_world: UiWorld, name: string, distance: number) => {
    const row = (await screen.findByText(name)).closest(
      '[data-testid="todo-row"]',
    ) as HTMLElement;
    await act(async () => {
      fireEvent.touchStart(row, { touches: [touch(20, 100)] });
      fireEvent.touchMove(row, { touches: [touch(20 + distance, 104)] });
      fireEvent.touchEnd(row, {
        changedTouches: [touch(20 + distance, 104)],
      });
    });
    await settle();
  },
);

When(
  "I pull the page down by {int} pixels",
  async (_world: UiWorld, distance: number) => {
    Object.defineProperty(window, "scrollY", { value: 0, configurable: true });
    await act(async () => {
      fireEvent.touchStart(document.body, { touches: [touch(100, 80)] });
      fireEvent.touchMove(document.body, {
        touches: [touch(100, 80 + distance)],
      });
    });
  },
);

When("I release the page", async () => {
  await act(async () => {
    fireEvent.touchEnd(document.body, { changedTouches: [touch(100, 300)] });
  });
  await settle();
});

When(
  "the todo {string} was added in the backend",
  (world: UiWorld, rawText: string) => {
    world.fake.addTodo(rawText);
  },
);

// ---------------------------------------------------------------------------------------------
// Bildschirmtastatur: der Visual Viewport schrumpft, die Seite bleibt gleich hoch (iOS)

class FakeViewport extends EventTarget {
  offsetTop = 0;
  scale = 1;
  constructor(public height: number) {
    super();
  }
}

let viewport: FakeViewport | undefined;

Given("the screen is {int} pixels high", (_world: UiWorld, height: number) => {
  viewport = new FakeViewport(height);
  vi.stubGlobal("innerHeight", height);
  vi.stubGlobal("visualViewport", viewport);
  Element.prototype.scrollIntoView = vi.fn();
});

When(
  "the on-screen keyboard opens with a height of {int} pixels",
  async (_world: UiWorld, keyboard: number) => {
    if (!viewport) throw new Error("Bildschirmhöhe nicht gesetzt");
    viewport.height = window.innerHeight - keyboard;
    await act(async () => {
      viewport?.dispatchEvent(new Event("resize"));
    });
  },
);

When("the on-screen keyboard closes", async () => {
  if (!viewport) throw new Error("Bildschirmhöhe nicht gesetzt");
  viewport.height = window.innerHeight;
  await act(async () => {
    viewport?.dispatchEvent(new Event("resize"));
  });
});

When("I focus the field {string}", async (_world: UiWorld, label: string) => {
  const field = await screen.findByLabelText(label);
  await act(async () => {
    field.focus();
  });
});

Then(
  "the field {string} is scrolled into view",
  async (_world: UiWorld, label: string) => {
    const field = screen.getByLabelText(label);
    await waitFor(() =>
      expect(
        vi.mocked(Element.prototype.scrollIntoView).mock.contexts,
      ).toContain(field),
    );
  },
);

// ---------------------------------------------------------------------------------------------
// Benachrichtigungen

Given(
  "the notification permission is {string}",
  (world: UiWorld, permission: string) => {
    world.fake.notificationPermission =
      permission as typeof world.fake.notificationPermission;
  },
);

Given(
  "the user answers the permission dialog with {string}",
  (world: UiWorld, answer: string) => {
    world.fake.permissionAnswer = answer as typeof world.fake.permissionAnswer;
  },
);

const androidCalls: string[] = [];

/** Android-Brücke aus `MainActivity.kt`; das Szenario setzt zusätzlich die Plattform „mobile“ (Android-Agent). */
function installAndroid(exactAlarms: boolean): void {
  androidCalls.length = 0;
  window.NuncAndroid = {
    canScheduleExactAlarms: () => exactAlarms,
    openExactAlarmSettings: () => androidCalls.push("exact"),
    openNotificationSettings: () => androidCalls.push("notifications"),
  };
}

Given("exact alarms are not allowed", () => installAndroid(false));
Given("exact alarms are allowed", () => installAndroid(true));

Then("Android opens the exact alarm settings", () => {
  expect(androidCalls).toEqual(["exact"]);
});

Then("Android opens the notification settings", () => {
  expect(androidCalls).toEqual(["notifications"]);
});

// ---------------------------------------------------------------------------------------------
// Lebenszyklus

Given(
  "the timer ran in the background and is now:",
  (world: UiWorld, table: import("quickpickle").DataTable) => {
    const row = table.hashes()[0];
    // kein Ereignis: im Hintergrund erreicht das Backend die Oberfläche nicht
    world.fake.timer = {
      state: row.state as TimerSnapshot["state"],
      mode: row.mode as TimerSnapshot["mode"],
      sessionType: row.sessionType as TimerSnapshot["sessionType"],
      remaining: Number(row.remaining),
      total: Number(row.total),
      todoId: null,
      labels: { projectTags: [], contextTags: [], note: "" },
    };
  },
);

When("the app returns to the foreground", async () => {
  vi.spyOn(document, "visibilityState", "get").mockReturnValue("visible");
  await act(async () => {
    document.dispatchEvent(new Event("visibilitychange"));
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
});

When("the app goes to the background", async () => {
  vi.spyOn(document, "visibilityState", "get").mockReturnValue("hidden");
  await act(async () => {
    document.dispatchEvent(new Event("visibilitychange"));
  });
});

When("I press the back button of the device", async (world: UiWorld) => {
  await act(async () => {
    world.history?.back();
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
});

Then("the back button would leave the app", (world: UiWorld) => {
  expect(world.history?.canGoBack()).toBe(false);
});
