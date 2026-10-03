import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";

type Handler = (args: unknown) => unknown;

export interface BackendMock {
  /** Alle bisherigen Aufrufe in Reihenfolge: `{ cmd, args }`. */
  calls: { cmd: string; args: unknown }[];
}

/**
 * Ersetzt das Tauri-Backend durch Handler pro Command-Name (snake_case wie in Rust).
 * Unbekannte Commands werfen, damit fehlende Mocks sofort auffallen.
 */
export function mockBackend(handlers: Record<string, Handler>): BackendMock {
  const mock: BackendMock = { calls: [] };
  mockIPC((cmd, args) => {
    mock.calls.push({ cmd, args });
    const handler = handlers[cmd];
    if (!handler) throw new Error(`Kein Mock für Command "${cmd}"`);
    return handler(args);
  });
  return mock;
}

export function resetBackend(): void {
  clearMocks();
}
