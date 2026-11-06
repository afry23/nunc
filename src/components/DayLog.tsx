import { useEffect, useState } from "react";
import { useT } from "../i18n";
import {
  commands,
  errorMessage,
  unwrap,
  type DayTimeline,
  type TimelineBlock,
} from "../lib/api";
import { trackedMinutes } from "../lib/dayPlan";
import { formatMinutes, formatTimeOfDay } from "../lib/format";
import { useTodoStore } from "../stores/todoStore";
import { useUiStore } from "../stores/uiStore";
import { blockName } from "./Timeline";
import { SessionDialog, type EditedSession } from "./SessionDialog";
import { Button } from "./ui";

type Loaded = { timeline: DayTimeline } | { error: string };
type Editing = { session: EditedSession | null } | null;

/**
 * Die Einträge eines Tages als Liste (Beginn–Ende, Name, Zuordnung) mit Summe, Nachtragen und Bearbeiten. Datenquelle
 * ist dieselbe wie im Verlauf (`get_day_timeline`); der Tag wird nur durch `day` begrenzt.
 */
export function DayLog({ day }: { day: string }) {
  const t = useT();
  const todos = useTodoStore((s) => s.all);
  const notify = useUiStore((s) => s.notify);
  const [loaded, setLoaded] = useState<Loaded | null>(null);
  const [editing, setEditing] = useState<Editing>(null);
  const [reload, setReload] = useState(0);

  useEffect(() => {
    if (day === "") return;
    let cancelled = false;
    void (async () => {
      try {
        const timeline = await unwrap(commands.getDayTimeline(day));
        if (!cancelled) setLoaded({ timeline });
      } catch (error) {
        if (!cancelled) setLoaded({ error: errorMessage(error) });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [day, reload]);

  const blocks =
    loaded && "timeline" in loaded
      ? loaded.timeline.blocks.filter((block) => block.sessionType === "work")
      : [];
  const edit = (block: TimelineBlock): void =>
    setEditing({
      session: {
        id: block.sessionId,
        start: block.start,
        end: block.end,
        todoId: block.todoId,
        note: block.note,
        projectTags: block.projectTags,
        contextTags: block.contextTags,
      },
    });

  return (
    <div className="flex flex-col gap-3">
      {loaded && "error" in loaded && (
        <p
          role="alert"
          className="rounded-xl bg-surface-container-low p-4 text-red-300"
        >
          {loaded.error}
        </p>
      )}
      {loaded && "timeline" in loaded && (
        <>
          <p className="font-semibold tabular-nums" data-testid="day-tracked">
            {t("daylog.tracked", {
              time: formatMinutes(trackedMinutes(blocks)),
            })}
          </p>
          {blocks.length === 0 ? (
            <p className="text-on-surface-variant">{t("timeline.empty")}</p>
          ) : (
            <ul aria-label={t("daylog.list")} className="flex flex-col gap-2">
              {blocks.map((block) => {
                const tags = [...block.projectTags, ...block.contextTags];
                const range = `${formatTimeOfDay(block.start)}–${formatTimeOfDay(block.end)}`;
                return (
                  <li key={block.sessionId}>
                    <button
                      type="button"
                      onClick={() => edit(block)}
                      className="flex min-h-11 w-full flex-col items-start rounded-xl bg-surface-container-low px-3 py-2 text-left hover:bg-surface-bright"
                    >
                      <span className="tabular-nums text-on-surface-variant">
                        {range}
                      </span>
                      <span className="break-words">
                        {block.note || blockName(block, todos, t)}
                      </span>
                      {tags.length > 0 && (
                        <span className="text-sm text-on-surface-variant">
                          {tags.join(" · ")}
                        </span>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </>
      )}
      <div>
        <Button
          disabled={day === ""}
          onClick={() => setEditing({ session: null })}
        >
          {t("daylog.add")}
        </Button>
      </div>
      {editing && (
        <SessionDialog
          session={editing.session}
          date={day}
          todos={todos}
          onClose={() => setEditing(null)}
          onSaved={(overlaps) => {
            setEditing(null);
            setReload(reload + 1);
            if (overlaps > 0) notify(t("entry.overlap"));
          }}
        />
      )}
    </div>
  );
}

/** Der Tagesverlauf als Karte (Überschrift plus Liste), z. B. in „Heute“. */
export function DayLogCard({ day }: { day: string }) {
  const t = useT();
  return (
    <section
      aria-labelledby="daylog-heading"
      className="flex flex-col gap-3 rounded-2xl bg-surface-container-low p-5"
    >
      <h2 id="daylog-heading" className="font-display text-lg font-bold">
        {t("daylog.title")}
      </h2>
      <DayLog day={day} />
    </section>
  );
}
