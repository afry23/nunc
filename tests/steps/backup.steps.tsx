import { act, screen, waitFor, within } from "@testing-library/react";
import { expect } from "vitest";
import { DataTable, Given, Then, When } from "quickpickle";
import type { BackupCategory } from "../../src/bindings";
import type { UiWorld } from "./world";

const BACKUP_DIR = "/data/backups";

const normalize = (text: string | null): string =>
  (text ?? "").replace(/\s+/g, " ").trim();

Given("the backups:", (world: UiWorld, table: DataTable) => {
  world.fake.backups = table.hashes().map((row) => {
    const filename = `nunc-backup-${row.date}.db`;
    return {
      filename,
      path: `${BACKUP_DIR}/${filename}`,
      createdAt: `${row.date}T12:00:00Z`,
      sizeBytes: Number(row.sizeBytes),
      category: row.category as BackupCategory,
    };
  });
});

Given("the backup status is outdated", (world: UiWorld) => {
  world.fake.backupsOutdated = true;
});

Given("the manual backup is held back", (world: UiWorld) => {
  world.fake.holdBackup = true;
});

Given(
  "the backup result is {string} with {int} bytes",
  (world: UiWorld, path: string, sizeBytes: number) => {
    world.fake.backupResult = { ...world.fake.backupResult, path, sizeBytes };
  },
);

Given(
  "the backup result has the rotation error {string}",
  (world: UiWorld, rotationError: string) => {
    world.fake.backupResult = { ...world.fake.backupResult, rotationError };
  },
);

When("the manual backup finishes", async (world: UiWorld) => {
  await act(async () => {
    world.fake.releaseBackup();
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
});

Then(
  "the button {string} is disabled",
  async (_world: UiWorld, name: string) => {
    const button = await screen.findByRole("button", { name });
    expect((button as HTMLButtonElement).disabled).toBe(true);
  },
);

Given("the folder dialog returns {string}", (world: UiWorld, path: string) => {
  world.fake.backupDirChoice = path;
});

Given("the folder dialog is cancelled", (world: UiWorld) => {
  world.fake.backupDirChoice = null;
});

Given("the file dialog returns {string}", (world: UiWorld, path: string) => {
  world.fake.restoreFile = path;
});

Given("the file dialog is cancelled", (world: UiWorld) => {
  world.fake.restoreFile = null;
});

Given(
  "the restore fails with {string}: {string}",
  (world: UiWorld, code: string, message: string) => {
    world.fake.restoreError = { code, message };
  },
);

Given(
  "the backup {string} contains the todos:",
  (world: UiWorld, filename: string, table: DataTable) => {
    world.fake.backupContents[`${BACKUP_DIR}/${filename}`] = table
      .hashes()
      .map((row) => row.rawText);
  },
);

Then("the backup list shows:", async (_world: UiWorld, table: DataTable) => {
  const list = await screen.findByRole("list", { name: "Backups" });
  await waitFor(() =>
    expect(within(list).getAllByRole("listitem")).toHaveLength(
      table.hashes().length,
    ),
  );
  const rows = within(list)
    .getAllByRole("listitem")
    .map((item) => normalize(item.textContent));
  table.hashes().forEach((expected, index) => {
    for (const part of [expected.date, expected.category, expected.size])
      expect(rows[index]).toContain(part);
  });
});

Given("the export dialog is cancelled", (world: UiWorld) => {
  world.fake.exportTarget = null;
});

Then(
  "the button {string} is enabled",
  async (_world: UiWorld, name: string) => {
    const button = await screen.findByRole("button", { name });
    expect((button as HTMLButtonElement).disabled).toBe(false);
  },
);
