import { Link } from "@tanstack/react-router";
import { ReviewPanel } from "../components/ReviewPanel";
import { SectionLabel } from "../components/ui";
import { useT } from "../i18n";

/** „Rückblick“: Ergebnisse, Arbeit und Reflexion eines Zeitraums (ADR 0025). Die Timerstatistik hängt dahinter. */
export function ReviewView() {
  const t = useT();
  return (
    <section
      aria-labelledby="review-heading"
      className="mx-auto flex max-w-5xl flex-col gap-8 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <SectionLabel>{t("review.eyebrow")}</SectionLabel>
          <h1 id="review-heading" className="headline-md">
            {t("review.title")}
          </h1>
        </div>
        <Link
          to="/stats"
          className="inline-flex min-h-11 items-center rounded-lg px-3 font-semibold text-on-surface-variant hover:bg-surface-container-high"
        >
          {t("review.workStyle")}
        </Link>
      </header>
      <ReviewPanel />
    </section>
  );
}
