import { useState } from "react";
import { useT } from "../i18n";
import { openExternal } from "../lib/links";
import { useSyncStore } from "../stores/syncStore";
import { useUiStore } from "../stores/uiStore";
import { Button } from "./ui";

const FIELD =
  "w-full min-w-0 min-h-11 rounded-lg bg-surface-container-highest px-3 focus:bg-secondary-container";

/** Anmeldedaten in der Google Cloud Console (Schritt 5 der Anleitung). */
export const GOOGLE_CONSOLE_URL =
  "https://console.cloud.google.com/apis/credentials";

const GUIDE_STEPS = [
  "drive.guide.step1",
  "drive.guide.step2",
  "drive.guide.step3",
  "drive.guide.step4",
  "drive.guide.step5",
] as const;

/**
 * Der eigene OAuth-Client für Google (ADR 0019): Client-ID und -Secret eintragen, mit Anleitung. Das Secret
 * kommt nie vom Backend zurück; nach dem Speichern ist das Feld wieder leer.
 */
export function GoogleClientForm({ storedId }: { storedId: string | null }) {
  const t = useT();
  const [clientId, setClientId] = useState(storedId ?? "");
  const [secret, setSecret] = useState("");
  const [guideOpen, setGuideOpen] = useState(false);
  const error = useSyncStore((s) => s.clientError);
  const { saveGoogleClient, clearGoogleClient } = useSyncStore.getState();
  const notify = useUiStore((s) => s.notify);
  const complete = clientId.trim() !== "" && secret.trim() !== "";

  const submit = async (): Promise<void> => {
    if (await saveGoogleClient(clientId, secret)) {
      setSecret("");
      notify(t("drive.client.saved"));
    }
  };

  return (
    <form
      aria-label={t("drive.client.title")}
      onSubmit={(event) => {
        event.preventDefault();
        if (complete) void submit();
      }}
      className="flex min-w-0 flex-col gap-3"
    >
      <p className="text-sm text-on-surface-variant">
        {t("drive.client.intro")}
      </p>
      {storedId && (
        <p className="text-sm break-all">
          {t("drive.client.current", { id: storedId })}
        </p>
      )}
      <div className="flex min-w-0 flex-col gap-2">
        <label htmlFor="google-client-id" className="font-semibold">
          {t("drive.client.id")}
        </label>
        <input
          id="google-client-id"
          type="text"
          autoComplete="off"
          spellCheck={false}
          placeholder="123-abc.apps.googleusercontent.com"
          value={clientId}
          onChange={(event) => setClientId(event.target.value)}
          className={FIELD}
        />
      </div>
      <div className="flex min-w-0 flex-col gap-2">
        <label htmlFor="google-client-secret" className="font-semibold">
          {t("drive.client.secret")}
        </label>
        <input
          id="google-client-secret"
          type="password"
          autoComplete="off"
          value={secret}
          onChange={(event) => setSecret(event.target.value)}
          className={FIELD}
        />
      </div>
      {error && (
        <p role="alert" className="text-red-300">
          {error}
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button type="submit" variant="primary" disabled={!complete}>
          {t("drive.client.save")}
        </Button>
        {storedId && (
          <Button variant="ghost" onClick={() => void clearGoogleClient()}>
            {t("drive.client.remove")}
          </Button>
        )}
      </div>
      <div className="flex min-w-0 flex-col gap-2">
        <Button
          variant="ghost"
          aria-expanded={guideOpen}
          onClick={() => setGuideOpen((open) => !open)}
        >
          {t("drive.guide.toggle")}
        </Button>
        {guideOpen && (
          <div className="flex min-w-0 flex-col gap-2 text-sm">
            <ol className="list-decimal space-y-1 pl-5">
              {GUIDE_STEPS.map((key) => (
                <li key={key}>{t(key)}</li>
              ))}
            </ol>
            <p>{t("drive.guide.devices")}</p>
            <p className="text-on-surface-variant">
              {t("drive.guide.alternative")}
            </p>
            <div>
              <Button
                variant="ghost"
                onClick={() => void openExternal(GOOGLE_CONSOLE_URL)}
              >
                {t("drive.guide.console")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </form>
  );
}
