import { openUrl } from "@tauri-apps/plugin-opener";

/** Öffnet eine Adresse im Standardbrowser. */
export async function openExternal(url: string): Promise<void> {
  await openUrl(url);
}
