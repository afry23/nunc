import { useEffect, useState } from "react";
import { useT } from "../i18n";
import { commands } from "../lib/api";

/** Über nunc: zeigt die Version der laufenden App. */
export function AboutSection() {
  const t = useT();
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    void commands.appInfo().then(
      (info) => setVersion(info.version),
      () => setVersion(null),
    );
  }, []);

  return (
    <section
      id="settings-about"
      aria-labelledby="about-heading"
      className="flex scroll-mt-24 flex-col gap-4"
    >
      <h2 id="about-heading" className="font-display text-xl font-bold">
        {t("about.title")}
      </h2>
      <div className="rounded-2xl bg-surface-container-low p-5">
        <p data-testid="app-version">
          {version ? t("about.version", { version }) : t("about.unknown")}
        </p>
      </div>
    </section>
  );
}
