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
