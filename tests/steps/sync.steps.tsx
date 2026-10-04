import { screen, waitFor, within } from "@testing-library/react";
import { expect } from "vitest";
import { act } from "@testing-library/react";
import { emit } from "@tauri-apps/api/event";
import { DataTable, Given, Then, When } from "quickpickle";
import type { UiWorld } from "./world";

Given(
  "the provider selection fails with {string}: {string}",
  (world: UiWorld, code: string, message: string) => {
    world.fake.providerError = { code, message };
  },
);

Then(
  "I see the radio group {string} with the options {string}",
  async (_world: UiWorld, label: string, options: string) => {
    const group = await screen.findByRole("radiogroup", { name: label });
    expect(
      within(group)
        .getAllByRole("radio")
        .map((radio) => radio.textContent?.trim()),
    ).toEqual(options.split(",").map((o) => o.trim()));
  },
);

const radio = async (name: string): Promise<HTMLElement> =>
  screen.findByRole("radio", { name });

Then("the option {string} is disabled", async (_w: UiWorld, name: string) => {
  expect((await radio(name)).hasAttribute("disabled")).toBe(true);
});

Then("the option {string} is enabled", async (_w: UiWorld, name: string) => {
  expect((await radio(name)).hasAttribute("disabled")).toBe(false);
});

Then("the option {string} is selected", async (_w: UiWorld, name: string) => {
  expect((await radio(name)).getAttribute("aria-checked")).toBe("true");
});

Then(
  "the option {string} is not selected",
  async (_w: UiWorld, name: string) => {
    expect((await radio(name)).getAttribute("aria-checked")).toBe("false");
  },
);

// ---------------------------------------------------------------------------------------------
// WebDAV-Einstellungen

Given("the WebDAV settings are:", (world: UiWorld, table: DataTable) => {
  const row = table.hashes()[0];
  world.fake.webdav = {
    ...world.fake.webdav,
    config: {
      url: row.url,
      authMethod: row.authMethod as "basic" | "digest",
      username: row.username,
      remotePath: row.remotePath,
    },
  };
});

Given("a WebDAV password is stored", (world: UiWorld) => {
  world.fake.webdav = { ...world.fake.webdav, hasPassword: true };
});

Given(
  "the connection test succeeds for {string}",
  (world: UiWorld, host: string) => {
    world.fake.webdavTest = { success: true, message: host };
  },
);

Given(
  "the connection test fails with {string}",
  (world: UiWorld, message: string) => {
    world.fake.webdavTest = { success: false, message };
  },
);

Then("I see the fields {string}", async (_world: UiWorld, labels: string) => {
  for (const label of labels.split(",").map((l) => l.trim()))
    await screen.findByLabelText(label);
});

Then("the field {string} is disabled", async (_w: UiWorld, label: string) => {
  expect(
    ((await screen.findByLabelText(label)) as HTMLInputElement).disabled,
  ).toBe(true);
});

Then("the field {string} is enabled", async (_w: UiWorld, label: string) => {
  await waitFor(async () =>
    expect(
      ((await screen.findByLabelText(label)) as HTMLInputElement).disabled,
    ).toBe(false),
  );
});

Then("the field {string} is masked", async (_w: UiWorld, label: string) => {
  expect((await screen.findByLabelText(label)).getAttribute("type")).toBe(
    "password",
  );
});

Then("the field {string} is readable", async (_w: UiWorld, label: string) => {
  expect((await screen.findByLabelText(label)).getAttribute("type")).toBe(
    "text",
  );
});

Then(
  "the field {string} has the value {string}",
  async (_w: UiWorld, label: string, value: string) => {
    expect(
      ((await screen.findByLabelText(label)) as HTMLInputElement).value,
    ).toBe(value);
  },
);

// ---------------------------------------------------------------------------------------------
// Sync-Status und -Ergebnis

Given(
  "the sync status is {string} with the last sync at {string}",
  (world: UiWorld, state: string, lastSyncAt: string) => {
    world.fake.syncStatus = {
      state: state as "idle" | "syncing" | "error",
      lastSyncAt,
      errorMessage: null,
    };
  },
);

Given(
  "the next sync imports {int} sessions, {int} todos and {int} conflicts",
  (world: UiWorld, sessions: number, todos: number, conflicts: number) => {
    world.fake.syncOutcome = {
      importedSessions: sessions,
      importedTodos: todos,
      conflictedTodos: conflicts,
    };
  },
);

Given("the next sync fails with {string}", (world: UiWorld, error: string) => {
  world.fake.syncOutcome = { error };
});

When(
  "the backend reports the sync status {string}",
  async (_world: UiWorld, state: string) => {
    await act(async () => {
      await emit("sync:status", {
        state,
        lastSyncAt: null,
        errorMessage: null,
      });
    });
  },
);

When(
  "the backend reports the sync status {string} with {string}",
  async (_world: UiWorld, state: string, message: string) => {
    await act(async () => {
      await emit("sync:status", {
        state,
        lastSyncAt: null,
        errorMessage: message,
      });
    });
  },
);

When(
  "the backend finishes a sync that imported the todos:",
  async (world: UiWorld, table: DataTable) => {
    for (const row of table.hashes()) world.fake.addTodo(row.rawText);
    await act(async () => {
      await emit("sync:complete", {
        importedTodos: table.hashes().length,
        importedSessions: 0,
        conflictedTodos: 0,
      });
    });
  },
);

// ---------------------------------------------------------------------------------------------
// Google Drive

Given(
  "Google Drive is connected as {string}",
  (world: UiWorld, email: string) => {
    world.fake.drive = { ...world.fake.drive, connected: true, email };
  },
);

Given("this build has no Google credentials", (world: UiWorld) => {
  world.fake.drive = { ...world.fake.drive, configured: false };
});

Given(
  "connecting Google Drive fails with {string}: {string}",
  (world: UiWorld, code: string, message: string) => {
    world.fake.driveConnectError = { code, message };
  },
);

Given(
  "Drive holds a sync file with {int} todos and {int} sessions",
  (world: UiWorld, todoCount: number, sessionCount: number) => {
    world.fake.importInfo = { available: true, todoCount, sessionCount };
  },
);

Given("the Drive folder link is {string}", (world: UiWorld, url: string) => {
  world.fake.drive = { ...world.fake.drive, folderUrl: url };
});

Given("the sync is held", (world: UiWorld) => {
  world.fake.holdSync = true;
});

When("the held sync finishes", async (world: UiWorld) => {
  await act(async () => {
    world.fake.releaseSync();
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
});

Then(
  "I see a progress bar {string}",
  async (_world: UiWorld, label: string) => {
    await screen.findByRole("progressbar", { name: label });
  },
);

Then("the browser opened {string}", async (world: UiWorld, url: string) => {
  await waitFor(() => expect(world.fake.openedUrls).toContain(url));
});
