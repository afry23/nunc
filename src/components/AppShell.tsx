import { Outlet, useRouterState } from "@tanstack/react-router";
import { useKeyboardInset } from "../hooks/useKeyboardInset";
import { useNavMode } from "../hooks/useMediaQuery";
import { usePullToRefresh } from "../hooks/usePullToRefresh";
import { useShortcuts } from "../hooks/useShortcuts";
import { useT } from "../i18n";
import { shouldRefresh } from "../lib/gestures";
import { reloadAll } from "../stores/backend";
import { useSettingsStore } from "../stores/settingsStore";
import { useSyncStore } from "../stores/syncStore";
import { useTimerStore } from "../stores/timerStore";
import { ActiveSessionBar } from "./ActiveSessionBar";
import { SessionEndDialog } from "./SessionEndDialog";
import { BottomTabs, ShellMenu, Sidebar } from "./Navigation";
import { SyncStatusBar } from "./SyncStatusBar";
import { Toast } from "./Toast";

/** Aktualisieren per Ziehen: alles neu laden und, wenn ein Anbieter gewählt ist, synchronisieren. */
const refresh = (): void => {
  void reloadAll();
  const settings = useSettingsStore.getState().settings;
  if (settings && settings.syncProvider !== "none")
    void useSyncStore.getState().syncNow();
};

/** Rahmen der Hauptansichten: Seitenleiste (≥ 1024 px), Leiste (≥ 600 px) oder Tab-Leiste, Meldungen, Pausenvorschlag. */
export function AppShell() {
  const t = useT();
  const navMode = useNavMode();
  // mit Seitenleiste oder Leiste: keine Tab-Leiste, die Aktiv-Leiste steht unten im Inhalt
  const wide = navMode !== "tabs";
  const running = useTimerStore((s) => s.snapshot.state === "running");
  const active = useTimerStore((s) => s.snapshot.state !== "idle");
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  // im Fokusmodus ist der Timer selbst zu sehen
  const showBar = active && pathname !== "/focus";
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
        {t("nav.skip")}
      </a>
      {wide && <Sidebar rail={navMode === "rail"} />}
      <main
        id="main"
        tabIndex={-1}
        className={`min-w-0 flex-1 outline-none ${wide || keyboardOpen ? "" : showBar ? "pb-36" : "pb-20"}`}
      >
        <div className="flex min-h-dvh flex-col">
          <div className="flex-1">
            {!wide && <ShellMenu />}
            <SyncStatusBar />
            <Outlet />
          </div>
          {showBar && wide && (
            <ActiveSessionBar floating={false} keyboardOpen={keyboardOpen} />
          )}
        </div>
      </main>
      {showBar && !wide && (
        <ActiveSessionBar floating keyboardOpen={keyboardOpen} />
      )}
      {!wide && !keyboardOpen && <BottomTabs />}
      {pull > 0 && (
        <p
          role="status"
          className="glass fixed top-4 left-1/2 z-50 -translate-x-1/2 rounded-full px-4 py-2 text-sm"
          style={{ marginTop: pull }}
        >
          {shouldRefresh(pull) ? t("shell.pull.release") : t("shell.pull.pull")}
        </p>
      )}
      <Toast />
      <SessionEndDialog />
    </div>
  );
}
