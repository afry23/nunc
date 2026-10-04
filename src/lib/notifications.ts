import {
  commands,
  unwrap,
  type NotificationPermission,
  type NotificationStatus,
} from "./api";
import { isAndroid } from "./platform";

/** Brücke zu Android-Funktionen ohne Tauri-Plugin (`MainActivity.kt`, `addJavascriptInterface`). */
interface AndroidBridge {
  canScheduleExactAlarms(): boolean;
  openExactAlarmSettings(): void;
  openNotificationSettings(): void;
}

declare global {
  interface Window {
    NuncAndroid?: AndroidBridge;
  }
}

export type NotificationInfo = {
  permission: NotificationPermission;
  /** Android 12+: darf die App genaue Alarme setzen? Sonst verspätet sich das Timer-Ende (ADR 0004). */
  exactAlarms: boolean;
};

const bridge = (): AndroidBridge | undefined =>
  isAndroid() ? window.NuncAndroid : undefined;

function withExactAlarms(status: NotificationStatus): NotificationInfo {
  return {
    permission: status.permission,
    exactAlarms: bridge()?.canScheduleExactAlarms() ?? true,
  };
}

/** Aktueller Stand der Benachrichtigungen, ohne den Nutzer zu fragen. */
export async function loadNotificationInfo(): Promise<NotificationInfo> {
  return withExactAlarms(await unwrap(commands.getNotificationStatus()));
}

/** Fragt die Berechtigung an (Systemdialog, falls noch offen). */
export async function requestNotificationPermission(): Promise<NotificationInfo> {
  return withExactAlarms(
    await unwrap(commands.requestNotificationPermission()),
  );
}

/** Öffnet die Benachrichtigungs-Einstellungen der App (nur Android; iOS: Einstellungen-App). */
export function openNotificationSettings(): void {
  bridge()?.openNotificationSettings();
}

/** Öffnet „Wecker & Erinnerungen“ der App (Android 12+). */
export function openExactAlarmSettings(): void {
  bridge()?.openExactAlarmSettings();
}
