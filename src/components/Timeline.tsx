import { useT } from "../i18n";
import type { DayTimeline, TimelineBlock, Todo } from "../lib/api";
import { formatTimeOfDay } from "../lib/format";
import { blockColor, blockPlacement, timelineWindow } from "../lib/timesheet";

/** Pixel je Stunde; die Spalte wird dadurch höher als der Bildschirm und scrollt mit der Seite. */
const HOUR_PX = 56;

/** Name eines Blocks: die Aufgabe, bei Pausen die Art. */
export function blockName(
  block: TimelineBlock,
  todos: Todo[],
  t: ReturnType<typeof useT>,
): string {
  if (block.sessionType !== "work")
    return t(`timeline.type.${block.sessionType}`);
  const todo = todos.find((candidate) => candidate.id === block.todoId);
  return todo?.description ?? t("timeline.noTodo");
}

/**
 * Senkrechter Zeitstrahl eines Tages. Arbeitsblöcke sind antippbar (`onSelect`), Pausen nur zur Orientierung.
 * Überlappende Blöcke liegen nebeneinander in der halben Breite.
 */
export function Timeline({
  date,
  timeline,
  todos,
  onSelect,
}: {
  date: string;
  timeline: DayTimeline;
  todos: Todo[];
  onSelect: (block: TimelineBlock) => void;
}) {
  const t = useT();
  const window = timelineWindow(timeline.blocks, date);
  const hours = Array.from(
    { length: window.toHour - window.fromHour },
    (_, index) => window.fromHour + index,
  );

  return (
    <div
      className="relative"
      style={{ height: hours.length * HOUR_PX }}
      data-testid="timeline"
    >
      {hours.map((hour, index) => (
        <div
          key={hour}
          aria-hidden="true"
          className="absolute inset-x-0 text-xs text-on-surface-variant"
          style={{ top: index * HOUR_PX }}
        >
          {String(hour).padStart(2, "0")}:00
        </div>
      ))}
      <ul
        aria-label={t("timeline.list")}
        className="absolute inset-y-0 right-0 left-14"
      >
        {timeline.blocks.map((block) => {
          const { top, height } = blockPlacement(block, date, window);
          const name = blockName(block, todos, t);
          const range = `${formatTimeOfDay(block.start)}–${formatTimeOfDay(block.end)}`;
          const style = {
            top: `${top}%`,
            height: `${height}%`,
            width: block.overlaps ? "50%" : "100%",
          };
          const body = (
            <>
              <span className="font-semibold">{name}</span>
              {block.note !== "" && (
                <span className="ml-2 opacity-80">{block.note}</span>
              )}
              {block.todoId === null &&
                [...block.projectTags, ...block.contextTags].length > 0 && (
                  <span className="ml-2 text-xs opacity-80">
                    {[...block.projectTags, ...block.contextTags].join(" ")}
                  </span>
                )}
              {block.origin === "manual" && (
                <span className="ml-2 text-xs opacity-80">
                  ✎ {t("timeline.manual")}
                </span>
              )}
              {block.overlaps && (
                <span className="ml-2 text-xs opacity-80">
                  ⚠ {t("timeline.overlaps")}
                </span>
              )}
            </>
          );
          const classes = `${blockColor(block.sessionType)} absolute overflow-hidden rounded-lg px-3 py-1 text-left text-sm text-on-primary`;
          return (
            <li key={block.sessionId}>
              {block.sessionType === "work" ? (
                <button
                  type="button"
                  aria-label={`${range} ${name}`}
                  onClick={() => onSelect(block)}
                  className={`${classes} min-h-6 hover:brightness-110`}
                  style={style}
                >
                  {body}
                </button>
              ) : (
                <div
                  role="img"
                  aria-label={`${range} ${name}`}
                  className={`${classes} opacity-60`}
                  style={style}
                >
                  {body}
                </div>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
