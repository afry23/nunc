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
    let disconnect: (() => void) | undefined;
    let cancelled = false;
    connectBackend()
      .then((unlisten) => {
        if (cancelled) unlisten();
        else disconnect = unlisten;
      })
      .catch((error: unknown) => useUiStore.getState().fail(error));
    return () => {
      cancelled = true;
      disconnect?.();
    };
  }, []);

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
