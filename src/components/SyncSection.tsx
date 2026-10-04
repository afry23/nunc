import { useState } from "react";
import { isMobile } from "../lib/platform";
import type { SyncProvider } from "../lib/api";
import { useSettingsStore } from "../stores/settingsStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Segmented } from "./ui";

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
  const mobile = isMobile();

  const options: { value: SyncProvider; disabled?: boolean }[] = [
    { value: "none" },
    // Google Drive folgt mit M7 und gibt es mobil nicht
    ...(mobile ? [] : [{ value: "google_drive" as const, disabled: true }]),
    { value: "webdav" },
  ];

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
      </div>

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
