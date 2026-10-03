import { QuickPickleWorld } from "quickpickle";
import type { BackendMock } from "../helpers/ipc";

export class UiWorld extends QuickPickleWorld {
  backend?: BackendMock;
  result?: unknown;
}
