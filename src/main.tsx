import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

async function bootstrap(): Promise<void> {
  // `npm run dev` im Browser: ohne Tauri-Laufzeit ein Fake-Backend mit Beispieldaten verwenden
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    await (await import("./dev/preview")).installPreviewBackend();
  }
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

void bootstrap();
