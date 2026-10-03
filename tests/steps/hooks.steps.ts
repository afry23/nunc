import { cleanup } from "@testing-library/react";
import { After, Before, setWorldConstructor } from "quickpickle";
import { resetBackend } from "../helpers/ipc";
import { UiWorld } from "./world";

setWorldConstructor(UiWorld);

// Tag-Filter: nur @ui-Szenarien laufen hier (@domain/@integration laufen in Rust).
Before(async (world: UiWorld) => {
  if (!world.info.tags.includes("@ui")) world.context.skip();
});

After(async () => {
  cleanup();
  resetBackend();
});
