import { Link, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { DotsIcon, PlayIcon } from "../components/icons";
import { ActionMenu } from "../components/ActionMenu";
import { CloseDayDialog } from "../components/CloseDayDialog";
import { CollapsibleSection } from "../components/CollapsibleSection";
import { DayLog } from "../components/DayLog";
import { DayNote } from "../components/DayNote";
import { StartButton } from "../components/StartButton";
import { DayTabs } from "../components/DayTabs";
import { FreeActivityDialog } from "../components/FreeActivityDialog";
import { blockName } from "../components/Timeline";
import { ActionButton } from "../components/ui";
import { useIsWide } from "../hooks/useMediaQuery";
import { useT } from "../i18n";
import type { PlanReason } from "../lib/api";
import { fillPercent, nextDay, planRows, type PlanRow } from "../lib/dayPlan";
import {
  formatMinutes,
  formatTimeOfDay,
  shortDate,
  weekdayLabel,
} from "../lib/format";
import { titleOf } from "../lib/todos";
import { usePlanStore } from "../stores/planStore";
import { useTodoStore } from "../stores/todoStore";

const REASON_CLASS: Record<PlanReason, string> = {
  overdue: "font-semibold text-error",
  due: "font-semibold text-on-surface",
  planned: "",
};

/** „Heute“: Übersicht des Tages (Plan gegen Kapazität, erfasste Arbeit, freie Tätigkeiten); der Timer liegt im Reiter „Timer“ (ADR 0027). */
export function TodayView() {
  const t = useT();
  const plan = usePlanStore((s) => s.plan);
  const tracked = usePlanStore((s) => s.tracked);
  const blocks = usePlanStore((s) => s.blocks);
  const note = usePlanStore((s) => s.note);
  const all = useTodoStore((s) => s.all);
  const { planTodo, setEstimate } = usePlanStore.getState();
  const { toggle } = useTodoStore.getState();
  const [closing, setClosing] = useState(false);
  const [freeActivity, setFreeActivity] = useState(false);
  const wide = useIsWide();
  const navigate = useNavigate();
  const day = plan?.day ?? "";

  // Die Planung steckt in der Beschreibung: ändert sich die Todo-Liste (auch durch Sync), lädt der Plan neu.
  useEffect(() => {
    void usePlanStore.getState().load();
  }, [all]);

  // die Notiz gehört zum Tag des Plans; das Feld folgt der gespeicherten Fassung
  useEffect(() => {
    if (day) void usePlanStore.getState().loadNote(day);
  }, [day]);

  const rows = plan ? planRows(plan, all) : [];
  const over = plan !== null && plan.remainingMin < 0;
  const free = blocks.filter((block) => block.todoId === null);

  const summary = plan && (
    <div className="flex flex-col gap-3" data-testid="capacity">
      <p
        className="font-display text-4xl font-bold tabular-nums"
        data-testid="tracked"
      >
        {t("today.tracked", { time: formatMinutes(tracked) })}
      </p>
      <div
        role="progressbar"
        aria-label={t("today.capacity")}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={fillPercent(plan.estimateMin, plan.capacityMin)}
        className="h-1.5 overflow-hidden rounded-full bg-surface-container-high"
      >
        <div
          className={`h-full rounded-full ${over ? "bg-error" : "bg-primary"}`}
          style={{
            width: `${fillPercent(plan.estimateMin, plan.capacityMin)}%`,
          }}
        />
      </div>
      <div className="flex flex-wrap items-baseline gap-x-5 gap-y-1 text-sm tabular-nums">
        <p className="font-semibold">
          {t("today.estimated", {
            planned: formatMinutes(plan.estimateMin),
            capacity: formatMinutes(plan.capacityMin),
          })}
        </p>
        <p className={over ? "text-error" : "text-on-surface-variant"}>
          {over
            ? t("today.over", { time: formatMinutes(-plan.remainingMin) })
            : t("today.remaining", { time: formatMinutes(plan.remainingMin) })}
        </p>
        {plan.unestimated > 0 && (
          <p className="text-on-surface-variant">
            {t("today.unestimated", { count: plan.unestimated })}
          </p>
        )}
      </div>
    </div>
  );

  const planned = (
    <section aria-labelledby="planned-heading" className="flex flex-col gap-2">
      <h2 id="planned-heading" className="font-display text-lg font-bold">
        {t("today.planned")}
      </h2>
      {plan && rows.length === 0 ? (
        <div className="flex flex-col items-start gap-4 rounded-2xl bg-surface-container-low p-6">
          <p className="font-display text-2xl font-bold leading-tight">
            {t("today.empty")}
          </p>
          <div className="flex flex-wrap gap-3">
            <Link
              to="/pick"
              className="inline-flex min-h-12 items-center rounded-xl bg-primary px-6 font-semibold text-on-primary hover:brightness-110"
            >
              {t("today.pick")}
            </Link>
            <Link
              to="/tasks"
              className="inline-flex min-h-12 items-center rounded-xl bg-surface-container-high px-5 font-semibold hover:bg-surface-bright"
            >
              {t("today.addTask")}
            </Link>
          </div>
          <p className="text-on-surface-variant">{t("today.emptyHint")}</p>
        </div>
      ) : (
        <>
          <ul aria-label={t("today.list")} className="flex flex-col">
            {rows.map((row) => (
              <PlanListRow
                key={row.todo.id}
                row={row}
                onToggle={() => void toggle(row.todo.id)}
                onEstimate={(value) => void setEstimate(row.todo.id, value)}
                onUnplan={() => void planTodo(row.todo.id, null)}
                onPostpone={() => void planTodo(row.todo.id, nextDay(day))}
              />
            ))}
          </ul>
          <div>
            <Link
              to="/pick"
              className="inline-flex min-h-11 items-center rounded-lg bg-surface-container-high px-4 font-semibold hover:bg-surface-bright"
            >
              {t("today.pick")}
            </Link>
          </div>
        </>
      )}
    </section>
  );

  const activities = (
    <section
      aria-labelledby="activities-heading"
      className="flex flex-col gap-2"
    >
      <h2 id="activities-heading" className="font-display text-lg font-bold">
        {t("today.freeActivities")}
      </h2>
      {free.length > 0 && (
        <ul aria-label={t("today.freeList")} className="flex flex-col">
          {free.map((block) => {
            const minutes = Math.round(
              (Date.parse(block.end) - Date.parse(block.start)) / 60000,
            );
            const tags = [...block.projectTags, ...block.contextTags];
            return (
              <li
                key={block.sessionId}
                className="flex flex-wrap items-baseline gap-x-3 px-1 py-2"
              >
                <span className="tabular-nums text-on-surface-variant">
                  {formatTimeOfDay(block.start)}–{formatTimeOfDay(block.end)}
                </span>
                <span className="min-w-0 flex-1 break-words">
                  {block.note || blockName(block, all, t)}
                  {tags.length > 0 && (
                    <span className="ml-2 text-sm text-on-surface-variant">
                      {tags.join(" · ")}
                    </span>
                  )}
                </span>
                <span className="tabular-nums text-on-surface-variant">
                  {formatMinutes(minutes)}
                </span>
              </li>
            );
          })}
        </ul>
      )}
      <div>
        <ActionButton
          icon={<PlayIcon />}
          variant="secondary"
          onClick={() => setFreeActivity(true)}
        >
          {t("activity.start")}
        </ActionButton>
      </div>
      {free.length === 0 && (
        <p className="text-sm text-on-surface-variant">{t("activity.hint")}</p>
      )}
    </section>
  );

  const notes = plan && (
    // neuer Schlüssel bei jeder gespeicherten Fassung: das Feld beginnt wieder mit ihr
    <DayNote
      key={`${day}|${note}`}
      note={note}
      onSave={(text) => void usePlanStore.getState().saveNote(day, text)}
    />
  );

  const side = (
    <>
      <CollapsibleSection title={t("daylog.title")}>
        <DayLog day={day} />
      </CollapsibleSection>
      <CollapsibleSection title={t("today.part.notes")}>
        {notes}
      </CollapsibleSection>
    </>
  );

  return (
    <section
      aria-labelledby="today-heading"
      className={`mx-auto flex flex-col gap-8 px-6 py-10 ${wide ? "max-w-5xl" : "max-w-3xl"}`}
    >
      <header className="flex flex-col gap-4">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h1 id="today-heading" className="headline-md">
              {t("today.title")}
            </h1>
            <p className="text-on-surface-variant">
              {plan
                ? `${weekdayLabel(plan.day)}, ${shortDate(plan.day)}`
                : t("today.subtitle")}
            </p>
          </div>
          <ActionMenu
            label={t("today.menu")}
            icon={<DotsIcon />}
            actions={[
              {
                label: t("today.close"),
                disabled: !plan,
                onSelect: () => setClosing(true),
              },
            ]}
          />
        </div>
        <DayTabs current="today" />
      </header>

      {summary}

      {wide ? (
        <div className="grid items-start gap-8 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
          <div className="flex flex-col gap-8">
            {planned}
            {activities}
          </div>
          <div className="flex flex-col gap-6">{side}</div>
        </div>
      ) : (
        <>
          {planned}
          {activities}
          {side}
        </>
      )}

      {freeActivity && (
        <FreeActivityDialog
          onCancel={() => setFreeActivity(false)}
          onStarted={() => {
            setFreeActivity(false);
            void navigate({ to: "/focus" });
          }}
        />
      )}
      {plan && closing && (
        <CloseDayDialog
          plan={plan}
          note={note}
          onCancel={() => setClosing(false)}
          onClose={async (action, text) => {
            if (await usePlanStore.getState().closeDay(day, action, text))
              setClosing(false);
          }}
        />
      )}
    </section>
  );
}

/**
 * Eine Zeile des Tagesplans: Erledigen, Titel mit Grund und erfasster Zeit, Schätzung (nur für die Planung, nicht die
 * Timer-Dauer), Start (öffnet den Timer) und das Menü. Nur eingeplante Einträge lassen sich verschieben oder abwählen;
 * Fristen ändert man an der Aufgabe.
 */
function PlanListRow({
  row,
  onToggle,
  onEstimate,
  onUnplan,
  onPostpone,
}: {
  row: PlanRow;
  onToggle: () => void;
  onEstimate: (value: number | null) => void;
  onUnplan: () => void;
  onPostpone: () => void;
}) {
  const t = useT();
  const { item, todo } = row;
  const title = titleOf(todo);
  const [editing, setEditing] = useState(false);
  return (
    <li
      data-testid="plan-item"
      data-reason={item.reason}
      className="flex flex-wrap items-center gap-x-2 gap-y-1 rounded-xl px-1 py-1 hover:bg-surface-container-low"
    >
      <label className="inline-flex size-11 shrink-0 items-center justify-center">
        <input
          type="checkbox"
          checked={false}
          onChange={onToggle}
          aria-label={t("today.done", { title })}
          className="size-5 accent-[var(--color-primary)]"
        />
      </label>
      <div className="min-w-0 flex-1 basis-40">
        <p className="break-words">
          {todo.priority && (
            <span className="mr-2 font-semibold text-primary">
              ({todo.priority})
            </span>
          )}
          {title}
        </p>
        <p className="flex flex-wrap gap-x-3 text-sm text-on-surface-variant">
          <span className={REASON_CLASS[item.reason]}>
            {t(`today.reason.${item.reason}`)}
          </span>
          {item.spentMin > 0 && (
            <span>
              {t("today.spent", { spent: formatMinutes(item.spentMin) })}
            </span>
          )}
        </p>
      </div>
      {editing ? (
        <label className="flex items-center gap-1.5">
          <input
            type="number"
            autoFocus
            aria-label={t("today.estimate", { title })}
            inputMode="numeric"
            min={1}
            max={10080}
            defaultValue={item.estimateMin ?? ""}
            onBlur={(event) => {
              const value = event.target.valueAsNumber;
              const next = Number.isNaN(value) ? null : value;
              if (next !== item.estimateMin) onEstimate(next);
              setEditing(false);
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") event.currentTarget.blur();
              if (event.key === "Escape") setEditing(false);
            }}
            className="h-11 w-16 rounded-lg bg-surface-container-high px-2 text-center tabular-nums focus:bg-secondary-container"
          />
          <span className="text-sm text-on-surface-variant">
            {t("today.estimateUnit")}
          </span>
        </label>
      ) : (
        <button
          type="button"
          onClick={() => setEditing(true)}
          title={t("today.estimateHint")}
          aria-label={t("today.estimateEdit", { title })}
          className="inline-flex min-h-11 items-center rounded-lg px-2 text-sm text-on-surface-variant tabular-nums hover:bg-surface-bright hover:text-on-surface"
        >
          {item.estimateMin === null
            ? t("today.estimateAdd")
            : t("today.estimateValue", {
                time: formatMinutes(item.estimateMin),
              })}
        </button>
      )}
      <StartButton todoId={todo.id} title={title} />
      {item.reason === "planned" ? (
        <ActionMenu
          label={t("todos.item.actions", { title })}
          icon={<DotsIcon />}
          actions={[
            {
              label: t("today.postponeShort"),
              onSelect: onPostpone,
            },
            { label: t("today.unplanShort"), onSelect: onUnplan },
          ]}
        />
      ) : (
        <span aria-hidden className="size-11" />
      )}
    </li>
  );
}
