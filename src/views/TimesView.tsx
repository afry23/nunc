import { useState } from "react";
import { ReviewPanel } from "../components/ReviewPanel";
import { TimesheetPanel } from "../components/TimesheetPanel";
import { SectionLabel, Segmented } from "../components/ui";
import { useT } from "../i18n";

type Panel = "times" | "review";

/** Zeiterfassung: Wochen-Timesheet, Tages-Timeline und Einträge – dazu der Rückblick (ADR 0025). */
export function TimesView() {
  const t = useT();
  const [panel, setPanel] = useState<Panel>("times");
  return (
    <section
      aria-labelledby="times-heading"
      className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <SectionLabel>{t("times.eyebrow")}</SectionLabel>
          <h1 id="times-heading" className="headline-md">
            {t("times.title")}
          </h1>
        </div>
        <Segmented
          label={t("review.view")}
          value={panel}
          options={[
            { value: "times", label: t("review.tab.times") },
            { value: "review", label: t("review.tab.review") },
          ]}
          onChange={setPanel}
        />
      </header>
      {panel === "times" ? <TimesheetPanel /> : <ReviewPanel />}
    </section>
  );
}
