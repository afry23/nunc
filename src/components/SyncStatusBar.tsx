import { formatDateTime } from "../lib/format";
import { useSettingsStore } from "../stores/settingsStore";
import { useSyncStore } from "../stores/syncStore";
import { Button } from "./ui";

/** Schmale Statusleiste der Synchronisierung (nur sichtbar, solange ein Anbieter gewählt ist). */
export function SyncStatusBar() {
  const provider = useSettingsStore((s) => s.settings?.syncProvider ?? "none");
  const status = useSyncStore((s) => s.status);
  const syncNow = useSyncStore((s) => s.syncNow);
  if (provider === "none") return null;

  return (
    <div
      role="status"
      aria-label="Sync-Status"
      data-testid="sync-status"
      data-state={status.state}
      className="flex min-h-11 flex-wrap items-center gap-x-3 gap-y-1 bg-surface-container-low px-4 py-1 text-sm text-on-surface-variant"
    >
      {status.state === "syncing" && (
        <>
          <span
            aria-hidden="true"
            className="size-3 animate-spin rounded-full border-2 border-current border-t-transparent"
          />
          <span>Synchronisiere …</span>
        </>
      )}
      {status.state === "idle" && (
        <span>
          {status.lastSyncAt
            ? `Zuletzt synchronisiert: ${formatDateTime(status.lastSyncAt)}`
            : "Noch nicht synchronisiert"}
        </span>
      )}
      {status.state === "error" && (
        <>
          <span className="text-red-300">
            Sync-Fehler: {status.errorMessage ?? "unbekannt"}
          </span>
          <Button variant="ghost" onClick={() => void syncNow()}>
            Erneut versuchen
          </Button>
        </>
      )}
    </div>
  );
}
