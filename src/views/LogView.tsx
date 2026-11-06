import { TimesheetPanel } from "../components/TimesheetPanel";
import { SectionLabel } from "../components/ui";
import { useT } from "../i18n";

/** „Verlauf“: Was ist wirklich passiert? Wochen-Timesheet, Tages-Timeline und Einträge. */
export function LogView() {
  const t = useT();
  return (
    <section
      aria-labelledby="log-heading"
      className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-10"
    >
      <header>
        <SectionLabel>{t("log.eyebrow")}</SectionLabel>
        <h1 id="log-heading" className="headline-md">
          {t("log.title")}
        </h1>
      </header>
      <TimesheetPanel />
    </section>
  );
}
