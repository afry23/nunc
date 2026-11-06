import {
  Outlet,
  Navigate,
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
  type RouterHistory,
} from "@tanstack/react-router";
import { AppShell } from "./components/AppShell";
import { HabitsView } from "./views/HabitsView";
import { HelpView } from "./views/HelpView";
import { MiniTimerView } from "./views/MiniTimerView";
import { SettingsView } from "./views/SettingsView";
import { StatsView } from "./views/StatsView";
import { LogView } from "./views/LogView";
import { ReviewView } from "./views/ReviewView";
import { TimerView } from "./views/TimerView";
import { TodayView } from "./views/TodayView";
import { TodoView } from "./views/TodoView";

const rootRoute = createRootRoute({ component: Outlet });
const shellRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: "shell",
  component: AppShell,
});

function redirect(from: string, to: string) {
  return createRoute({
    getParentRoute: () => shellRoute,
    path: from,
    component: () => <Navigate to={to} replace />,
  });
}

const routeTree = rootRoute.addChildren([
  shellRoute.addChildren([
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/",
      component: TodayView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/tasks",
      // geöffnete Aufgabe (Details); Filter und Sortierung bleiben beim Wechsel erhalten
      validateSearch: (search: Record<string, unknown>): { task?: number } => {
        const task = Number(search.task);
        return Number.isInteger(task) && task > 0 ? { task } : {};
      },
      component: TodoView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/log",
      component: LogView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/review",
      component: ReviewView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/focus",
      component: TimerView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/habits",
      component: HabitsView,
    }),
    // alte Adressen (Lesezeichen, Tray, Hilfe) führen zu den neuen Bereichen
    redirect("/today", "/"),
    redirect("/todos", "/tasks"),
    redirect("/times", "/log"),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/stats",
      component: StatsView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/settings",
      component: SettingsView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/help",
      component: HelpView,
    }),
  ]),
  // eigenes Fenster ohne Navigation
  createRoute({
    getParentRoute: () => rootRoute,
    path: "/mini",
    component: MiniTimerView,
  }),
]);

export function createAppRouter(history?: RouterHistory) {
  return createRouter({ routeTree, history, defaultPreload: "intent" });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof createAppRouter>;
  }
}

export { RouterProvider };
