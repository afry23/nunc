import { useEffect, useState } from "react";
import { formatTimeOfDay } from "../lib/format";
import { useTodoistStore } from "../stores/todoistStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button } from "./ui";

/** Kurze Statuszeile: `✓ Todoist: 08:30` · `⟳ Todoist synchronisiert…` · `⚠ Todoist-Sync fehlgeschlagen`. */
export function todoistStatusText(
  state: "idle" | "syncing" | "error",
  lastSyncAt: string | null,
): string {
  if (state === "syncing") return "⟳ Todoist synchronisiert…";
  if (state === "error") return "⚠ Todoist-Sync fehlgeschlagen";
  return lastSyncAt
    ? `✓ Todoist: ${formatTimeOfDay(lastSyncAt)}`
    : "✓ Todoist: noch nicht synchronisiert";
}

/** Todoist-Konto verbinden und trennen, Zustand des Abgleichs (nur Desktop). */
export function TodoistSection() {
  const account = useTodoistStore((s) => s.account);
  const status = useTodoistStore((s) => s.status);
  const connecting = useTodoistStore((s) => s.connecting);
  const error = useTodoistStore((s) => s.connectError);
  const { connect, disconnect, syncNow } = useTodoistStore.getState();
  const [confirming, setConfirming] = useState(false);
  useEffect(() => {
    void useTodoistStore.getState().load();
  }, []);
  if (!account) return null;
  const syncing = status.state === "syncing";

  return (
    <section
      aria-labelledby="todoist-heading"
      className="flex min-w-0 flex-col gap-4"
    >
      <h2 id="todoist-heading" className="font-display text-xl font-bold">
        Todoist
      </h2>
      <div className="flex min-w-0 flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        {!account.connected ? (
          <>
            <p data-testid="todoist-state">Nicht verbunden</p>
            <p className="text-sm text-on-surface-variant">
              Aufgaben werden in beide Richtungen abgeglichen. Sessions und
              Einstellungen bleiben auf diesem Gerät.
            </p>
            {!account.configured && (
              <p className="text-sm text-orange-300">
                In diesem Build sind keine Todoist-Zugangsdaten hinterlegt.
              </p>
            )}
            {error && (
              <p role="alert" className="text-red-300">
                {error}
              </p>
            )}
            <div>
              <Button
                variant="primary"
                disabled={connecting || !account.configured}
                aria-busy={connecting}
                onClick={() => void connect()}
              >
                {connecting
                  ? "Warte auf den Browser …"
                  : "Mit Todoist verbinden"}
              </Button>
            </div>
          </>
        ) : (
          <>
            <div className="flex items-center gap-3">
              {account.avatarUrl && (
                <img
                  src={account.avatarUrl}
                  alt="Avatar"
                  className="size-10 rounded-full"
                />
              )}
              <p data-testid="todoist-state">
                {account.email ? `Verbunden als ${account.email}` : "Verbunden"}
              </p>
            </div>
            {status.state === "error" && status.errorMessage && (
              <p className="text-sm text-red-300">{status.errorMessage}</p>
            )}
            <div className="flex flex-wrap gap-2">
              <Button
                disabled={syncing}
                aria-busy={syncing}
                onClick={() => void syncNow()}
              >
                {syncing
                  ? "Synchronisiere Todoist …"
                  : "Todoist jetzt synchronisieren"}
              </Button>
              <Button variant="danger" onClick={() => setConfirming(true)}>
                Todoist trennen
              </Button>
            </div>
          </>
        )}
      </div>
      {confirming && (
        <ConfirmDialog
          title="Todoist-Verbindung trennen?"
          confirmLabel="Trennen"
          danger
          onConfirm={() => {
            setConfirming(false);
            void disconnect();
          }}
          onCancel={() => setConfirming(false)}
        >
          <p>
            Lokale Todos bleiben erhalten, werden aber nicht mehr mit Todoist
            synchronisiert.
          </p>
        </ConfirmDialog>
      )}
    </section>
  );
}
