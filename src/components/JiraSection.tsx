import { useEffect, useState } from "react";
import { useT } from "../i18n";
import type { ConnectionTestResult, JiraConfig } from "../lib/api";
import { isHttpUrl } from "../lib/sync";
import { useJiraStore } from "../stores/jiraStore";
import { useUiStore } from "../stores/uiStore";
import { Button, Segmented } from "./ui";

const FIELD =
  "w-full min-w-0 min-h-11 rounded-lg bg-surface-container-highest px-3 focus:bg-secondary-container";

/** Zugang zu Jira für den Ticket-Import (ADR 0038). Gilt pro Gerät; das Token kommt nie vom Backend zurück. */
export function JiraSection() {
  const t = useT();
  const settings = useJiraStore((s) => s.settings);
  useEffect(() => {
    void useJiraStore.getState().load();
  }, []);

  return (
    <section
      id="settings-jira"
      aria-labelledby="jira-heading"
      className="flex min-w-0 scroll-mt-24 flex-col gap-4"
    >
      <h2 id="jira-heading" className="font-display text-xl font-bold">
        {t("jira.title")}
      </h2>
      <div className="flex min-w-0 flex-col gap-3 rounded-2xl bg-surface-container-low p-5">
        <p className="text-sm text-on-surface-variant">{t("jira.intro")}</p>
        {settings && (
          <JiraForm
            key={JSON.stringify(settings)}
            stored={settings.config}
            hasToken={settings.hasToken}
          />
        )}
      </div>
    </section>
  );
}

/** Änderungen gelten erst nach „Speichern“; ein leeres Token-Feld behält das gespeicherte. */
function JiraForm({
  stored,
  hasToken,
}: {
  stored: JiraConfig;
  hasToken: boolean;
}) {
  const t = useT();
  const [draft, setDraft] = useState<JiraConfig>(stored);
  const [token, setToken] = useState("");
  const [revealed, setRevealed] = useState(false);
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<ConnectionTestResult | null>(null);
  const { save, test, clear } = useJiraStore.getState();
  const notify = useUiStore((s) => s.notify);

  const set = <K extends keyof JiraConfig>(
    key: K,
    value: JiraConfig[K],
  ): void => {
    setDraft((d) => ({ ...d, [key]: value }));
    setResult(null);
  };
  const validUrl = isHttpUrl(draft.baseUrl);
  const dirty =
    JSON.stringify(draft) !== JSON.stringify(stored) || token !== "";
  const typedToken = token === "" ? null : token;
  const cloud = draft.kind === "cloud";

  const runTest = async (): Promise<void> => {
    setTesting(true);
    setResult(null);
    setResult(await test(draft, typedToken));
    setTesting(false);
  };

  const submit = async (): Promise<void> => {
    if (await save(draft, typedToken)) {
      setToken("");
      notify(t("jira.saved"));
    }
  };

  const remove = async (): Promise<void> => {
    if (await clear()) notify(t("jira.removed"));
  };

  return (
    <form
      aria-label="Jira"
      onSubmit={(event) => {
        event.preventDefault();
        if (validUrl) void submit();
      }}
      className="flex min-w-0 flex-col gap-4"
    >
      <Segmented
        label={t("jira.kind")}
        value={draft.kind}
        options={[
          { value: "cloud", label: t("jira.kind.cloud") },
          { value: "data_center", label: t("jira.kind.dataCenter") },
        ]}
        onChange={(kind) => set("kind", kind)}
      />
      <div className="flex min-w-0 flex-col gap-2">
        <label htmlFor="jira-url" className="font-semibold">
          {t("jira.url")}
        </label>
        <input
          id="jira-url"
          type="url"
          inputMode="url"
          autoComplete="off"
          placeholder={
            cloud ? "https://firma.atlassian.net" : "https://jira.firma.de"
          }
          value={draft.baseUrl}
          aria-invalid={draft.baseUrl !== "" && !validUrl}
          onChange={(event) => set("baseUrl", event.target.value)}
          className={FIELD}
        />
        {draft.baseUrl !== "" && !validUrl && (
          <span role="alert" className="text-sm text-red-300">
            {t("jira.urlInvalid")}
          </span>
        )}
      </div>
      {cloud && (
        <div className="flex min-w-0 flex-col gap-2">
          <label htmlFor="jira-email" className="font-semibold">
            {t("jira.email")}
          </label>
          <input
            id="jira-email"
            type="email"
            autoComplete="off"
            autoCapitalize="none"
            value={draft.email}
            onChange={(event) => set("email", event.target.value)}
            className={FIELD}
          />
        </div>
      )}
      <div className="flex min-w-0 flex-col gap-2">
        <label htmlFor="jira-token" className="font-semibold">
          {t("jira.token")}
        </label>
        <div className="flex min-w-0 flex-wrap gap-2">
          <input
            id="jira-token"
            type={revealed ? "text" : "password"}
            autoComplete="new-password"
            value={token}
            placeholder={hasToken ? t("jira.tokenStored") : ""}
            onChange={(event) => {
              setToken(event.target.value);
              setResult(null);
            }}
            className={`${FIELD} flex-1 basis-40`}
          />
          <Button
            aria-pressed={revealed}
            onClick={() => setRevealed((r) => !r)}
          >
            {revealed ? t("jira.hide") : t("jira.show")}
          </Button>
        </div>
        <span className="text-sm text-on-surface-variant">
          {cloud ? t("jira.tokenHint.cloud") : t("jira.tokenHint.dataCenter")}
        </span>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button
          disabled={!validUrl || testing}
          aria-busy={testing}
          onClick={() => void runTest()}
        >
          {testing ? t("jira.testing") : t("jira.test")}
        </Button>
        <Button
          type="submit"
          variant="primary"
          aria-label={t("jira.saveLabel")}
          disabled={!validUrl || !dirty}
        >
          {t("common.save")}
        </Button>
        {(hasToken || stored.baseUrl !== "") && (
          <Button onClick={() => void remove()}>{t("jira.remove")}</Button>
        )}
      </div>
      {result && (
        <p
          role={result.success ? "status" : "alert"}
          className={result.success ? "text-primary" : "text-red-300"}
        >
          {result.success ? t("jira.success") : result.message}
        </p>
      )}
    </form>
  );
}
