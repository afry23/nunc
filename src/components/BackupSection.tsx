import { useEffect } from "react";
import { formatBytes, formatDateTime, formatUtcDate } from "../lib/format";
import { isMobile } from "../lib/platform";
import type { BackupCategory } from "../lib/api";
import { useBackupStore } from "../stores/backupStore";
import { useSettingsStore } from "../stores/settingsStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button } from "./ui";

const CATEGORY_LABELS: Record<BackupCategory, string> = {
  daily: "Täglich",
  weekly: "Wöchentlich",
  monthly: "Monatlich",
};

/** Datensicherung: Status, „Jetzt sichern“, Backup-Ordner (Desktop) und Wiederherstellung. */
export function BackupSection() {
  const {
    status,
    entries,
    running,
    lastResult,
    pending,
    restoring,
    restoreError,
  } = useBackupStore();
  const backupDir = useSettingsStore((s) => s.settings?.backupDir ?? "");
  const mobile = isMobile();

  useEffect(() => {
    void useBackupStore.getState().load();
  }, []);

  const actions = useBackupStore.getState();

  return (
    <section aria-labelledby="backup-heading" className="flex flex-col gap-4">
      <h2 id="backup-heading" className="font-display text-xl font-bold">
        Datensicherung
      </h2>

      <div className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        <p data-testid="backup-status">
          {status?.lastBackupAt
            ? `Letztes Backup: ${formatDateTime(status.lastBackupAt)}`
            : "Noch kein Backup vorhanden."}
        </p>
        {status && (
          <p className="text-sm text-on-surface-variant">
            {status.backupCount}{" "}
            {status.backupCount === 1 ? "Backup" : "Backups"} ·{" "}
            {formatBytes(status.totalSizeBytes)}
          </p>
        )}
        {status?.isOutdated && (
          <p role="alert" className="text-orange-300">
            Das letzte Backup ist älter als zwei Tage.
          </p>
        )}
        {!mobile && (
          <p className="text-sm text-on-surface-variant">
            Backup-Ordner: {backupDir === "" ? "Standardordner" : backupDir}
          </p>
        )}
        <div className="flex flex-wrap gap-2">
          <Button
            variant="primary"
            disabled={running}
            aria-busy={running}
            onClick={() => void actions.backupNow()}
          >
            {running ? (
              <>
                <span
                  aria-hidden="true"
                  className="size-4 animate-spin rounded-full border-2 border-current border-t-transparent"
                />
                Sichere …
              </>
            ) : (
              "Jetzt sichern"
            )}
          </Button>
          {!mobile && (
            <Button onClick={() => void actions.chooseDir()}>
              Backup-Ordner wählen
            </Button>
          )}
        </div>
        {lastResult && (
          <p role="status" className="text-sm text-on-surface-variant">
            Gesichert: {lastResult.path} ({formatBytes(lastResult.sizeBytes)})
            {lastResult.rotationError &&
              ` – Alte Backups konnten nicht aufgeräumt werden: ${lastResult.rotationError}`}
          </p>
        )}
      </div>

      <div className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        <h3 className="font-semibold">Wiederherstellen</h3>
        {entries.length === 0 ? (
          <p className="text-on-surface-variant">Keine Backups vorhanden.</p>
        ) : (
          <ul aria-label="Backups" className="flex flex-col gap-2">
            {entries.map((entry) => (
              <li
                key={entry.path}
                className="flex flex-wrap items-center justify-between gap-3 rounded-xl bg-surface-container-high px-4 py-2"
              >
                <span>
                  <span className="font-semibold">
                    {formatUtcDate(entry.createdAt)}
                  </span>{" "}
                  <span className="text-on-surface-variant">
                    {CATEGORY_LABELS[entry.category]} ·{" "}
                    {formatBytes(entry.sizeBytes)}
                  </span>
                </span>
                <Button
                  aria-label={`Backup vom ${formatUtcDate(entry.createdAt)} wiederherstellen`}
                  onClick={() => actions.requestRestore(entry)}
                >
                  Wiederherstellen
                </Button>
              </li>
            ))}
          </ul>
        )}
        <div>
          <Button onClick={() => void actions.requestRestoreFromFile()}>
            {mobile ? "Backup-Datei importieren" : "Aus Datei wiederherstellen"}
          </Button>
        </div>
        {restoreError && (
          <p role="alert" className="text-red-300">
            {restoreError}
          </p>
        )}
      </div>

      {pending && (
        <ConfirmDialog
          title="Daten wiederherstellen?"
          confirmLabel="Jetzt wiederherstellen"
          danger
          busy={restoring}
          onConfirm={() => void actions.confirmRestore()}
          onCancel={actions.cancelRestore}
        >
          <p>
            Alle aktuellen Daten werden durch den Stand von{" "}
            <strong className="text-on-surface">{pending.label}</strong>{" "}
            ersetzt. Das lässt sich nicht rückgängig machen.
          </p>
          <p>Vorher wird automatisch ein Notfall-Backup angelegt.</p>
        </ConfirmDialog>
      )}
    </section>
  );
}
