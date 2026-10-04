import { cleanup } from "@testing-library/react";
import { vi } from "vitest";
import { After, Before, setWorldConstructor } from "quickpickle";
import {
  resetNotificationHint,
  useNotificationStore,
} from "../../src/stores/notificationStore";
import { useSettingsStore } from "../../src/stores/settingsStore";
import { useTimerStore } from "../../src/stores/timerStore";
import { useTodoStore } from "../../src/stores/todoStore";
import { useUiStore } from "../../src/stores/uiStore";
import { createFakeBackend } from "../helpers/fakeBackend";
import { setLanguage } from "../../src/i18n";
import { resetBackend } from "../helpers/ipc";
import { UiWorld } from "./world";

setWorldConstructor(UiWorld);

/** Die Stores sind Singletons: vor jedem Szenario auf den Anfangszustand setzen. */
function resetStores(): void {
  useSettingsStore.setState(useSettingsStore.getInitialState(), true);
  useTimerStore.setState(useTimerStore.getInitialState(), true);
  useTodoStore.setState(useTodoStore.getInitialState(), true);
  useNotificationStore.setState(useNotificationStore.getInitialState(), true);
  resetNotificationHint();
  useUiStore.setState(useUiStore.getInitialState(), true);
  setLanguage("de");
}

// Tag-Filter: nur @ui-Szenarien laufen hier (@domain/@integration laufen in Rust).
Before(async (world: UiWorld) => {
  if (!world.info.tags.includes("@ui")) {
    world.context.skip();
    return;
  }
  resetStores();
  // jsdom kennt kein Scrollen; der Router ruft es beim Navigieren auf
  window.scrollTo = () => undefined;
  world.fake = createFakeBackend();
});

After(async () => {
  cleanup();
  resetBackend();
  resetStores();
  vi.restoreAllMocks();
  localStorage.clear();
  vi.unstubAllGlobals();
  Reflect.deleteProperty(Element.prototype, "scrollIntoView");
  Reflect.deleteProperty(window, "scrollY");
  Reflect.deleteProperty(window, "NuncAndroid");
  Reflect.deleteProperty(window, "matchMedia");
});
