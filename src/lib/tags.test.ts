import { describe, expect, it } from "vitest";
import {
  addTag,
  matchingSuggestions,
  removeTag,
  tagsOfLine,
  toTag,
  withoutTags,
  withTags,
} from "./tags";

describe("toTag", () => {
  it("adds the prefix when it is missing", () => {
    expect(toTag("meetings", "+")).toBe("+meetings");
    expect(toTag(" office ", "@")).toBe("@office");
  });

  it("keeps an existing prefix", () => {
    expect(toTag("+meetings", "+")).toBe("+meetings");
  });

  it("rejects empty input, a bare prefix and input with whitespace", () => {
    expect(toTag("", "+")).toBeNull();
    expect(toTag("   ", "+")).toBeNull();
    expect(toTag("+", "+")).toBeNull();
    expect(toTag("two words", "@")).toBeNull();
  });

  it("treats the other prefix as part of the name", () => {
    expect(toTag("@office", "+")).toBe("+@office");
  });
});

describe("addTag and removeTag", () => {
  it("appends new tags in order and ignores duplicates", () => {
    expect(addTag(["+a"], "b", "+")).toEqual(["+a", "+b"]);
    expect(addTag(["+a"], "+a", "+")).toEqual(["+a"]);
  });

  it("ignores invalid input", () => {
    const tags = ["+a"];
    expect(addTag(tags, "+", "+")).toBe(tags);
  });

  it("removes a tag", () => {
    expect(removeTag(["+a", "+b"], "+a")).toEqual(["+b"]);
    expect(removeTag(["+a"], "+x")).toEqual(["+a"]);
  });
});

describe("matchingSuggestions", () => {
  const all = ["+meetings", "+Team", "+work"];

  it("offers everything for an empty input except chosen tags", () => {
    expect(matchingSuggestions(all, ["+work"], "")).toEqual([
      "+meetings",
      "+Team",
    ]);
  });

  it("filters by the name, ignoring case and a typed prefix", () => {
    expect(matchingSuggestions(all, [], "tea")).toEqual(["+Team"]);
    expect(matchingSuggestions(all, [], "+MEET")).toEqual(["+meetings"]);
    expect(matchingSuggestions(all, [], "zzz")).toEqual([]);
  });
});

describe("Tags in der todo.txt-Zeile", () => {
  it("liest Projekte und Kontexte ohne Duplikate", () => {
    expect(tagsOfLine("(A) Bericht +work @office +work + @")).toEqual({
      projects: ["+work"],
      contexts: ["@office"],
    });
  });
  it("entfernt Tags und lässt den Rest wortgleich", () => {
    expect(withoutTags("(A) Bericht +work @office plan:2026-11-06")).toBe(
      "(A) Bericht plan:2026-11-06",
    );
    expect(withoutTags("Mail a@b.de + x")).toBe("Mail a@b.de + x");
  });
  it("setzt genau die gewählten Tags ans Ende, ohne Duplikate", () => {
    expect(withTags("Bericht", ["+work"], ["@office"])).toBe(
      "Bericht +work @office",
    );
    expect(withTags("Bericht +work @a", ["+work", "+home"], [])).toBe(
      "Bericht +work +home @a",
    );
    expect(withTags("Bericht +work", [], [])).toBe("Bericht +work");
    expect(withTags("", ["+a"], [])).toBe("+a");
  });
});
