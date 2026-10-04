import { InfoIcon } from "./icons";
import { Button } from "./ui";
import { isAndroid, isMobile } from "../lib/platform";
import {
  openExactAlarmSettings,
  openNotificationSettings,
} from "../lib/notifications";
import { useNotificationStore } from "../stores/notificationStore";
import { useSettingsStore } from "../stores/settingsStore";

/**
 * Mobil: erklärt, warum das Timer-Ende im Hintergrund fehlen oder sich verspäten kann, und bietet den Weg
 * zur Abhilfe. Mit `withRequest` zusätzlich die Aufforderung, die noch offene Berechtigung zu erteilen.
 * Zeigt nichts, wenn alles in Ordnung ist, die Benachrichtigungen aus sind oder es nicht mobil ist.
 */
export function NotificationHint({
  withRequest = false,
}: {
  withRequest?: boolean;
}) {
  const info = useNotificationStore((s) => s.info);
  const request = useNotificationStore((s) => s.request);
  const enabled = useSettingsStore((s) => s.settings?.notificationsEnabled);
  if (!isMobile() || !enabled || !info) return null;

  const blocked = info.permission === "denied";
  const open = withRequest && info.permission === "prompt";
  const lateAlarms = isAndroid() && !info.exactAlarms;
  if (!blocked && !open && !lateAlarms) return null;

  return (
    <div
      role="note"
      className="flex flex-col gap-3 rounded-xl bg-surface-container-low p-4 text-orange-300"
    >
      {open && (
        <>
          <p className="flex items-center gap-2">
            <InfoIcon />
            Erlaube Benachrichtigungen, damit nunc das Ende des Timers auch im
            Hintergrund meldet.
          </p>
          <Button onClick={() => void request()}>
            Benachrichtigungen erlauben
          </Button>
        </>
      )}
      {blocked && (
        <>
          <p className="flex items-center gap-2">
            <InfoIcon />
            Benachrichtigungen sind für nunc ausgeschaltet. Der Timer meldet
            sein Ende nur bei geöffneter App.
          </p>
          {isAndroid() && (
            <Button onClick={openNotificationSettings}>
              Systemeinstellungen öffnen
            </Button>
          )}
        </>
      )}
      {lateAlarms && (
        <>
          <p className="flex items-center gap-2">
            <InfoIcon />
            Genaue Alarme sind ausgeschaltet. Das Timer-Ende kann sich um
            mehrere Minuten verspäten.
          </p>
          <Button onClick={openExactAlarmSettings}>Alarme erlauben</Button>
        </>
      )}
    </div>
  );
}
