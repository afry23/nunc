import { TimesheetPanel } from "../components/TimesheetPanel";
import { SectionLabel } from "../components/ui";
import { useT } from "../i18n";

/** Zeiterfassung: Wochen-Timesheet, Tages-Timeline und Einträge nachtragen, bearbeiten, löschen. */
export function TimesView() {
  const t = useT();
  return (
    <section
      aria-labelledby="times-heading"
      className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-10"
    >
      <header>
        <SectionLabel>{t("times.eyebrow")}</SectionLabel>
        <h1 id="times-heading" className="headline-md">
          {t("times.title")}
        </h1>
      </header>
      <TimesheetPanel />
    </section>
  );
}
