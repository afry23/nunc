import {
  act,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { expect } from "vitest";
import { DataTable, Given, Then, When } from "quickpickle";
import { addDays, toIsoDate } from "../../src/lib/format";
import type { UiWorld } from "./world";

const normalize = (text: string | null): string =>
  (text ?? "").replace(/\s+/g, " ").trim();
const names = (list: string): string[] => list.split(",").map((s) => s.trim());

// ---------------------------------------------------------------------------------------------
// Timer-Ring

Then("the ring shows {string}", async (_world: UiWorld, text: string) => {
  await waitFor(() =>
    expect(screen.getByTestId("ring-time").textContent).toBe(text),
  );
});

Then(
  "the ring color is the {string} color",
  async (_world: UiWorld, token: string) => {
    await waitFor(() =>
      expect(screen.getByTestId("ring-progress").getAttribute("stroke")).toBe(
        `var(--color-${token})`,
      ),
    );
  },
);

Then("the ring is pulsing", async () => {
  await waitFor(() =>
    expect(screen.getByTestId("ring-progress").getAttribute("class")).toContain(
      "timer-pulse",
    ),
  );
});

Then("the ring is not pulsing", async () => {
  await waitFor(() =>
    expect(
      screen.getByTestId("ring-progress").getAttribute("class") ?? "",
    ).not.toContain("timer-pulse"),
  );
});

Then(
  "the screen reader announces {string}",
  async (_world: UiWorld, text: string) => {
    await waitFor(() =>
      expect(
        within(screen.getByRole("timer")).getByRole("status").textContent,
      ).toBe(text),
    );
  },
);

// ---------------------------------------------------------------------------------------------
// Aufgabenliste

const items = (): HTMLElement[] => screen.queryAllByTestId("todo-item");
const raws = (): string[] =>
  items().map((item) => item.getAttribute("data-raw") ?? "");

Then("the todo list shows {string}", async (_world: UiWorld, list: string) => {
  await waitFor(() => expect(raws()).toEqual(names(list)));
});

Then("the todo list is empty", async () => {
  await waitFor(() => expect(items()).toHaveLength(0));
});

const itemFor = (name: string): HTMLElement => {
  const found = items().find((item) =>
    (item.getAttribute("data-raw") ?? "").includes(name),
  );
  if (!found) throw new Error(`Keine Aufgabe mit "${name}" sichtbar`);
  return found;
};

Then(
  "the todo {string} shows {int} pomodoros",
  async (_world: UiWorld, name: string, count: number) => {
    await waitFor(() =>
      expect(
        within(itemFor(name)).getByLabelText(`${count} Pomodoros`),
      ).toBeTruthy(),
    );
  },
);

Then(
  "the todo {string} shows no pomodoros",
  async (_world: UiWorld, name: string) => {
    await waitFor(() =>
      expect(within(itemFor(name)).queryByLabelText(/Pomodoros$/)).toBeNull(),
    );
  },
);

/**
 * Tastatur-Ziehen über dnd-kit (Leertaste greifen, Pfeil bewegen, Leertaste ablegen). jsdom hat kein Layout:
 * jede Zeile bekommt eine feste Geometrie, damit dnd-kit die Nachbarn findet.
 */
When(
  "I drag the todo {string} below the todo {string}",
  async (_world: UiWorld, name: string, below: string) => {
    const rows = items();
    const rect = (index: number): DOMRect =>
      ({
        top: index * 60,
        bottom: index * 60 + 50,
        left: 0,
        right: 400,
        width: 400,
        height: 50,
        x: 0,
        y: index * 60,
        toJSON: () => ({}),
      }) as DOMRect;
    for (const [index, row] of rows.entries())
      row.getBoundingClientRect = () => rect(index);

    const from = rows.indexOf(itemFor(name));
    const to = rows.indexOf(itemFor(below));
    const handle = within(rows[from]).getByRole("button", {
      name: /^Verschieben:/,
    });
    await act(async () => handle.focus());
    const key = async (code: string): Promise<void> => {
      await act(async () => {
        fireEvent.keyDown(handle, { key: code, code });
        await new Promise((resolve) => setTimeout(resolve, 10));
      });
    };
    await key("Space");
    for (let i = from; i < to; i++) await key("ArrowDown");
    await key("Space");
  },
);

// ---------------------------------------------------------------------------------------------
// Statistik

function dailyFrom(world: UiWorld, table: DataTable): void {
  const today = new Date();
  const rows = table.hashes().map((row) => ({
    date: toIsoDate(addDays(today, -Number(row.daysAgo))),
    pomodoroCount: Number(row.pomodoros),
    workMinutes: Number(row.minutes),
    breakMinutes: 0,
  }));
  world.fake.stats = {
    ...world.fake.stats,
    daily: rows,
    totalPomodoros: rows.reduce((sum, r) => sum + r.pomodoroCount, 0),
    totalWorkMin: rows.reduce((sum, r) => sum + r.workMinutes, 0),
  };
}

Given(
  "the statistics contain daily data:",
  (world: UiWorld, table: DataTable) => dailyFrom(world, table),
);

Given(
  "the statistics contain the projects:",
  (world: UiWorld, table: DataTable) => {
    world.fake.stats = {
      ...world.fake.stats,
      projects: table.hashes().map((row) => ({
        project: row.project,
        workMinutes: Number(row.minutes),
      })),
    };
  },
);

Given(
  "the statistics have a current streak of {int} and a longest streak of {int}",
  (world: UiWorld, current: number, longest: number) => {
    world.fake.stats = {
      ...world.fake.stats,
      currentStreak: current,
      longestStreak: longest,
    };
  },
);

Given("the export is saved to {string}", (world: UiWorld, path: string) => {
  world.fake.exportPath = path;
});

Then(
  "the statistics were requested for the period {string}",
  async (world: UiWorld, period: string) => {
    await waitFor(() =>
      expect(
        (world.fake.last("get_stats")?.request as { period: string }).period,
      ).toBe(period),
    );
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
    });
  },
);

Then(
  "the statistic {string} is {string}",
  async (_world: UiWorld, label: string, value: string) => {
    await waitFor(() => {
      const term = screen.getByText(label, { selector: "dt" });
      expect(normalize(term.nextElementSibling?.textContent ?? null)).toBe(
        value,
      );
    });
  },
);

Then("I see the charts {string}", async (_world: UiWorld, list: string) => {
  for (const chart of names(list)) await screen.findByTestId(`chart-${chart}`);
});

Then("I do not see the charts {string}", (_world: UiWorld, list: string) => {
  for (const chart of names(list))
    expect(screen.queryByTestId(`chart-${chart}`)).toBeNull();
});

Given(
  "the statistics count {int} pomodoros over all time",
  (world: UiWorld, count: number) => {
    world.fake.stats = { ...world.fake.stats, totalPomodoros: count };
  },
);

Given(
  "the statistics contain monthly data:",
  (world: UiWorld, table: DataTable) => {
    world.fake.stats = {
      ...world.fake.stats,
      monthly: table.hashes().map((row) => ({
        year: Number(row.year),
        month: Number(row.month),
        pomodoroCount: Number(row.pomodoros),
        workMinutes: Number(row.minutes),
      })),
    };
  },
);

Then(
  "the month {string} shows {int} pomodoros",
  async (_world: UiWorld, month: string, count: number) => {
    await waitFor(() => {
      const row = within(
        screen.getByRole("table", { name: "Pomodoros pro Monat" }),
      ).getByRole("row", {
        name: new RegExp(`^${month}`),
      });
      expect(within(row).getAllByRole("cell")[0].textContent).toBe(
        String(count),
      );
    });
  },
);

Then("I do not see the monthly table", () => {
  expect(
    screen.queryByRole("table", { name: "Pomodoros pro Monat" }),
  ).toBeNull();
});

Then(
  "the line chart labels the days with {string}",
  async (_world: UiWorld, list: string) => {
    await waitFor(() => {
      const labels = within(screen.getByTestId("chart-line"))
        .getAllByTestId("line-value")
        .map((node) => node.textContent);
      // die letzten Tage der Woche: Reihenfolge ab dem ältesten sichtbaren Tag
      expect(labels.slice(-names(list).length)).toEqual(names(list));
    });
  },
);
