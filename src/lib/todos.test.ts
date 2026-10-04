import { describe, expect, it } from "vitest";
import type { Todo } from "../bindings";
import {
  collectTags,
  mergeOrder,
  moveItem,
  priorityTone,
  splitSegments,
  titleOf,
} from "./todos";

const todo = (over: Partial<Todo>): Todo => ({
  id: 1,
  rawText: "",
  priority: null,
  completed: false,
  completedAt: null,
  createdAt: null,
  description: "",
  projectTags: [],
  contextTags: [],
  pomodoroCount: 0,
  position: 0,
  ...over,
});

describe("splitSegments", () => {
  it("marks tags and merges plain text", () => {
    expect(splitSegments("Bericht +work an @desk senden")).toEqual([
      { text: "Bericht ", kind: "plain" },
      { text: "+work", kind: "project" },
      { text: " an ", kind: "plain" },
      { text: "@desk", kind: "context" },
      { text: " senden", kind: "plain" },
    ]);
  });
  it("keeps lone prefixes as text and handles empty input", () => {
    expect(splitSegments("a + b @")).toEqual([
      { text: "a + b @", kind: "plain" },
    ]);
    expect(splitSegments("")).toEqual([]);
  });
});

describe("titleOf", () => {
  it("drops tags", () => {
    expect(titleOf({ description: "Bericht +work @desk" })).toBe("Bericht");
  });
  it("falls back to the description when only tags remain", () => {
    expect(titleOf({ description: "+work" })).toBe("+work");
  });
});

describe("collectTags", () => {
  it("is sorted and unique", () => {
    const tags = collectTags([
      todo({ projectTags: ["+b", "+a"], contextTags: ["@x"] }),
      todo({ projectTags: ["+a"], contextTags: ["@x", "@c"] }),
    ]);
    expect(tags).toEqual({ projects: ["+a", "+b"], contexts: ["@c", "@x"] });
  });
});

describe("mergeOrder", () => {
  it("keeps hidden todos in place and reorders the visible ones", () => {
    // 2 und 4 sind ausgeblendet (z. B. erledigt)
    expect(mergeOrder([1, 2, 3, 4, 5], [5, 1, 3])).toEqual([5, 2, 1, 4, 3]);
  });
  it("is the identity for an unchanged order", () => {
    expect(mergeOrder([1, 2, 3], [1, 2, 3])).toEqual([1, 2, 3]);
  });
});

describe("moveItem", () => {
  it("moves an element", () => {
    expect(moveItem(["a", "b", "c"], 0, 2)).toEqual(["b", "c", "a"]);
    expect(moveItem(["a", "b", "c"], 2, 0)).toEqual(["c", "a", "b"]);
  });
  it("ignores invalid moves", () => {
    const items = ["a", "b"];
    expect(moveItem(items, 0, 0)).toBe(items);
    expect(moveItem(items, -1, 1)).toBe(items);
    expect(moveItem(items, 0, 2)).toBe(items);
    expect(moveItem(items, 2, 0)).toBe(items);
  });
});

describe("priorityTone", () => {
  it("maps priorities", () => {
    expect(priorityTone("A")).toBe("high");
    expect(priorityTone("B")).toBe("medium");
    expect(priorityTone("C")).toBe("low");
    expect(priorityTone(null)).toBe("none");
  });
});
