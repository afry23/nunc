import { openExternal } from "../lib/links";
import { useSettingsStore } from "../stores/settingsStore";
import { useSyncStore } from "../stores/syncStore";
import { Button, Toggle } from "./ui";

/** Verbindung zu Google Drive: anmelden, abmelden, pausieren, Ablageort (nur Desktop). */
export function DriveSection() {
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
          <p data-testid="drive-state">Nicht verbunden</p>
          {!drive.configured && (
            <p className="text-sm text-orange-300">
              In diesem Build sind keine Google-Zugangsdaten hinterlegt.
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
              disabled={connecting || !drive.configured}
              aria-busy={connecting}
              onClick={() => void connectDrive()}
            >
              {connecting
                ? "Warte auf den Browser …"
                : "Mit Google Drive verbinden"}
            </Button>
          </div>
        </>
      ) : (
        <>
          <p data-testid="drive-state">
            {drive.email ? `Verbunden als ${drive.email}` : "Verbunden"}
          </p>
          <p className="text-sm text-on-surface-variant">
            Ablage in Google Drive: nunc/nunc-sync.json
          </p>
          <Toggle
            label="Synchronisierung aktiv"
            description={
              enabled
                ? undefined
                : "Pausiert: Lokale Daten bleiben erhalten. Die Sync-Datei auf Google Drive wird nicht gelöscht."
            }
            checked={enabled}
            onChange={(value) => void patch({ syncEnabled: value })}
          />
          <div className="flex flex-wrap gap-2">
            {drive.folderUrl && (
              <Button
                variant="ghost"
                onClick={() => void openExternal(drive.folderUrl ?? "")}
              >
                In Drive öffnen
              </Button>
            )}
            <Button variant="danger" onClick={() => void disconnectDrive()}>
              Verbindung trennen
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
