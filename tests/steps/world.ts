import { QuickPickleWorld } from "quickpickle";
import type { BackendMock } from "../helpers/ipc";
import type { FakeBackend } from "../helpers/fakeBackend";

export class UiWorld extends QuickPickleWorld {
  backend?: BackendMock;
  /** Zustandsbehaftetes Fake-Backend, wird vor jedem @ui-Szenario angelegt. */
  fake!: FakeBackend;
  /** `true`, sobald die App gerendert ist (danach lösen Backend-Meldungen echte Ereignisse aus). */
  rendered = false;
  result?: unknown;
}
