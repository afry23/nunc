import { useEffect, useState } from "react";
import { useT } from "../i18n";
import { isMobile } from "../lib/platform";
import type { SyncProvider } from "../lib/api";
import { isWebDavComplete } from "../lib/sync";
import { useSettingsStore } from "../stores/settingsStore";
import { describeSyncResult } from "../lib/sync";
import { useSyncStore } from "../stores/syncStore";
import { useUiStore } from "../stores/uiStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button, Segmented } from "./ui";
import { DriveSection } from "./DriveSection";
import { ImportDialog } from "./ImportDialog";
import { WebDavForm } from "./WebDavForm";

/** Erlaubte Abstände des automatischen Syncs in Minuten (0 = aus); Quelle: `SYNC_INTERVALS_MIN` in nunc-core. */
const SYNC_INTERVALS_MIN = [0, 5, 15, 30] as const;

/** Anbieter-Auswahl und Konfiguration der Synchronisierung. Die Wahl wirkt sofort (kein „Speichern“). */
export function SyncSection() {
  const t = useT();
  const provider = useSettingsStore((s) => s.settings?.syncProvider ?? "none");
  const setProvider = useSettingsStore((s) => s.setSyncProvider);
  const interval = useSettingsStore((s) => s.settings?.syncIntervalMin ?? 15);
  const patchSettings = useSettingsStore((s) => s.patch);
  const [pending, setPending] = useState<SyncProvider | null>(null);
  const webdav = useSyncStore((s) => s.webdav);
  const syncing = useSyncStore((s) => s.status.state === "syncing");
  const syncNow = useSyncStore((s) => s.syncNow);
  const drive = useSyncStore((s) => s.drive);
  const notify = useUiStore((s) => s.notify);
  const mobile = isMobile();

  useEffect(() => {
    void useSyncStore.getState().loadWebdav();
    if (!mobile) void useSyncStore.getState().loadDrive();
  }, [mobile]);
  const needsSetup =
    provider === "webdav" &&
    webdav !== null &&
    !(isWebDavComplete(webdav.config) && webdav.hasPassword);

  const options: { value: SyncProvider; disabled?: boolean }[] = [
    { value: "none" },
    // Google Drive gibt es mobil nicht
    ...(mobile ? [] : [{ value: "google_drive" as const }]),
    { value: "webdav" },
  ];

  const runSync = async (): Promise<void> => {
    const result = await syncNow();
    // Änderungen meldet `sync:complete`; hier nur der Fall „nichts zu tun“
    if (result && describeSyncResult(result) === null)
      notify(t("sync.upToDate"));
  };

  const choose = (next: SyncProvider): void => {
    if (next === provider) return;
    if (provider === "none") void setProvider(next);
    else setPending(next);
  };

  return (
    <section
      aria-labelledby="sync-heading"
      className="flex min-w-0 flex-col gap-4"
    >
      <h2 id="sync-heading" className="font-display text-xl font-bold">
        {t("sync.title")}
      </h2>
      <div className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        <Segmented
          label={t("sync.providerLabel")}
          value={provider}
          options={options.map((o) => ({
            ...o,
            label: t(`sync.provider.${o.value}`),
          }))}
          onChange={choose}
        />
        {provider !== "none" && !mobile && (
          <Segmented
            label={t("sync.intervalLabel")}
            value={String(interval)}
            options={SYNC_INTERVALS_MIN.map((minutes) => ({
              value: String(minutes),
              label: minutes === 0 ? t("sync.intervalOff") : `${minutes} min`,
            }))}
            onChange={(value) =>
              void patchSettings({ syncIntervalMin: Number(value) })
            }
          />
        )}
        {provider !== "none" && (
          <div>
            <Button
              disabled={
                syncing || (provider === "google_drive" && !drive?.connected)
              }
              aria-busy={syncing}
              onClick={() => void runSync()}
            >
              {syncing ? t("sync.syncing") : t("sync.now")}
            </Button>
          </div>
        )}
      </div>

      {needsSetup && (
        <p
          role="status"
          className="rounded-xl bg-surface-container-low p-4 text-orange-300"
        >
          {t("sync.webdavNotSetUp")}
        </p>
      )}

      {provider === "google_drive" && drive && !drive.connected && (
        <p
          role="status"
          className="rounded-xl bg-surface-container-low p-4 text-orange-300"
        >
          {t("sync.driveNotConnected")}
        </p>
      )}

      {provider === "google_drive" && !mobile && <DriveSection />}

      {provider !== "google_drive" && webdav && (
        <div className="flex min-w-0 flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
          <h3 className="font-semibold">WebDAV</h3>
          {provider !== "webdav" && (
            <p className="text-sm text-on-surface-variant">
              {t("sync.webdavChoose")}
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

      <ImportDialog />

      {pending && (
        <ConfirmDialog
          title={t("sync.switch.title")}
          confirmLabel={t("sync.switch.button")}
          onConfirm={() => {
            void setProvider(pending);
            setPending(null);
          }}
          onCancel={() => setPending(null)}
        >
          <p>
            {t("sync.switch.body", {
              provider: t(`sync.provider.${provider}`),
            })}
          </p>
        </ConfirmDialog>
      )}
    </section>
  );
}
