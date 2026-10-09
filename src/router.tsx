import {
  Outlet,
  RouterProvider,
  createRootRoute,
  createRoute,
  createRouter,
  type RouterHistory,
} from "@tanstack/react-router";
import { AppShell } from "./components/AppShell";
import { HelpView } from "./views/HelpView";
import { MiniTimerView } from "./views/MiniTimerView";
import { SettingsView } from "./views/SettingsView";
import { StatsView } from "./views/StatsView";
import { TimesView } from "./views/TimesView";
import { TimerView } from "./views/TimerView";
import { TodoView } from "./views/TodoView";

const rootRoute = createRootRoute({ component: Outlet });
const shellRoute = createRoute({
  getParentRoute: () => rootRoute,
  id: "shell",
  component: AppShell,
});

const routeTree = rootRoute.addChildren([
  shellRoute.addChildren([
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/",
      component: TimerView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/todos",
      component: TodoView,
    }),
    createRoute({
      getParentRoute: () => shellRoute,
      path: "/times",
      component: TimesView,
    }),
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
