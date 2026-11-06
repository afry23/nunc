import { describe, expect, it } from "vitest";
import type { DayPlan, TimelineBlock, Todo } from "./api";
import {
  closeSummary,
  fillPercent,
  nextDay,
  planRows,
  trackedMinutes,
} from "./dayPlan";

const todo = (id: number): Todo => ({
  id,
  rawText: `Task ${id}`,
  priority: null,
  completed: false,
  completedAt: null,
  createdAt: null,
  description: `Task ${id}`,
  projectTags: [],
  contextTags: [],
  pomodoroCount: 0,
  position: id,
  parentId: null,
});

const plan = (ids: number[]): DayPlan => ({
  day: "2026-03-04",
  items: ids.map((todoId) => ({
    todoId,
    reason: "planned",
    due: null,
    estimateMin: null,
    spentMin: 0,
  })),
  estimateMin: 0,
  unestimated: ids.length,
  capacityMin: 240,
  remainingMin: 240,
});

describe("nextDay", () => {
  it.each([
    ["2026-03-04", "2026-03-05"],
    ["2026-02-28", "2026-03-01"],
    ["2026-12-31", "2027-01-01"],
    ["2026-03-28", "2026-03-29"],
  ])("%s -> %s", (day, expected) => expect(nextDay(day)).toBe(expected));
});

describe("planRows", () => {
  it("keeps the order of the plan and joins the todos", () => {
    const rows = planRows(plan([3, 1]), [todo(1), todo(2), todo(3)]);
    expect(rows.map((row) => row.todo.id)).toEqual([3, 1]);
    expect(rows.map((row) => row.item.todoId)).toEqual([3, 1]);
  });

  it("drops entries whose todo is not in the list", () => {
    expect(planRows(plan([1, 9]), [todo(1)])).toHaveLength(1);
  });
});

describe("fillPercent", () => {
  it.each([
    [0, 240, 0],
    [60, 240, 25],
    [240, 240, 100],
    [480, 240, 100],
    [-5, 240, 0],
    [10, 0, 100],
    [0, 0, 0],
  ])("%i of %i -> %i", (estimate, capacity, expected) =>
    expect(fillPercent(estimate, capacity)).toBe(expected),
  );
});

describe("closeSummary", () => {
  it("counts planned entries apart from those with a due date", () => {
    const base = plan([1, 2, 3, 4]);
    const reasons = ["overdue", "due", "planned", "planned"] as const;
    const mixed = {
      ...base,
      items: base.items.map((item, index) => ({
        ...item,
        reason: reasons[index],
      })),
    };
    expect(closeSummary(mixed)).toEqual({ planned: 2, fixed: 2 });
    expect(closeSummary(plan([]))).toEqual({ planned: 0, fixed: 0 });
  });
});

describe("trackedMinutes", () => {
  const block = (
    start: string,
    end: string,
    sessionType: TimelineBlock["sessionType"] = "work",
  ): TimelineBlock => ({
    sessionId: 1,
    start,
    end,
    sessionType,
    status: "completed",
    todoId: null,
    note: "",
    origin: "timer",
    projectTags: [],
    contextTags: [],
    overlaps: false,
  });

  it("sums work blocks and ignores breaks", () => {
    expect(
      trackedMinutes([
        block("2026-03-04T09:00:00Z", "2026-03-04T09:25:00Z"),
        block("2026-03-04T09:25:00Z", "2026-03-04T09:30:00Z", "short_break"),
        block("2026-03-04T10:00:00Z", "2026-03-04T10:35:00Z"),
      ]),
    ).toBe(60);
  });

  it("counts overlapping blocks once", () => {
    expect(
      trackedMinutes([
        block("2026-03-04T09:00:00Z", "2026-03-04T10:00:00Z"),
        block("2026-03-04T09:30:00Z", "2026-03-04T10:30:00Z"),
        block("2026-03-04T09:10:00Z", "2026-03-04T09:20:00Z"),
      ]),
    ).toBe(90);
  });

  it("is zero without blocks and ignores empty spans", () => {
    expect(trackedMinutes([])).toBe(0);
    expect(
      trackedMinutes([block("2026-03-04T09:00:00Z", "2026-03-04T09:00:00Z")]),
    ).toBe(0);
  });
});
