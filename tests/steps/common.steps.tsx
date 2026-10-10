import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { emit } from "@tauri-apps/api/event";
import { createMemoryHistory } from "@tanstack/react-router";
import { expect, vi } from "vitest";
import { DataTable, Given, Then, When } from "quickpickle";
import App from "../../src/App";
import type { TimerMode, TimerSnapshot } from "../../src/bindings";
import { commands } from "../../src/lib/api";
import { mockBackend } from "../helpers/ipc";
import type { UiWorld } from "./world";

// ---------------------------------------------------------------------------------------------
// Hilfen

const normalize = (text: string | null): string =>
  (text ?? "").replace(/\s+/g, " ").trim();

const names = (list: string): string[] => list.split(",").map((s) => s.trim());

/** Wartet, bis die App die Anfangsdaten geladen hat. */
async function settle(world: UiWorld, extra: string[] = []): Promise<void> {
  const needed = [
    "get_settings",
    "get_todos",
    "get_timer_tick",
    "get_suggested_break",
    ...extra,
  ];
  await waitFor(() =>
    needed.forEach((cmd) => expect(world.fake.count(cmd)).toBeGreaterThan(0)),
  );
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
}

async function openApp(world: UiWorld, path: string): Promise<void> {
  const history = createMemoryHistory({ initialEntries: [path] });
  render(<App history={history} />);
  world.history = history;
  world.rendered = true;
  if (path === "/mini")
    await screen.findByRole("heading", { name: "Mini-Timer" });
  else await screen.findByRole("main");
  await settle(world, path === "/stats" ? ["get_stats"] : []);
}

function parseValue(current: unknown, text: string): unknown {
  if (typeof current === "boolean") return text === "true";
  if (typeof current === "number") return Number(text);
  return text;
}

const KEYS: Record<string, string> = { Space: " " };

async function press(key: string, target: Element | Document): Promise<void> {
  await act(async () => {
    fireEvent.keyDown(target, { key: KEYS[key] ?? key });
    if (key === "Enter" && target instanceof HTMLInputElement && target.form)
      fireEvent.submit(target.form);
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
}

function clickable(name: string): HTMLElement {
  for (const role of ["button", "link", "radio"] as const) {
    const found = screen.queryByRole(role, { name });
    if (found) return found;
  }
  throw new Error(`Kein Button, Link oder Radio "${name}" gefunden`);
}

// ---------------------------------------------------------------------------------------------
// Backend

Given(
  "the backend reports version {string}",
  (world: UiWorld, version: string) => {
    world.backend = mockBackend({
      app_info: () => ({ name: "nunc", version }),
    });
  },
);

Given(
  "the setting {string} is {string}",
  (world: UiWorld, key: string, value: string) => {
    const settings = world.fake.settings as Record<string, unknown>;
    settings[key] = parseValue(settings[key], value);
  },
);

Given("the todos:", (world: UiWorld, table: DataTable) => {
  for (const row of table.hashes()) {
    world.fake.addTodo(row.rawText, {
      pomodoroCount: Number(row.pomodoroCount ?? 0),
      completed: row.completed === "true",
    });
  }
});

Given(
  "the backend suggests a break of {int} minutes",
  (world: UiWorld, minutes: number) => {
    world.fake.suggestedBreak = { sessionType: "short_break", minutes };
  },
);

function snapshotOf(row: Record<string, string>): TimerSnapshot {
  return {
    state: row.state as TimerSnapshot["state"],
    mode: row.mode as TimerMode,
    sessionType: row.sessionType as TimerSnapshot["sessionType"],
    remaining: Number(row.remaining),
    total: Number(row.total),
    todoId: null,
  };
}

/** Vor dem Öffnen der App: Ausgangszustand des Backends. Danach: Timer-Ereignis des Backends. */
Given(
  "the backend reports the timer as:",
  async (world: UiWorld, table: DataTable) => {
    const snapshot = snapshotOf(table.hashes()[0]);
    if (!world.rendered) {
      world.fake.timer = snapshot;
      return;
    }
    world.fake.timer = snapshot;
    await act(async () => {
      await emit("timer:tick", snapshot);
    });
  },
);

When(
  "the backend reports a finished {string} session with a suggested break of {int} minutes",
  async (_world: UiWorld, mode: string, minutes: number) => {
    await act(async () => {
      await emit("timer:complete", {
        session: {
          id: 1,
          mode,
          sessionType: "work",
          status: "completed",
          startedAt: "2026-10-04T09:00:00Z",
          endedAt: "2026-10-04T09:25:00Z",
          plannedMin: mode === "pomodoro" ? 25 : null,
          actualMin: 25,
          todoId: null,
          breakMin: null,
        },
        suggestedBreak: { sessionType: "short_break", minutes },
      });
    });
  },
);

When(
  "the backend reports a finished {string} session without a suggested break",
  async (_world: UiWorld, mode: string) => {
    await act(async () => {
      await emit("timer:stopped", {
        session: {
          id: 1,
          mode,
          sessionType: "work",
          status: "completed",
          startedAt: "2026-10-04T09:00:00Z",
          endedAt: "2026-10-04T10:00:00Z",
          plannedMin: null,
          actualMin: 60,
          todoId: null,
          breakMin: null,
        },
        suggestedBreak: null,
      });
    });
  },
);

Given("the viewport is {int} pixels wide", (_world: UiWorld, width: number) => {
  window.matchMedia = ((query: string) => {
    const min = /min-width:\s*(\d+)px/.exec(query)?.[1];
    return {
      matches: min ? width >= Number(min) : false,
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    };
  }) as unknown as typeof window.matchMedia;
});

Given("the platform is {string}", (_world: UiWorld, platform: string) => {
  const agent =
    platform === "mobile"
      ? "Mozilla/5.0 (Linux; Android 15; Pixel) AppleWebKit/537.36"
      : "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15";
  vi.spyOn(navigator, "userAgent", "get").mockReturnValue(agent);
});

When("I ask the backend for the app info", async (world: UiWorld) => {
  world.result = await commands.appInfo();
});

Then("the app info version is {string}", (world: UiWorld, version: string) => {
  expect(world.result).toMatchObject({ version });
});

Then("the backend received {string}", async (world: UiWorld, cmd: string) => {
  await waitFor(() => expect(world.fake.count(cmd)).toBeGreaterThan(0));
});

Then("the backend did not receive {string}", (world: UiWorld, cmd: string) => {
  expect(world.fake.count(cmd)).toBe(0);
});

Then(
  "the backend received {string} with:",
  async (world: UiWorld, cmd: string, expected: string) => {
    await waitFor(() =>
      expect(world.fake.last(cmd)).toMatchObject(JSON.parse(String(expected))),
    );
  },
);

// ---------------------------------------------------------------------------------------------
// App und Navigation

When("I open the app", async (world: UiWorld) => openApp(world, "/"));
When("I open the app at {string}", async (world: UiWorld, path: string) =>
  openApp(world, path),
);

Then("I see the heading {string}", async (_world: UiWorld, text: string) => {
  await screen.findByRole("heading", { name: text });
});

Then(
  "I see a navigation with the entries {string}",
  async (_world: UiWorld, entries: string) => {
    const nav = await screen.findByRole("navigation", {
      name: "Hauptnavigation",
    });
    expect(
      within(nav)
        .getAllByRole("link")
        .map((link) => normalize(link.textContent)),
    ).toEqual(names(entries));
  },
);

Then("I do not see a navigation", () => {
  expect(screen.queryByRole("navigation")).toBeNull();
});

Then(
  "the entry {string} is marked as current",
  async (_world: UiWorld, name: string) => {
    const nav = await screen.findByRole("navigation", {
      name: "Hauptnavigation",
    });
    await waitFor(() =>
      expect(within(nav).getByRole("link", { name })).toHaveProperty(
        "ariaCurrent",
        "page",
      ),
    );
  },
);

Then(
  "the entry {string} is not marked as current",
  (_world: UiWorld, name: string) => {
    const nav = screen.getByRole("navigation", { name: "Hauptnavigation" });
    expect(
      within(nav).getByRole("link", { name }).getAttribute("aria-current"),
    ).toBeNull();
  },
);

Given("the sidebar was collapsed earlier", () => {
  localStorage.setItem("nunc.sidebarCollapsed", "1");
});

Then("the sidebar is collapsed", async () => {
  await waitFor(() =>
    expect(screen.getByTestId("sidebar").getAttribute("data-collapsed")).toBe(
      "true",
    ),
  );
});

Then("the sidebar is expanded", async () => {
  await waitFor(() =>
    expect(screen.getByTestId("sidebar").getAttribute("data-collapsed")).toBe(
      "false",
    ),
  );
});

Then(
  "the navigation still offers {string}",
  (_world: UiWorld, entries: string) => {
    const nav = screen.getByRole("navigation", { name: "Hauptnavigation" });
    expect(
      within(nav)
        .getAllByRole("link")
        .map((link) => normalize(link.textContent)),
    ).toEqual(names(entries));
  },
);

Then("I see a sidebar", () => {
  expect(screen.getByTestId("sidebar")).toBeTruthy();
});
Then("I do not see a sidebar", () => {
  expect(screen.queryByTestId("sidebar")).toBeNull();
});
Then("I see a bottom tab bar", () => {
  expect(screen.getByTestId("tabbar")).toBeTruthy();
});
Then("I do not see a bottom tab bar", () => {
  expect(screen.queryByTestId("tabbar")).toBeNull();
});

// ---------------------------------------------------------------------------------------------
// Bedienung

When("I click {string}", async (_world: UiWorld, name: string) => {
  await waitFor(() => clickable(name));
  await act(async () => {
    fireEvent.click(clickable(name));
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
});

When("I check {string}", async (_world: UiWorld, name: string) => {
  const box = await screen.findByRole("checkbox", { name });
  await act(async () => {
    fireEvent.click(box);
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
});

async function setSwitch(name: string, on: boolean): Promise<void> {
  const toggle = await screen.findByRole("switch", { name });
  if ((toggle.getAttribute("aria-checked") === "true") !== on) {
    await act(async () => {
      fireEvent.click(toggle);
    });
  }
}
When("I switch on {string}", async (_world: UiWorld, name: string) =>
  setSwitch(name, true),
);
When("I switch off {string}", async (_world: UiWorld, name: string) =>
  setSwitch(name, false),
);

When(
  "I enter {string} into the field {string}",
  async (_world: UiWorld, value: string, label: string) => {
    const field = await screen.findByLabelText(label);
    await act(async () => {
      fireEvent.change(field, { target: { value } });
      await new Promise((resolve) => setTimeout(resolve, 0));
    });
  },
);

When(
  "I select {string} in the field {string}",
  async (_world: UiWorld, option: string, label: string) => {
    const select = (await screen.findByLabelText(label)) as HTMLSelectElement;
    await waitFor(() =>
      expect(
        Array.from(select.options).some((o) =>
          normalize(o.textContent).startsWith(option),
        ),
      ).toBe(true),
    );
    const chosen = Array.from(select.options).find((o) =>
      normalize(o.textContent).startsWith(option),
    );
    await act(async () => {
      fireEvent.change(select, { target: { value: chosen?.value } });
    });
  },
);

When(
  "I set the slider {string} to {string}",
  async (_world: UiWorld, label: string, value: string) => {
    const slider = await screen.findByLabelText(label);
    await act(async () => {
      fireEvent.change(slider, { target: { value } });
    });
  },
);

When("I press the key {string}", async (_world: UiWorld, key: string) =>
  press(key, document.body),
);

When(
  "I press the key {string} in the field {string}",
  async (_world: UiWorld, key: string, label: string) => {
    press(key, await screen.findByLabelText(label));
  },
);

// ---------------------------------------------------------------------------------------------
// Anzeige

Then("I see {string}", async (_world: UiWorld, text: string) => {
  await waitFor(() =>
    expect(normalize(document.body.textContent)).toContain(text),
  );
});

Then("I do not see {string}", (_world: UiWorld, text: string) => {
  expect(normalize(document.body.textContent)).not.toContain(text);
});

Then("I see the buttons {string}", async (_world: UiWorld, list: string) => {
  for (const name of names(list)) await screen.findByRole("button", { name });
});

Then("I do not see the button {string}", (_world: UiWorld, name: string) => {
  expect(screen.queryByRole("button", { name })).toBeNull();
});

Then(
  "I see a dialog with the text {string}",
  async (_world: UiWorld, text: string) => {
    const dialog = await screen.findByRole("dialog");
    expect(normalize(dialog.textContent)).toContain(text);
  },
);

Then("I do not see a dialog", async () => {
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});

Then(
  "the field {string} has the focus",
  async (_world: UiWorld, label: string) => {
    await waitFor(() =>
      expect(document.activeElement).toBe(screen.getByLabelText(label)),
    );
  },
);

Then("the field {string} does not exist", (_world: UiWorld, label: string) => {
  expect(screen.queryByLabelText(label)).toBeNull();
});

Then("the field {string} is empty", (_world: UiWorld, label: string) => {
  expect((screen.getByLabelText(label) as HTMLInputElement).value).toBe("");
});

Then(
  "the field {string} accepts values from {int} to {int}",
  (_world: UiWorld, label: string, min: number, max: number) => {
    const field = screen.getByLabelText(label);
    expect([field.getAttribute("min"), field.getAttribute("max")]).toEqual([
      String(min),
      String(max),
    ]);
  },
);

Then(
  "the slider {string} ranges from {float} to {float} in steps of {float}",
  (_world: UiWorld, label: string, min: number, max: number, step: number) => {
    const slider = screen.getByLabelText(label);
    expect(slider.getAttribute("type")).toBe("range");
    expect([
      Number(slider.getAttribute("min")),
      Number(slider.getAttribute("max")),
      Number(slider.getAttribute("step")),
    ]).toEqual([min, max, step]);
  },
);

Then("the slider {string} is disabled", (_world: UiWorld, label: string) => {
  expect((screen.getByLabelText(label) as HTMLInputElement).disabled).toBe(
    true,
  );
});

Then(
  "the ratio badge shows {string}",
  async (_world: UiWorld, text: string) => {
    await waitFor(() =>
      expect(screen.getByTestId("ratio-badge").textContent).toBe(text),
    );
  },
);
