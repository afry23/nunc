import { screen, within } from "@testing-library/react";
import { expect } from "vitest";
import { Given, Then } from "quickpickle";
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

Then("the option {string} is selected", async (_w: UiWorld, name: string) => {
  expect((await radio(name)).getAttribute("aria-checked")).toBe("true");
});

Then(
  "the option {string} is not selected",
  async (_w: UiWorld, name: string) => {
    expect((await radio(name)).getAttribute("aria-checked")).toBe("false");
  },
);
