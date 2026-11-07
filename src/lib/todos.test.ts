import { describe, expect, it } from "vitest";
import type { Todo } from "../bindings";
import {
  buildTree,
  describeImport,
  duplicateText,
  indentParent,
  outdentParent,
  collectTags,
  mergeOrder,
  moveItem,
  priorityTone,
  selectableTodos,
  dueOf,
  splitSegments,
  subtaskProgress,
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

describe("splitSegments links and fields", () => {
  it("shows the text of a markdown link", () => {
    expect(splitSegments("Video ([Watch](https://y.be/x?t=1)) +A")).toEqual([
      { text: "Video (", kind: "plain" },
      { text: "Watch", kind: "link", href: "https://y.be/x?t=1" },
      { text: ") ", kind: "plain" },
      { text: "+A", kind: "project" },
    ]);
  });
  it("shortens a bare url to its host and stops at a closing bracket", () => {
    expect(splitSegments("Kurs (https://a.example.com/p/q?x=1) @work")).toEqual(
      [
        { text: "Kurs (", kind: "plain" },
        {
          text: "a.example.com",
          kind: "link",
          href: "https://a.example.com/p/q?x=1",
        },
        { text: ") ", kind: "plain" },
        { text: "@work", kind: "context" },
      ],
    );
    expect(splitSegments("https://a.de")).toEqual([
      { text: "a.de", kind: "link", href: "https://a.de" },
    ]);
  });
  it("marks planning fields so they can be hidden", () => {
    expect(splitSegments("A due:2026-12-15 est:3")).toEqual([
      { text: "A ", kind: "plain" },
      { text: "due:2026-12-15", kind: "field" },
      { text: " ", kind: "plain" },
      { text: "est:3", kind: "field" },
    ]);
  });
});

describe("dueOf", () => {
  it("reads the due date", () => {
    expect(dueOf("A due:2026-12-15 +x")).toBe("2026-12-15");
    expect(dueOf("due:2026-12-15")).toBe("2026-12-15");
  });
  it("ignores missing or malformed dates", () => {
    expect(dueOf("A")).toBeNull();
    expect(dueOf("A due:morgen")).toBeNull();
    expect(dueOf("Adue:2026-12-15")).toBeNull();
  });
});

describe("titleOf", () => {
  it("blendet rec: aus", () => {
    expect(titleOf({ description: "Gießen rec:+1w due:2026-03-04" })).toBe(
      "Gießen",
    );
  });

  it("drops tags", () => {
    expect(titleOf({ description: "Bericht +work @desk" })).toBe("Bericht");
  });
  it("drops the planning fields of the day planner", () => {
    expect(
      titleOf({
        description: "Bericht due:2026-03-04 plan:2026-03-02 est:45 +work",
      }),
    ).toBe("Bericht");
  });
  it("keeps words that only look like planning fields", () => {
    expect(titleOf({ description: "Bericht due:morgen est:10m" })).toBe(
      "Bericht due:morgen est:10m",
    );
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

describe("selectableTodos", () => {
  const todo = (id: number, completed: boolean) =>
    ({ id, completed }) as unknown as Todo;
  const all = [todo(1, false), todo(2, true), todo(3, true)];
  it("hides completed todos", () => {
    expect(selectableTodos(all, null).map((x) => x.id)).toEqual([1]);
  });
  it("keeps the current todo even when completed", () => {
    expect(selectableTodos(all, 3).map((x) => x.id)).toEqual([1, 3]);
  });
});

describe("subtaskProgress", () => {
  const list = [
    todo({ id: 1 }),
    todo({ id: 2, parentId: 1, completed: true }),
    todo({ id: 3, parentId: 1 }),
    todo({ id: 4, parentId: 3, completed: true }),
  ];

  it("counts direct subtasks only", () => {
    expect(subtaskProgress(list, 1)).toEqual({ done: 1, total: 2 });
    expect(subtaskProgress(list, 3)).toEqual({ done: 1, total: 1 });
  });

  it("is null without subtasks", () => {
    expect(subtaskProgress(list, 2)).toBeNull();
  });
});

describe("duplicateText", () => {
  it("keeps an open todo as it is", () => {
    expect(duplicateText("(A) Bericht +work est:30")).toBe(
      "(A) Bericht +work est:30",
    );
  });

  it("drops the completion marker with one or two dates", () => {
    expect(duplicateText("x 2026-03-03 Bericht")).toBe("Bericht");
    expect(duplicateText("x 2026-03-03 2026-03-01 Bericht")).toBe("Bericht");
  });

  it("does not touch an x inside the text", () => {
    expect(duplicateText("Kiste x 2026-03-03 prüfen")).toBe(
      "Kiste x 2026-03-03 prüfen",
    );
  });
});
