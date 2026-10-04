import { useEffect } from "react";
import { useT } from "../i18n";
import { formatBytes, formatDateTime, formatUtcDate } from "../lib/format";
import { isMobile } from "../lib/platform";
import { useBackupStore } from "../stores/backupStore";
import { useSettingsStore } from "../stores/settingsStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button } from "./ui";

/** Datensicherung: Status, „Jetzt sichern“, Backup-Ordner (Desktop) und Wiederherstellung. */
export function BackupSection() {
  const t = useT();
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
        {t("backup.title")}
      </h2>

      <div className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        <p data-testid="backup-status">
          {status?.lastBackupAt
            ? t("backup.last", { time: formatDateTime(status.lastBackupAt) })
            : t("backup.none")}
        </p>
        {status && (
          <p className="text-sm text-on-surface-variant">
            {t("backup.count", { count: status.backupCount })} ·{" "}
            {formatBytes(status.totalSizeBytes)}
          </p>
        )}
        {status?.isOutdated && (
          <p role="alert" className="text-orange-300">
            {t("backup.outdated")}
          </p>
        )}
        {!mobile && (
          <p className="text-sm text-on-surface-variant">
            {t("backup.dir", {
              dir: backupDir === "" ? t("backup.dirDefault") : backupDir,
            })}
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
                {t("backup.running")}
              </>
            ) : (
              t("backup.now")
            )}
          </Button>
          {!mobile && (
            <Button onClick={() => void actions.chooseDir()}>
              {t("backup.chooseDir")}
            </Button>
          )}
        </div>
        {lastResult && (
          <p role="status" className="text-sm text-on-surface-variant">
            {t("backup.saved", {
              path: lastResult.path,
              size: formatBytes(lastResult.sizeBytes),
            })}
            {lastResult.rotationError &&
              t("backup.rotationError", { error: lastResult.rotationError })}
          </p>
        )}
      </div>

      <div className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        <h3 className="font-semibold">{t("backup.restoreTitle")}</h3>
        {entries.length === 0 ? (
          <p className="text-on-surface-variant">{t("backup.empty")}</p>
        ) : (
          <ul aria-label={t("backup.list")} className="flex flex-col gap-2">
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
                    {t("backup.entryLine", {
                      category: t(`backup.category.${entry.category}`),
                      size: formatBytes(entry.sizeBytes),
                    })}
                  </span>
                </span>
                <span className="flex gap-2">
                  {mobile && (
                    <Button
                      aria-label={t("backup.exportLabel", {
                        date: formatUtcDate(entry.createdAt),
                      })}
                      onClick={() => void actions.exportBackup(entry)}
                    >
                      {t("backup.export")}
                    </Button>
                  )}
                  <Button
                    aria-label={t("backup.restoreLabel", {
                      date: formatUtcDate(entry.createdAt),
                    })}
                    onClick={() => actions.requestRestore(entry)}
                  >
                    {t("backup.restore")}
                  </Button>
                </span>
              </li>
            ))}
          </ul>
        )}
        <div>
          <Button onClick={() => void actions.requestRestoreFromFile()}>
            {mobile ? t("backup.importFile") : t("backup.restoreFromFile")}
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
          title={t("backup.confirm.title")}
          confirmLabel={t("backup.confirm.button")}
          danger
          busy={restoring}
          onConfirm={() => void actions.confirmRestore()}
          onCancel={actions.cancelRestore}
        >
          <p>{t("backup.confirm.body", { label: pending.label })}</p>
          <p>{t("backup.confirm.safety")}</p>
        </ConfirmDialog>
      )}
    </section>
  );
}
