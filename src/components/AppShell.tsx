import { Outlet } from "@tanstack/react-router";
import { useShortcuts } from "../hooks/useShortcuts";
import { useIsWide } from "../hooks/useMediaQuery";
import { useTimerStore } from "../stores/timerStore";
import { BreakDialog } from "./BreakDialog";
import { BottomTabs, Sidebar } from "./Navigation";
import { Toast } from "./Toast";

/** Rahmen der Hauptansichten: Seitenleiste (≥ 768 px) oder Tab-Leiste, Meldungen, Pausenvorschlag. */
export function AppShell() {
  const wide = useIsWide();
  const running = useTimerStore((s) => s.snapshot.state === "running");
  useShortcuts();

  return (
    <div
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
        className={`min-w-0 flex-1 outline-none ${wide ? "" : "pb-20"}`}
      >
        <Outlet />
      </main>
      {!wide && <BottomTabs />}
      <Toast />
      <BreakDialog />
    </div>
  );
}
