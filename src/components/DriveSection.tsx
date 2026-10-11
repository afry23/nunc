import { useT } from "../i18n";
import { openExternal } from "../lib/links";
import { useSettingsStore } from "../stores/settingsStore";
import { useSyncStore } from "../stores/syncStore";
import { GoogleClientForm } from "./GoogleClientForm";
import { Button, Toggle } from "./ui";

/** Verbindung zu Google Drive: anmelden, abmelden, pausieren, Ablageort (nur Desktop). */
export function DriveSection() {
  const t = useT();
  const drive = useSyncStore((s) => s.drive);
  const connecting = useSyncStore((s) => s.connecting);
  const error = useSyncStore((s) => s.connectError);
  const { connectDrive, disconnectDrive } = useSyncStore.getState();
  const enabled = useSettingsStore((s) => s.settings?.syncEnabled ?? true);
  const patch = useSettingsStore((s) => s.patch);
  if (!drive) return null;

  return (
    <div className="flex min-w-0 flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
      <h3 className="font-semibold">Google Drive</h3>
      {!drive.connected ? (
        <>
          <p data-testid="drive-state">{t("drive.notConnected")}</p>
          {error && (
            <p role="alert" className="text-red-300">
              {error}
            </p>
          )}
          <div>
            <Button
              variant="primary"
              disabled={connecting || !drive.configured}
              aria-busy={connecting}
              onClick={() => void connectDrive()}
            >
              {connecting ? t("drive.waiting") : t("drive.connect")}
            </Button>
          </div>
          <h4 className="font-semibold">{t("drive.client.title")}</h4>
          <GoogleClientForm storedId={drive.clientId} />
        </>
      ) : (
        <>
          <p data-testid="drive-state">
            {drive.email
              ? t("drive.connectedAs", { email: drive.email })
              : t("drive.connected")}
          </p>
          <p className="text-sm text-on-surface-variant">
            {t("drive.location")}
          </p>
          <Toggle
            label={t("drive.syncEnabled")}
            description={enabled ? undefined : t("drive.paused")}
            checked={enabled}
            onChange={(value) => void patch({ syncEnabled: value })}
          />
          <div className="flex flex-wrap gap-2">
            {drive.folderUrl && (
              <Button
                variant="ghost"
                onClick={() => void openExternal(drive.folderUrl ?? "")}
              >
                {t("drive.open")}
              </Button>
            )}
            <Button variant="danger" onClick={() => void disconnectDrive()}>
              {t("drive.disconnect")}
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
