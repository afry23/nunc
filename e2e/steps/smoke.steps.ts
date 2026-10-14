/// <reference types="expect-webdriverio" />
import { $, expect } from "@wdio/globals";
import { Given, Then, When } from "@wdio/cucumber-framework";

/** Klickt ein Element nach sichtbarem Namen (Button, Link oder Radio). */
async function clickByName(name: string): Promise<void> {
  const element = await $(
    `//*[(self::button or self::a or @role='radio') and normalize-space(.)='${name}']`,
  );
  await element.waitForClickable();
  await element.click();
}

Given("the app is started with a short pomodoro", async () => {
  await $("main").waitForDisplayed();
});

When("I add the todo {string}", async (text: string) => {
  await clickByName("Aufgaben");
  const input = await $("#todo-input");
  await input.waitForDisplayed();
  await input.setValue(text);
  await clickByName("Hinzufügen");
  await expect($(`li[data-testid="todo-item"]`)).toBeDisplayed();
});

When("I start a pomodoro for the todo {string}", async (title: string) => {
  await clickByName("Timer");
  const select = await $("#active-todo");
  await select.selectByVisibleText(title);
  await clickByName("Start");
  await expect($('[role="timer"]')).toHaveAttribute("data-state", "running");
});

When("the pomodoro has ended", async () => {
  await $('[role="timer"][data-state="idle"]').waitForExist({
    timeout: 20_000,
    timeoutMsg: "Der Pomodoro ist nicht abgelaufen",
  });
  // der Pausenvorschlag liegt als Dialog über der Oberfläche
  await $('[role="dialog"]').waitForDisplayed();
  await clickByName("Später");
  await $('[role="dialog"]').waitForExist({ reverse: true });
});

Then("the statistics show {int} pomodoro", async (count: number) => {
  await clickByName("Statistik");
  const term = await $(
    "//dt[normalize-space(.)='Pomodoros']/following-sibling::dd[1]",
  );
  await term.waitForDisplayed();
  await expect(term).toHaveText(String(count));
});
