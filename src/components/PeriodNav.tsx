import { ChevronLeftIcon, ChevronRightIcon } from "./icons";
import { Button, IconButton, Segmented } from "./ui";
import { useT } from "../i18n";
import { REVIEW_KINDS, rangeLabel, type ReviewKind } from "../lib/review";

/** Zeitraumwahl für Auswertung und Rückblick: Art (Tag bis Jahr), Blättern und Sprung zum aktuellen Zeitraum. */
export function PeriodNav({
  kind,
  periodsAgo,
  from,
  to,
  onKind,
  onPeriodsAgo,
}: {
  kind: ReviewKind;
  periodsAgo: number;
  from: string;
  to: string;
  onKind: (kind: ReviewKind) => void;
  onPeriodsAgo: (periodsAgo: number) => void;
}) {
  const t = useT();
  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex items-center gap-1">
        <IconButton
          label={t("review.prev")}
          onClick={() => onPeriodsAgo(periodsAgo + 1)}
        >
          <ChevronLeftIcon />
        </IconButton>
        <p className="font-display min-w-44 text-center text-lg font-bold">
          {rangeLabel(from, to)}
        </p>
        <IconButton
          label={t("review.next")}
          onClick={() => onPeriodsAgo(periodsAgo - 1)}
        >
          <ChevronRightIcon />
        </IconButton>
        <Button
          variant="ghost"
          disabled={periodsAgo === 0}
          onClick={() => onPeriodsAgo(0)}
        >
          {t("review.today")}
        </Button>
      </div>
      <Segmented
        label={t("review.kind")}
        value={kind}
        options={REVIEW_KINDS.map((value) => ({
          value,
          label: t(`review.kind.${value}`),
        }))}
        onChange={onKind}
      />
    </div>
  );
}
