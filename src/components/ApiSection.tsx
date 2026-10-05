import { useEffect, useId, useState } from "react";
import { useT } from "../i18n";
import { commands, events, unwrap, type ApiStatus } from "../lib/api";
import { useUiStore } from "../stores/uiStore";
import { ConfirmDialog } from "./ConfirmDialog";
import { Button, Toggle } from "./ui";

/**
 * Lokale HTTP-Schnittstelle (nur Desktop, ADR 0013): Ein/Aus und Port gehören zum Einstellungsentwurf und gelten mit
 * „Speichern“; Zustand und Token wirken sofort. Das Token steht nur nach „Token anzeigen“ auf dem Bildschirm.
 */
export function ApiSection({
  enabled,
  port,
  storedEnabled,
  storedPort,
  onEnabled,
  onPort,
}: {
  enabled: boolean;
  port: number;
  storedEnabled: boolean;
  storedPort: number;
  onEnabled: (value: boolean) => void;
  onPort: (value: number) => void;
}) {
  const t = useT();
  const portId = useId();
  const tokenId = useId();
  const { fail, notify } = useUiStore.getState();
  const [status, setStatus] = useState<ApiStatus | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);
  const unsaved = enabled !== storedEnabled || port !== storedPort;

  useEffect(() => {
    unwrap(commands.getApiStatus())
      .then(setStatus)
      .catch((error: unknown) => useUiStore.getState().fail(error));
  }, [storedEnabled, storedPort]);

  // Der Server startet beim App-Start im Hintergrund und folgt einem Restore: das Backend meldet jeden Wechsel
  useEffect(() => {
    const unlisten = events.apiStatus.listen((event) =>
      setStatus(event.payload),
    );
    return () => void unlisten.then((fn) => fn());
  }, []);

  const toggleToken = async (): Promise<void> => {
    if (token !== null) return setToken(null);
    try {
      setToken(await unwrap(commands.getApiToken()));
    } catch (error) {
      fail(error);
    }
  };

  const regenerate = async (): Promise<void> => {
    setConfirming(false);
    try {
      setToken(await unwrap(commands.regenerateApiToken()));
      notify(t("api.token.regenerated"));
    } catch (error) {
      fail(error);
    }
  };

  const shownPort = status?.port ?? storedPort;
  return (
    <div className="flex flex-col gap-4 rounded-2xl bg-surface-container-low p-5">
      <p className="text-on-surface-variant">{t("api.description")}</p>
      <Toggle label={t("api.enable")} checked={enabled} onChange={onEnabled} />
      <label htmlFor={portId} className="flex flex-col gap-2">
        <span className="font-semibold">{t("api.port")}</span>
        <input
          id={portId}
          type="number"
          inputMode="numeric"
          min={1024}
          max={65535}
          step={1}
          value={Number.isNaN(port) ? "" : port}
          onChange={(event) => onPort(event.target.valueAsNumber)}
          className="font-display h-12 w-32 rounded-sm bg-surface-container-highest px-3 text-xl font-bold focus:bg-secondary-container"
        />
      </label>

      {unsaved && (
        <p className="text-sm text-on-surface-variant">{t("api.unsaved")}</p>
      )}
      {status && !unsaved && (
        <p role="status" className="text-sm">
          {status.running
            ? t("api.status.running", { port: shownPort })
            : status.error
              ? t("api.status.error", { message: status.error })
              : t("api.status.off")}
        </p>
      )}

      {status?.running && !unsaved && (
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap gap-2">
            <Button onClick={() => void toggleToken()}>
              {token === null ? t("api.token.show") : t("api.token.hide")}
            </Button>
            <Button onClick={() => setConfirming(true)}>
              {t("api.token.regenerate")}
            </Button>
          </div>
          {token !== null && (
            <label htmlFor={tokenId} className="flex flex-col gap-1">
              <span className="font-semibold">{t("api.token.label")}</span>
              <input
                id={tokenId}
                readOnly
                value={token}
                onFocus={(event) => event.target.select()}
                className="min-h-11 rounded-lg bg-surface-container-highest px-3 font-mono text-sm"
              />
            </label>
          )}
          <p className="text-sm text-on-surface-variant">{t("api.example")}</p>
          <code className="block overflow-x-auto rounded-lg bg-surface-container-highest p-3 text-sm whitespace-pre">
            {`curl -H "Authorization: Bearer <${t("api.token.label")}>" \\\n  http://127.0.0.1:${shownPort}/v1/todos`}
          </code>
        </div>
      )}

      {confirming && (
        <ConfirmDialog
          title={t("api.token.confirm.title")}
          confirmLabel={t("api.token.confirm.button")}
          danger
          onConfirm={() => void regenerate()}
          onCancel={() => setConfirming(false)}
        >
          <p>{t("api.token.confirm.body")}</p>
        </ConfirmDialog>
      )}
    </div>
  );
}
