import { platform } from "@tauri-apps/plugin-os";

const MOBILE_AGENT = /Android|iPhone|iPad|iPod/i;

/**
 * Mobile Plattform (Android/iOS): dort fehlen Tray, Tastenkürzel, Mini-Timer und Backup-Ordner.
 *
 * In der Tauri-Laufzeit entscheidet `plugin-os` (erkennt auch iPadOS, dessen User-Agent „Macintosh“ lautet).
 * Ohne Laufzeit (Browser-Vorschau, Tests) gilt der User-Agent; ein übergebener `userAgent` hat Vorrang.
 */
export function isMobile(userAgent?: string): boolean {
  if (userAgent === undefined) {
    try {
      const current = platform();
      return current === "android" || current === "ios";
    } catch {
      // Plugin nicht initialisiert
    }
  }
  return MOBILE_AGENT.test(userAgent ?? navigator.userAgent);
}

/** `true` auf Android (für Android-spezifische Hinweise wie Exact-Alarm). */
export function isAndroid(userAgent?: string): boolean {
  if (userAgent === undefined) {
    try {
      return platform() === "android";
    } catch {
      // Plugin nicht initialisiert
    }
  }
  return /Android/i.test(userAgent ?? navigator.userAgent);
}
