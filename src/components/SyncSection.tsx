import { useEffect, useState } from "react";
import { isMobile } from "../lib/platform";
import type { SyncProvider } from "../lib/api";
import { isWebDavComplete } from "../lib/sync";
import { useSettingsStore } from "../stores/settingsStore";
import { describeSyncResult } from "../lib/sync";
import { useSyncStore } from "../stores/syncStore";
import { useUiStore } from "../stores/uiStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button, Segmented } from "./ui";
import { WebDavForm } from "./WebDavForm";

const PROVIDER_LABELS: Record<SyncProvider, string> = {
  none: "Kein Sync",
  google_drive: "Google Drive",
  webdav: "WebDAV",
};

/** Anbieter-Auswahl und Konfiguration der Synchronisierung. Die Wahl wirkt sofort (kein „Speichern“). */
export function SyncSection() {
  const provider = useSettingsStore((s) => s.settings?.syncProvider ?? "none");
  const setProvider = useSettingsStore((s) => s.setSyncProvider);
  const [pending, setPending] = useState<SyncProvider | null>(null);
  const webdav = useSyncStore((s) => s.webdav);
  const syncing = useSyncStore((s) => s.status.state === "syncing");
  const syncNow = useSyncStore((s) => s.syncNow);
  const notify = useUiStore((s) => s.notify);
  const mobile = isMobile();

  useEffect(() => {
    void useSyncStore.getState().loadWebdav();
  }, []);
  const needsSetup =
    provider === "webdav" &&
    webdav !== null &&
    !(isWebDavComplete(webdav.config) && webdav.hasPassword);

  const options: { value: SyncProvider; disabled?: boolean }[] = [
    { value: "none" },
    // Google Drive folgt mit M7 und gibt es mobil nicht
    ...(mobile ? [] : [{ value: "google_drive" as const, disabled: true }]),
    { value: "webdav" },
  ];

  const runSync = async (): Promise<void> => {
    const result = await syncNow();
    // Änderungen meldet `sync:complete`; hier nur der Fall „nichts zu tun“
    if (result && describeSyncResult(result) === null)
      notify("Bereits auf dem neuesten Stand.");
  };

  const choose = (next: SyncProvider): void => {
    if (next === provider) return;
    if (provider === "none") void setProvider(next);
    else setPending(next);
  };

  return (
    <section aria-labelledby="sync-heading" className="flex flex-col gap-4">
      <h2 id="sync-heading" className="font-display text-xl font-bold">
        Synchronisierung
      </h2>
      <div className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        <Segmented
          label="Sync-Anbieter"
          value={provider}
          options={options.map((o) => ({
            ...o,
            label: PROVIDER_LABELS[o.value],
          }))}
          onChange={choose}
        />
        {!mobile && (
          <p className="text-sm text-on-surface-variant">
            Google Drive folgt in einer späteren Version.
          </p>
        )}
        {provider !== "none" && (
          <div>
            <Button
              disabled={syncing}
              aria-busy={syncing}
              onClick={() => void runSync()}
            >
              {syncing ? "Synchronisiere …" : "Jetzt synchronisieren"}
            </Button>
          </div>
        )}
      </div>

      {needsSetup && (
        <p
          role="status"
          className="rounded-xl bg-surface-container-low p-4 text-orange-300"
        >
          WebDAV ist ausgewählt, aber noch nicht eingerichtet. Trage unten die
          Verbindungsdaten ein, damit synchronisiert werden kann.
        </p>
      )}

      {webdav && (
        <div className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
          <h3 className="font-semibold">WebDAV</h3>
          {provider !== "webdav" && (
            <p className="text-sm text-on-surface-variant">
              Wähle WebDAV als Anbieter, um die Verbindung zu bearbeiten.
            </p>
          )}
          <WebDavForm
            key={JSON.stringify(webdav)}
            stored={webdav.config}
            hasPassword={webdav.hasPassword}
            disabled={provider !== "webdav"}
          />
        </div>
      )}

      {pending && (
        <ConfirmDialog
          title="Anbieter wechseln?"
          confirmLabel="Anbieter wechseln"
          onConfirm={() => {
            void setProvider(pending);
            setPending(null);
          }}
          onCancel={() => setPending(null)}
        >
          <p>
            Die Synchronisierung mit {PROVIDER_LABELS[provider]} wird beendet.
            Die gespeicherten Zugangsdaten bleiben erhalten.
          </p>
        </ConfirmDialog>
      )}
    </section>
  );
}
