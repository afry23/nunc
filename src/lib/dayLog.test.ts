import { describe, expect, it } from "vitest";
import type { TimelineBlock, Todo } from "./api";
import { blockMinutes, groupBlocks } from "./dayLog";

const block = (
  id: number,
  start: string,
  end: string,
  over: Partial<TimelineBlock> = {},
): TimelineBlock => ({
  sessionId: id,
  start: `2026-03-04T${start}:00Z`,
  end: `2026-03-04T${end}:00Z`,
  sessionType: "work",
  status: "completed",
  todoId: null,
  note: "",
  origin: "timer",
  projectTags: [],
  contextTags: [],
  overlaps: false,
  ...over,
});

const todo = (
  id: number,
  description: string,
  over: Partial<Todo> = {},
): Todo => ({
  id,
  rawText: description,
  priority: null,
  completed: false,
  completedAt: null,
  createdAt: null,
  description,
  projectTags: [],
  contextTags: [],
  pomodoroCount: 0,
  position: 0,
  parentId: null,
  ...over,
});

const labels = (groups: ReturnType<typeof groupBlocks>) =>
  groups.map((group) => [group.label, group.minutes]);

describe("blockMinutes", () => {
  it("rounds to minutes and never goes below zero", () => {
    expect(blockMinutes(block(1, "09:00", "09:25"))).toBe(25);
    expect(blockMinutes(block(1, "09:30", "09:00"))).toBe(0);
  });
});

describe("groupBlocks", () => {
  const blocks = [
    block(1, "09:00", "10:00", { projectTags: ["+Produkt"] }),
    block(2, "10:00", "10:30", {
      projectTags: ["+Plattform"],
      contextTags: ["@support"],
    }),
    block(3, "11:00", "11:15"),
    block(4, "12:00", "12:05", { sessionType: "short_break" }),
  ];

  it("groups by project, biggest first, unassigned last, breaks ignored", () => {
    expect(labels(groupBlocks(blocks, [], "project"))).toEqual([
      ["+Produkt", 60],
      ["+Plattform", 30],
      [null, 15],
    ]);
  });

  it("groups by context", () => {
    expect(labels(groupBlocks(blocks, [], "context"))).toEqual([
      ["@support", 30],
      [null, 75],
    ]);
  });

  it("uses the tags of the task instead of the own ones", () => {
    const tasks = [todo(7, "Bericht +work", { projectTags: ["+work"] })];
    const withTask = [
      block(1, "09:00", "09:30", { todoId: 7, projectTags: ["+ignored"] }),
    ];
    expect(labels(groupBlocks(withTask, tasks, "project"))).toEqual([
      ["+work", 30],
    ]);
  });

  it("falls back to the own tags when the task is gone", () => {
    const orphan = [
      block(1, "09:00", "09:30", { todoId: 9, projectTags: ["+own"] }),
    ];
    expect(labels(groupBlocks(orphan, [], "project"))).toEqual([["+own", 30]]);
  });

  it("puts a block with several tags into each group, counting a tag once", () => {
    const many = [
      block(1, "09:00", "10:00", { projectTags: ["+a", "+b", "+a"] }),
    ];
    expect(labels(groupBlocks(many, [], "project"))).toEqual([
      ["+a", 60],
      ["+b", 60],
    ]);
  });

  it("groups by task with a group for work without a task", () => {
    const tasks = [todo(7, "Bericht +work")];
    const mixed = [
      block(1, "09:00", "09:30", { todoId: 7 }),
      block(2, "10:00", "10:15", { todoId: 7 }),
      block(3, "11:00", "11:10"),
    ];
    const groups = groupBlocks(mixed, tasks, "todo");
    expect(labels(groups)).toEqual([
      ["Bericht", 45],
      [null, 10],
    ]);
    expect(groups[0].blocks.map((b) => b.sessionId)).toEqual([1, 2]);
  });

  it("orders equal minutes by name", () => {
    const equal = [
      block(1, "09:00", "09:30", { projectTags: ["+b"] }),
      block(2, "10:00", "10:30", { projectTags: ["+a"] }),
    ];
    expect(labels(groupBlocks(equal, [], "project"))).toEqual([
      ["+a", 30],
      ["+b", 30],
    ]);
  });
});
