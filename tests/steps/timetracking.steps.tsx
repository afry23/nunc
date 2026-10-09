import {
  act,
  fireEvent,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { expect } from "vitest";
import { DataTable, Given, Then, When } from "quickpickle";
import type { TimelineBlock } from "../../src/bindings";
import { addDays, toIsoDate } from "../../src/lib/format";
import type { UiWorld } from "./world";

const normalize = (text: string | null): string =>
  (text ?? "").replace(/\s+/g, " ").trim();

const DAYS = ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"];

/** Montag der aktuellen Woche (lokal), unabhängig von `src/lib/timesheet.ts`. */
function thisMonday(): Date {
  const today = new Date();
  return addDays(today, -((today.getDay() + 6) % 7));
}

function at(date: string, time: string): string {
  const [year, month, day] = date.split("-").map(Number);
  const [hour, minute] = time.split(":").map(Number);
  return new Date(year, month - 1, day, hour, minute).toISOString();
}

function blocksFrom(
  world: UiWorld,
  date: string,
  table: DataTable,
): TimelineBlock[] {
  return table.hashes().map((row, index) => ({
    sessionId: index + 1,
    start: at(date, row.start),
    end: at(date, row.end),
    sessionType: row.type as TimelineBlock["sessionType"],
    status: "completed",
    todoId:
      world.fake.todos.find((todo) => todo.description === row.todo)?.id ??
      null,
    note: row.note ?? "",
    origin: row.origin || "timer",
    overlaps: false,
  }));
}

Given(
  "the timesheet grouped by {string} contains:",
  (world: UiWorld, groupBy: string, table: DataTable) => {
    world.fake.timesheet[groupBy as "project"] = table.hashes().map((row) => ({
      group: row.group === "" ? null : row.group,
      minutes: DAYS.map((day) => Number(row[day])),
    }));
  },
);

Given(
  "the timeline of today has the sessions:",
  (world: UiWorld, table: DataTable) => {
    const today = toIsoDate(new Date());
    world.fake.timeline[today] = blocksFrom(world, today, table);
  },
);

Given(
  "the timeline of the weekday {string} has the sessions:",
  (world: UiWorld, weekday: string, table: DataTable) => {
    const date = toIsoDate(addDays(thisMonday(), DAYS.indexOf(weekday)));
    world.fake.timeline[date] = blocksFrom(world, date, table);
  },
);

Given(
  "saving a session fails with {string}: {string}",
  (world: UiWorld, code: string, message: string) => {
    world.fake.sessionError = { code, message };
  },
);

When("I choose the weekday {string}", async (_world: UiWorld, day: string) => {
  const button = await screen.findByRole("button", {
    name: new RegExp(`^${day} \\d\\d\\.\\d\\d\\.$`),
  });
  await act(async () => {
    fireEvent.click(button);
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
});

When(
  "I enter {string} for today into the field {string}",
  async (_world: UiWorld, time: string, label: string) => {
    const field = await screen.findByLabelText(label);
    await act(async () => {
      fireEvent.change(field, {
        target: { value: `${toIsoDate(new Date())}T${time}` },
      });
    });
  },
);

Then("the timesheet shows:", async (_world: UiWorld, table: DataTable) => {
  const grid = await screen.findByRole("table", {
    name: "Arbeitszeit pro Woche",
  });
  const expected = table.raw().slice(1);
  await waitFor(() => {
    const rows = within(grid)
      .getAllByRole("row")
      .slice(1)
      .map((row) =>
        Array.from(row.querySelectorAll("th, td")).map((cell) =>
          normalize(cell.textContent),
        ),
      );
    expect(rows).toEqual(expected);
  });
});

Then(
  "the timesheet was requested grouped by {string}",
  async (world: UiWorld, groupBy: string) => {
    await waitFor(() =>
      expect(world.fake.last("get_timesheet")?.groupBy).toBe(groupBy),
    );
  },
);

Then(
  "the week shown starts {int} weeks from the current week",
  async (world: UiWorld, weeks: number) => {
    const start = toIsoDate(addDays(thisMonday(), 7 * weeks));
    const end = toIsoDate(addDays(thisMonday(), 7 * weeks + 6));
    await waitFor(() => {
      const request = world.fake.last("get_timesheet");
      expect([request?.startDate, request?.endDate]).toEqual([start, end]);
    });
  },
);

Then(
  "the field {string} shows today at {string}",
  async (_world: UiWorld, label: string, time: string) => {
    const field = (await screen.findByLabelText(label)) as HTMLInputElement;
    expect(field.value).toBe(`${toIsoDate(new Date())}T${time}`);
  },
);

Then(
  "the last {string} call has {string} equal to {string}",
  async (world: UiWorld, cmd: string, path: string, value: string) => {
    await waitFor(() => {
      const found = path
        .split(".")
        .reduce<unknown>(
          (node, key) => (node as Record<string, unknown> | undefined)?.[key],
          world.fake.last(cmd),
        );
      expect(String(found)).toBe(value);
    });
  },
);
