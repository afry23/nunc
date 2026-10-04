import { useState } from "react";
import type { ConnectionTestResult, WebDavConfig } from "../lib/api";
import { isHttpUrl } from "../lib/sync";
import { useSyncStore } from "../stores/syncStore";
import { useUiStore } from "../stores/uiStore";
import { Button } from "./ui";

const FIELD =
  "w-full min-w-0 min-h-11 rounded-lg bg-surface-container-highest px-3 focus:bg-secondary-container disabled:opacity-40";

/**
 * Verbindungsdaten für WebDAV. Änderungen gelten erst nach „Speichern“; das Passwort kommt nie vom Backend
 * zurück (leer lassen = gespeichertes behalten).
 */
export function WebDavForm({
  stored,
  hasPassword,
  disabled,
}: {
  stored: WebDavConfig;
  hasPassword: boolean;
  disabled: boolean;
}) {
  const [draft, setDraft] = useState<WebDavConfig>(stored);
  const [password, setPassword] = useState("");
  const [revealed, setRevealed] = useState(false);
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<ConnectionTestResult | null>(null);
  const save = useSyncStore((s) => s.saveWebdav);
  const test = useSyncStore((s) => s.testWebdav);
  const notify = useUiStore((s) => s.notify);

  const set = <K extends keyof WebDavConfig>(
    key: K,
    value: WebDavConfig[K],
  ): void => {
    setDraft((d) => ({ ...d, [key]: value }));
    setResult(null);
  };
  const validUrl = isHttpUrl(draft.url);
  const dirty =
    JSON.stringify(draft) !== JSON.stringify(stored) || password !== "";
  const typedPassword = password === "" ? null : password;

  const runTest = async (): Promise<void> => {
    setTesting(true);
    setResult(null);
    setResult(await test(draft, typedPassword));
    setTesting(false);
  };

  const submit = async (): Promise<void> => {
    if (await save(draft, typedPassword)) {
      setPassword("");
      notify("WebDAV-Einstellungen gespeichert.");
    }
  };

  return (
    <form
      aria-label="WebDAV"
      onSubmit={(event) => {
        event.preventDefault();
        if (validUrl) void submit();
      }}
      className="flex min-w-0 flex-col gap-4"
    >
      <div className="flex min-w-0 flex-col gap-2">
        <label htmlFor="webdav-url" className="font-semibold">
          Server-URL
        </label>
        <input
          id="webdav-url"
          type="url"
          inputMode="url"
          autoComplete="off"
          placeholder="https://cloud.example.org/remote.php/dav/files/name"
          disabled={disabled}
          value={draft.url}
          aria-invalid={draft.url !== "" && !validUrl}
          onChange={(event) => set("url", event.target.value)}
          className={FIELD}
        />
        {draft.url !== "" && !validUrl && (
          <span role="alert" className="text-sm text-red-300">
            Die Adresse muss mit http:// oder https:// beginnen.
          </span>
        )}
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <label className="flex min-w-0 flex-col gap-2">
          <span className="font-semibold">Authentifizierung</span>
          <select
            disabled={disabled}
            value={draft.authMethod}
            onChange={(event) =>
              set(
                "authMethod",
                event.target.value as WebDavConfig["authMethod"],
              )
            }
            className={FIELD}
          >
            <option value="basic">Basic</option>
            <option value="digest">Digest</option>
          </select>
        </label>
        <label className="flex min-w-0 flex-col gap-2">
          <span className="font-semibold">Benutzername</span>
          <input
            type="text"
            autoComplete="off"
            autoCapitalize="none"
            disabled={disabled}
            value={draft.username}
            onChange={(event) => set("username", event.target.value)}
            className={FIELD}
          />
        </label>
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="flex min-w-0 flex-col gap-2">
          <label htmlFor="webdav-password" className="font-semibold">
            Passwort
          </label>
          <div className="flex min-w-0 flex-wrap gap-2">
            <input
              id="webdav-password"
              type={revealed ? "text" : "password"}
              autoComplete="new-password"
              disabled={disabled}
              value={password}
              placeholder={
                hasPassword ? "gespeichert – leer lassen zum Behalten" : ""
              }
              onChange={(event) => {
                setPassword(event.target.value);
                setResult(null);
              }}
              className={`${FIELD} flex-1 basis-40`}
            />
            <Button
              aria-pressed={revealed}
              disabled={disabled}
              onClick={() => setRevealed((r) => !r)}
            >
              {revealed ? "Passwort verbergen" : "Passwort anzeigen"}
            </Button>
          </div>
        </div>
        <label className="flex min-w-0 flex-col gap-2">
          <span className="font-semibold">Remote-Pfad</span>
          <input
            type="text"
            autoComplete="off"
            autoCapitalize="none"
            disabled={disabled}
            value={draft.remotePath}
            onChange={(event) => set("remotePath", event.target.value)}
            className={FIELD}
          />
        </label>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button
          disabled={disabled || !validUrl || testing}
          aria-busy={testing}
          onClick={() => void runTest()}
        >
          {testing ? "Teste …" : "Verbindung testen"}
        </Button>
        <Button
          type="submit"
          variant="primary"
          aria-label="WebDAV-Einstellungen speichern"
          disabled={disabled || !validUrl || !dirty}
        >
          Speichern
        </Button>
      </div>
      {result && (
        <p
          role={result.success ? "status" : "alert"}
          className={result.success ? "text-primary" : "text-red-300"}
        >
          {result.success
            ? `Verbindung erfolgreich: ${result.message}`
            : result.message}
        </p>
      )}
    </form>
  );
}
