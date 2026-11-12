import { DataTable, Given } from "quickpickle";
import type { UiWorld } from "./world";

Given("the Jira settings are:", (world: UiWorld, table: DataTable) => {
  const row = table.hashes()[0];
  world.fake.jira = {
    ...world.fake.jira,
    config: {
      baseUrl: row.baseUrl,
      kind: row.kind as "cloud" | "data_center",
      email: row.email,
    },
  };
});

Given("a Jira token is stored", (world: UiWorld) => {
  world.fake.jira = { ...world.fake.jira, hasToken: true };
});

Given("the Jira connection test succeeds", (world: UiWorld) => {
  world.fake.jiraTest = { success: true, message: "" };
});

Given(
  "the Jira connection test fails with {string}",
  (world: UiWorld, message: string) => {
    world.fake.jiraTest = { success: false, message };
  },
);

Given("Jira is set up", (world: UiWorld) => {
  world.fake.jira = {
    config: {
      baseUrl: "https://firma.atlassian.net",
      kind: "cloud",
      email: "me@firma.de",
    },
    hasToken: true,
  };
});

Given("Jira knows the tickets:", (world: UiWorld, table: DataTable) => {
  world.fake.jiraIssues = table.hashes().map((row) => ({
    key: row.key,
    summary: row.summary,
    mine: row.mine === "true",
  }));
});

Given(
  "the Jira search fails with {string}: {string}",
  (world: UiWorld, code: string, message: string) => {
    world.fake.jiraSearchError = { code, message };
  },
);
