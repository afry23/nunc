import { useEffect, useState } from "react";
import { t, useT } from "../i18n";
import { formatTimeOfDay } from "../lib/format";
import { useTodoistStore } from "../stores/todoistStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button } from "./ui";

/** Kurze Statuszeile: `✓ Todoist: 08:30` · `⟳ Todoist synchronisiert…` · `⚠ Todoist-Sync fehlgeschlagen`. */
export function todoistStatusText(
  state: "idle" | "syncing" | "error",
  lastSyncAt: string | null,
): string {
  if (state === "syncing") return t("todoist.status.syncing");
  if (state === "error") return t("todoist.status.error");
  return lastSyncAt
    ? t("todoist.status.last", { time: formatTimeOfDay(lastSyncAt) })
    : t("todoist.status.never");
}

/** Todoist-Konto verbinden und trennen, Zustand des Abgleichs (nur Desktop). */
export function TodoistSection() {
  const t = useT();
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
      id="settings-todoist"
      aria-labelledby="todoist-heading"
      className="flex min-w-0 scroll-mt-24 flex-col gap-4"
    >
      <h2 id="todoist-heading" className="font-display text-xl font-bold">
        {t("todoist.title")}
      </h2>
      <div className="flex min-w-0 flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        {!account.connected ? (
          <>
            <p data-testid="todoist-state">{t("todoist.notConnected")}</p>
            <p className="text-sm text-on-surface-variant">
              {t("todoist.explain")}
            </p>
            {!account.configured && (
              <p className="text-sm text-orange-300">
                {t("todoist.noCredentials")}
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
                {connecting ? t("todoist.waiting") : t("todoist.connect")}
              </Button>
            </div>
          </>
        ) : (
          <>
            <div className="flex items-center gap-3">
              {account.avatarUrl && (
                <img
                  src={account.avatarUrl}
                  alt={t("todoist.avatar")}
                  className="size-10 rounded-full"
                />
              )}
              <p data-testid="todoist-state">
                {account.email
                  ? t("todoist.connectedAs", { email: account.email })
                  : t("todoist.connected")}
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
                {syncing ? t("todoist.syncing") : t("todoist.syncNow")}
              </Button>
              <Button variant="danger" onClick={() => setConfirming(true)}>
                {t("todoist.disconnect")}
              </Button>
            </div>
          </>
        )}
      </div>
      {confirming && (
        <ConfirmDialog
          title={t("todoist.disconnectTitle")}
          confirmLabel={t("todoist.disconnectConfirm")}
          danger
          onConfirm={() => {
            setConfirming(false);
            void disconnect();
          }}
          onCancel={() => setConfirming(false)}
        >
          <p>{t("todoist.disconnectBody")}</p>
        </ConfirmDialog>
      )}
    </section>
  );
}
