import { describe, expect, it } from "vitest";
import { isHttpUrl, isWebDavComplete } from "./sync";

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
