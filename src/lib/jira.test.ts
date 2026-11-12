import { describe, expect, it } from "vitest";
import type { JiraSettings } from "./api";
import { isJiraConfigured } from "./jira";

const settings = (
  over: Partial<JiraSettings["config"]> = {},
  hasToken = true,
) =>
  ({
    config: {
      baseUrl: "https://firma.atlassian.net",
      kind: "cloud",
      email: "me@firma.de",
      ...over,
    },
    hasToken,
  }) as JiraSettings;

describe("isJiraConfigured", () => {
  it("is false before the settings are loaded", () => {
    expect(isJiraConfigured(null)).toBe(false);
  });
  it("needs a stored token", () => {
    expect(isJiraConfigured(settings({}, false))).toBe(false);
    expect(isJiraConfigured(settings())).toBe(true);
  });
  it("needs an http(s) address", () => {
    expect(isJiraConfigured(settings({ baseUrl: "" }))).toBe(false);
    expect(isJiraConfigured(settings({ baseUrl: "firma.net" }))).toBe(false);
  });
  it("needs an e-mail for Cloud but not for Data Center", () => {
    expect(isJiraConfigured(settings({ email: "  " }))).toBe(false);
    expect(isJiraConfigured(settings({ kind: "data_center", email: "" }))).toBe(
      true,
    );
  });
});
