import { describe, expect, it } from "vitest";
import {
  changedData,
  describeSyncResult,
  isHttpUrl,
  isWebDavComplete,
} from "./sync";

describe("isHttpUrl", () => {
  it("accepts http and https addresses with a host", () => {
    for (const ok of [
      "https://cloud.example.org/remote.php/dav",
      "http://192.168.1.2:8080",
      "  HTTPS://a.b  ",
    ])
      expect(isHttpUrl(ok), ok).toBe(true);
  });
  it("rejects everything else", () => {
    for (const bad of [
      "",
      "cloud.example.org",
      "ftp://x.org",
      "https://",
      "https:///pfad",
      "https://a b/",
      "http://:80",
      "https:/x.org",
    ])
      expect(isHttpUrl(bad), bad).toBe(false);
  });
});

describe("isWebDavComplete", () => {
  const base = {
    url: "https://x.org",
    authMethod: "basic" as const,
    username: "me",
    remotePath: "/nunc/",
  };
  it("needs a valid address and a user", () => {
    expect(isWebDavComplete(base)).toBe(true);
    expect(isWebDavComplete({ ...base, url: "x.org" })).toBe(false);
    expect(isWebDavComplete({ ...base, username: "  " })).toBe(false);
  });
});

describe("describeSyncResult", () => {
  const none = { importedTodos: 0, importedSessions: 0, conflictedTodos: 0 };
  it("is silent when nothing changed", () => {
    expect(describeSyncResult(none)).toBeNull();
    expect(changedData(none)).toBe(false);
  });
  it("names imported sessions and todos with singular and plural", () => {
    expect(
      describeSyncResult({ ...none, importedSessions: 2, importedTodos: 3 }),
    ).toBe("2 Sessions und 3 Aufgaben importiert.");
    expect(
      describeSyncResult({ ...none, importedSessions: 1, importedTodos: 1 }),
    ).toBe("1 Session und 1 Aufgabe importiert.");
    expect(describeSyncResult({ ...none, importedTodos: 4 })).toBe(
      "4 Aufgaben importiert.",
    );
    expect(describeSyncResult({ ...none, importedSessions: 5 })).toBe(
      "5 Sessions importiert.",
    );
  });
  it("adds a conflict note", () => {
    expect(describeSyncResult({ ...none, conflictedTodos: 1 })).toBe(
      "1 Aufgabe wurde durch eine neuere Version vom Server ersetzt.",
    );
    expect(
      describeSyncResult({ ...none, importedTodos: 1, conflictedTodos: 2 }),
    ).toBe(
      "1 Aufgabe importiert. 2 Aufgaben wurden durch neuere Versionen vom Server ersetzt.",
    );
    expect(changedData({ ...none, conflictedTodos: 1 })).toBe(true);
    expect(changedData({ ...none, importedSessions: 1 })).toBe(true);
  });
});
