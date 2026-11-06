import { useState } from "react";
import { DayLog } from "../components/DayLog";
import { TimesheetPanel } from "../components/TimesheetPanel";
import { ChevronLeftIcon, ChevronRightIcon } from "../components/icons";
import { IconButton, SectionLabel, Segmented } from "../components/ui";
import { useIsWide } from "../hooks/useMediaQuery";
import { useT } from "../i18n";
import {
  addDays,
  parseIsoDate,
  shortDate,
  toIsoDate,
  weekdayLabel,
} from "../lib/format";

type Part = "day" | "week";

/**
 * „Verlauf“: Was ist wirklich passiert? Breit: Wochen-Timesheet und Tages-Timeline zusammen. Schmal: die Einträge
 * eines Tages als Liste, die Wochenmatrix auf Wunsch.
 */
export function LogView() {
  const t = useT();
  const wide = useIsWide();
  const [part, setPart] = useState<Part>("day");
  const [day, setDay] = useState(() => toIsoDate(new Date()));
  const move = (delta: number): void =>
    setDay(toIsoDate(addDays(parseIsoDate(day), delta)));

  return (
    <section
      aria-labelledby="log-heading"
      className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <SectionLabel>{t("log.eyebrow")}</SectionLabel>
          <h1 id="log-heading" className="headline-md">
            {t("log.title")}
          </h1>
        </div>
        {!wide && (
          <Segmented<Part>
            label={t("log.view")}
            value={part}
            onChange={setPart}
            options={[
              { value: "day", label: t("log.view.day") },
              { value: "week", label: t("log.view.week") },
            ]}
          />
        )}
      </header>
      {wide || part === "week" ? (
        <TimesheetPanel />
      ) : (
        <div className="flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <IconButton label={t("log.prevDay")} onClick={() => move(-1)}>
              <ChevronLeftIcon />
            </IconButton>
            <p className="font-display text-lg font-bold">
              {weekdayLabel(day)}, {shortDate(day)}
            </p>
            <IconButton label={t("log.nextDay")} onClick={() => move(1)}>
              <ChevronRightIcon />
            </IconButton>
          </div>
          <DayLog day={day} />
        </div>
      )}
    </section>
  );
}
