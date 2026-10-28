import { describe, expect, it } from "vitest";
import type { Todo } from "../bindings";
import {
  buildTree,
  describeImport,
  indentParent,
  outdentParent,
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
  parentId: null,
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

describe("describeImport", () => {
  it("nennt neue Aufgaben und Duplikate in der richtigen Zahl", () => {
    expect(describeImport({ added: 3, duplicates: 1 })).toBe(
      "3 Aufgaben importiert, 1 Duplikat übersprungen.",
    );
    expect(describeImport({ added: 1, duplicates: 2 })).toBe(
      "1 Aufgabe importiert, 2 Duplikate übersprungen.",
    );
  });

  it("lässt die Duplikate weg, wenn es keine gab", () => {
    expect(describeImport({ added: 2, duplicates: 0 })).toBe(
      "2 Aufgaben importiert.",
    );
  });

  it("meldet eine Datei ohne neue Zeilen", () => {
    expect(describeImport({ added: 0, duplicates: 0 })).toBe(
      "Die Datei enthält keine neuen Aufgaben.",
    );
    expect(describeImport({ added: 0, duplicates: 4 })).toBe(
      "0 Aufgaben importiert, 4 Duplikate übersprungen.",
    );
  });
});

describe("buildTree", () => {
  const list = [
    todo({ id: 1, pomodoroCount: 1 }),
    todo({ id: 2, parentId: 1, pomodoroCount: 2 }),
    todo({ id: 3, parentId: 2, pomodoroCount: 4 }),
    todo({ id: 4 }),
    todo({ id: 5, parentId: 99 }),
    todo({ id: 6, parentId: 1 }),
  ];
  it("puts children under their parent and keeps sibling order", () => {
    const rows = buildTree(list);
    expect(rows.map((r) => [r.todo.id, r.depth])).toEqual([
      [1, 0],
      [2, 1],
      [3, 2],
      [6, 1],
      [4, 0],
      [5, 0],
    ]);
    expect(rows[0].hasChildren).toBe(true);
    expect(rows[3].hasChildren).toBe(false);
    expect(rows[2].parentIndex).toBe(1);
    expect(rows[4].parentIndex).toBeNull();
  });
  it("sums pomodoros over the subtree, also when collapsed", () => {
    const rows = buildTree(list, new Set([1]));
    expect(rows.map((r) => r.todo.id)).toEqual([1, 4, 5]);
    expect(rows[0].pomodoroTotal).toBe(7);
  });
  it("survives a cycle in the input", () => {
    const rows = buildTree([
      todo({ id: 1, parentId: 2, pomodoroCount: 1 }),
      todo({ id: 2, parentId: 1 }),
    ]);
    expect(rows).toEqual([]);
  });
});

describe("indent and outdent", () => {
  const rows = buildTree([
    todo({ id: 1 }),
    todo({ id: 2, parentId: 1 }),
    todo({ id: 3, parentId: 2 }),
    todo({ id: 4, parentId: 1 }),
    todo({ id: 5 }),
  ]);
  it("indents under the previous sibling", () => {
    expect(indentParent(rows, 3)).toBe(2); // 4 unter 2
    expect(indentParent(rows, 4)).toBe(1); // 5 unter 1
  });
  it("cannot indent the first sibling", () => {
    expect(indentParent(rows, 0)).toBeNull();
    expect(indentParent(rows, 1)).toBeNull();
    expect(indentParent(rows, 99)).toBeNull();
  });
  it("outdents to the grandparent or the root", () => {
    expect(outdentParent(rows, 2)).toBe(1);
    expect(outdentParent(rows, 1)).toBeNull();
    expect(outdentParent(rows, 0)).toBeUndefined();
  });
});
