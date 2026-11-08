import { Link } from "@tanstack/react-router";
import { useEffect, useState, type FormEvent } from "react";
import { DownloadIcon } from "./icons";
import { Button } from "./ui";
import { currentLanguage, useT } from "../i18n";
import { commands, errorMessage, unwrap, type Review } from "../lib/api";
import { toIsoDate } from "../lib/format";
import { isWeekReviewTime, reviewRange, type ReviewKind } from "../lib/review";
import { useTodoStore } from "../stores/todoStore";
import { useUiStore } from "../stores/uiStore";
import { CollapsibleSection } from "./CollapsibleSection";
import {
  DayNotesCard,
  REFLECTION_FIELD_ID,
  ReflectionCard,
  ResultsCard,
} from "./ReviewJournal";
import { PeriodNav } from "./PeriodNav";
import { ActivityList, TaskList } from "./WorkLists";

type Loaded = { review: Review } | { error: string };

/** Folgeaufgabe für den nächsten Zeitraum: legt eine Aufgabe an (todo.txt-Syntax). */
function NextPeriodCard() {
  const t = useT();
  const [text, setText] = useState("");
  const submit = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    if (text.trim() === "") return;
    if (await useTodoStore.getState().add(text)) {
      setText("");
      useUiStore.getState().notify(t("review.next.created"));
    }
  };
  return (
    <CollapsibleSection title={t("review.nextTitle")} defaultOpen>
      <form
        onSubmit={(event) => void submit(event)}
        className="flex flex-col gap-2"
      >
        <input
          type="text"
          value={text}
          autoComplete="off"
          aria-label={t("review.next.label")}
          placeholder={t("review.next.placeholder")}
          onChange={(event) => setText(event.target.value)}
          className="min-h-11 w-full rounded-lg bg-surface-container-high px-3"
        />
        <div>
          <Button type="submit" disabled={text.trim() === ""}>
            {t("review.next.create")}
          </Button>
        </div>
      </form>
    </CollapsibleSection>
  );
}

/** Rückblick (ADR 0025): Was wurde bearbeitet und ist es erledigt, dazu Erfolge, Notizen und Reflexion eines Zeitraums. */
export function ReviewPanel() {
  const t = useT();
  const notify = useUiStore((s) => s.notify);
  const [kind, setKind] = useState<ReviewKind>("week");
  const [periodsAgo, setPeriodsAgo] = useState(0);
  const [today] = useState(() => new Date());
  const { from, to } = reviewRange(kind, today, periodsAgo);
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [version, setVersion] = useState(0);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const review = await unwrap(commands.getReview(from, to));
        if (!cancelled) setLoaded({ review });
      } catch (error) {
        if (!cancelled) setLoaded({ error: errorMessage(error) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [from, to, version]);

  const exportMarkdown = async (): Promise<void> => {
    try {
      const path = await unwrap(
        commands.saveReviewMarkdown(from, to, currentLanguage()),
      );
      if (path) notify(t("review.exportSaved", { path }));
    } catch (error) {
      useUiStore.getState().fail(error);
    }
  };

  const review = loaded && "review" in loaded ? loaded.review : null;
  const reloadReview = (): void => setVersion((value) => value + 1);

  return (
    <div className="flex flex-col gap-6">
      <PeriodNav
        kind={kind}
        periodsAgo={periodsAgo}
        from={from}
        to={to}
        onKind={(value) => {
          setKind(value);
          setPeriodsAgo(0);
        }}
        onPeriodsAgo={setPeriodsAgo}
      />
      <div className="flex flex-wrap items-center gap-3">
        {kind === "week" && (
          <Button
            variant="primary"
            disabled={!review}
            onClick={() =>
              document.getElementById(REFLECTION_FIELD_ID)?.focus()
            }
          >
            {t("review.startWeek")}
          </Button>
        )}
        <Button disabled={!review} onClick={() => void exportMarkdown()}>
          <DownloadIcon /> {t("review.export")}
        </Button>
        <Link
          to="/stats"
          className="inline-flex min-h-11 items-center rounded-lg bg-surface-container-high px-4 font-semibold hover:bg-surface-bright"
        >
          {t("review.openStatsLink")}
        </Link>
      </div>

      {loaded && "error" in loaded && (
        <p
          role="alert"
          className="rounded-xl bg-surface-container-low p-4 text-red-300"
        >
          {loaded.error}
        </p>
      )}
      {!loaded && (
        <p className="text-on-surface-variant">{t("review.loading")}</p>
      )}

      {review && (
        <>
          <CollapsibleSection title={t("review.section.work")} defaultOpen>
            <TaskList
              tasks={review.tasks}
              withTime={false}
              empty={t("review.none.work")}
            />
          </CollapsibleSection>
          <CollapsibleSection
            title={t("review.section.activities")}
            defaultOpen
          >
            <ActivityList
              activities={review.activities}
              withTime={false}
              empty={t("review.none.activities")}
            />
          </CollapsibleSection>
          <ResultsCard
            review={review}
            today={toIsoDate(today)}
            onChanged={reloadReview}
          />
          <DayNotesCard
            review={review}
            today={toIsoDate(today)}
            onChanged={reloadReview}
          />
          <ReflectionCard
            review={review}
            isWeek={kind === "week"}
            weekHint={
              kind === "week" && periodsAgo === 0 && isWeekReviewTime(today)
            }
            onChanged={reloadReview}
          />
          <NextPeriodCard />
        </>
      )}
    </div>
  );
}
