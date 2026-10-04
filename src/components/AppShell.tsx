import { Outlet } from "@tanstack/react-router";
import { useKeyboardInset } from "../hooks/useKeyboardInset";
import { useIsWide } from "../hooks/useMediaQuery";
import { usePullToRefresh } from "../hooks/usePullToRefresh";
import { useShortcuts } from "../hooks/useShortcuts";
import { shouldRefresh } from "../lib/gestures";
import { reloadAll } from "../stores/backend";
import { useTimerStore } from "../stores/timerStore";
import { BreakDialog } from "./BreakDialog";
import { BottomTabs, Sidebar } from "./Navigation";
import { Toast } from "./Toast";

const refresh = (): void => void reloadAll();

/** Rahmen der Hauptansichten: Seitenleiste (≥ 768 px) oder Tab-Leiste, Meldungen, Pausenvorschlag. */
export function AppShell() {
  const wide = useIsWide();
  const running = useTimerStore((s) => s.snapshot.state === "running");
  const keyboardOpen = useKeyboardInset();
  const pull = usePullToRefresh(refresh);
  useShortcuts();

  return (
    <div
      data-testid="app-shell"
      className={`safe-top flex min-h-dvh ${running ? "timer-breathing" : ""}`}
    >
      <a
        href="#main"
        className="sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-50 focus:rounded-lg focus:bg-surface-container-high focus:px-4 focus:py-2"
      >
        Zum Inhalt springen
      </a>
      {wide && <Sidebar />}
      <main
        id="main"
        tabIndex={-1}
        className={`min-w-0 flex-1 outline-none ${wide || keyboardOpen ? "" : "pb-20"}`}
      >
        <Outlet />
      </main>
      {!wide && !keyboardOpen && <BottomTabs />}
      {pull > 0 && (
        <p
          role="status"
          className="glass fixed top-4 left-1/2 z-50 -translate-x-1/2 rounded-full px-4 py-2 text-sm"
          style={{ marginTop: pull }}
        >
          {shouldRefresh(pull)
            ? "Loslassen zum Aktualisieren"
            : "Zum Aktualisieren ziehen"}
        </p>
      )}
      <Toast />
      <BreakDialog />
    </div>
  );
}
