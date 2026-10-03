import { expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Given, Then, When } from "quickpickle";
import App from "../../src/App";
import { commands } from "../../src/lib/api";
import { mockBackend } from "../helpers/ipc";
import type { UiWorld } from "./world";

Given(
  "the backend reports version {string}",
  (world: UiWorld, version: string) => {
    world.backend = mockBackend({
      app_info: () => ({ name: "nunc", version }),
    });
  },
);

When("I open the app", () => {
  render(<App />);
});

When("I ask the backend for the app info", async (world: UiWorld) => {
  world.result = await commands.appInfo();
});

Then("I see the heading {string}", (_world: UiWorld, text: string) => {
  screen.getByRole("heading", { name: text });
});

Then("the app info version is {string}", (world: UiWorld, version: string) => {
  expect(world.result).toMatchObject({ version });
});
