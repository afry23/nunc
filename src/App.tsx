import type { RouterHistory } from "@tanstack/react-router";
import { useEffect, useMemo } from "react";
import { useCompletionSound } from "./hooks/useCompletionSound";
import { RouterProvider, createAppRouter } from "./router";
import { connectBackend } from "./stores/backend";
import { useUiStore } from "./stores/uiStore";

/** Wurzel: verbindet Backend-Ereignisse mit den Stores und startet den Router. */
function App({ history }: { history?: RouterHistory }) {
  const router = useMemo(() => createAppRouter(history), [history]);
  const isMini = router.state.location.pathname === "/mini";

  useEffect(() => {
    useUiStore.getState().restorePrefs();
    let disconnect: (() => void) | undefined;
    let cancelled = false;
    connectBackend((route) => {
      // das Mini-Fenster bleibt beim Timer; nur das Hauptfenster folgt
      if (!isMini) void router.navigate({ to: route });
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else disconnect = unlisten;
      })
      .catch((error: unknown) => useUiStore.getState().fail(error));
    return () => {
      cancelled = true;
      disconnect?.();
    };
  }, [router, isMini]);

  return (
    <>
      {!isMini && <CompletionSound />}
      <RouterProvider router={router} />
    </>
  );
}

function CompletionSound() {
  useCompletionSound();
  return null;
}

export default App;
