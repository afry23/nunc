import { Link } from "@tanstack/react-router";
import { useEffect } from "react";
import {
  ChevronRightIcon,
  ChevronLeftIcon,
  CloseIcon,
} from "../components/icons";
import { IconButton } from "../components/ui";
import { useT } from "../i18n";
import type { PlanReason } from "../lib/api";
import { fillPercent, nextDay, planRows } from "../lib/dayPlan";
import { formatMinutes, shortDate, weekdayLabel } from "../lib/format";
import { titleOf } from "../lib/todos";
import { usePlanStore } from "../stores/planStore";
import { useTodoStore } from "../stores/todoStore";

const REASON_CLASS: Record<PlanReason, string> = {
  overdue: "bg-error-container text-on-error-container",
  due: "bg-primary-container text-on-primary-container",
  planned: "bg-surface-container-high text-on-surface-variant",
};

/** „Heute“: überfällige, heute fällige und eingeplante Aufgaben gegen die Tageskapazität (ADR 0027). */
export function TodayView() {
  const t = useT();
  const plan = usePlanStore((s) => s.plan);
  const all = useTodoStore((s) => s.all);
  const { planTodo, setEstimate } = usePlanStore.getState();
  const { toggle } = useTodoStore.getState();

  // Die Planung steckt in der Beschreibung: ändert sich die Todo-Liste (auch durch Sync), lädt der Plan neu.
  useEffect(() => {
    void usePlanStore.getState().load();
  }, [all]);

  const rows = plan ? planRows(plan, all) : [];
  const over = plan !== null && plan.remainingMin < 0;
  const day = plan?.day ?? "";

  return (
    <section
      aria-labelledby="today-heading"
      className="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
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
        <Link
          to="/todos"
          className="inline-flex min-h-11 items-center gap-1 rounded-lg px-3 font-semibold text-on-surface-variant hover:bg-surface-container-high"
        >
          <ChevronLeftIcon />
          {t("today.back")}
        </Link>
      </header>

      {plan && (
        <div
          className="flex flex-col gap-2 rounded-xl bg-surface-container-low p-4"
          data-testid="capacity"
        >
          <div className="flex flex-wrap items-baseline justify-between gap-x-4">
            <p className="font-semibold">
              {t("today.estimated", {
                planned: formatMinutes(plan.estimateMin),
                capacity: formatMinutes(plan.capacityMin),
              })}
            </p>
            <p className={over ? "text-error" : "text-on-surface-variant"}>
              {over
                ? t("today.over", { time: formatMinutes(-plan.remainingMin) })
                : t("today.remaining", {
                    time: formatMinutes(plan.remainingMin),
                  })}
            </p>
          </div>
          <div
            role="progressbar"
            aria-label={t("today.capacity")}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={fillPercent(plan.estimateMin, plan.capacityMin)}
            className="h-2 overflow-hidden rounded-full bg-surface-container-highest"
          >
            <div
              className={`h-full ${over ? "bg-error" : "bg-primary"}`}
              style={{
                width: `${fillPercent(plan.estimateMin, plan.capacityMin)}%`,
              }}
            />
          </div>
          {plan.unestimated > 0 && (
            <p className="text-sm text-on-surface-variant">
              {t("today.unestimated", { count: plan.unestimated })}
            </p>
          )}
        </div>
      )}

      {plan && rows.length === 0 ? (
        <p className="rounded-xl bg-surface-container-low p-6 text-center text-on-surface-variant">
          {t("today.empty")}
        </p>
      ) : (
        <ul aria-label={t("today.list")} className="flex flex-col gap-3">
          {rows.map(({ item, todo }) => {
            const title = titleOf(todo);
            return (
              <li
                key={todo.id}
                data-testid="plan-item"
                data-reason={item.reason}
                className="flex flex-col gap-2 rounded-xl bg-surface-container-low p-3"
              >
                <div className="flex min-w-0 items-center gap-2">
                  <label className="inline-flex size-11 shrink-0 items-center justify-center">
                    <input
                      type="checkbox"
                      checked={false}
                      onChange={() => void toggle(todo.id)}
                      aria-label={t("today.done", { title })}
                      className="size-6 accent-[var(--color-primary)]"
                    />
                  </label>
                  <span className="min-w-0 flex-1 break-words">
                    {todo.priority && (
                      <span className="mr-2 font-semibold text-primary">
                        ({todo.priority})
                      </span>
                    )}
                    {title}
                  </span>
                  <span
                    className={`shrink-0 rounded-lg px-2 py-1 text-sm font-semibold ${REASON_CLASS[item.reason]}`}
                  >
                    {t(`today.reason.${item.reason}`)}
                  </span>
                </div>
                <div className="flex flex-wrap items-center gap-x-3 gap-y-1 pl-13">
                  <label className="flex items-center gap-2">
                    <input
                      type="number"
                      aria-label={t("today.estimate", { title })}
                      inputMode="numeric"
                      min={1}
                      max={10080}
                      defaultValue={item.estimateMin ?? ""}
                      key={`${todo.id}:${item.estimateMin ?? ""}`}
                      onBlur={(event) => {
                        const value = event.target.valueAsNumber;
                        const next = Number.isNaN(value) ? null : value;
                        if (next !== item.estimateMin)
                          void setEstimate(todo.id, next);
                      }}
                      className="h-11 w-20 rounded-sm bg-surface-container-highest px-3 tabular-nums focus:bg-secondary-container"
                    />
                    <span className="text-on-surface-variant">
                      {t("today.estimateUnit")}
                    </span>
                  </label>
                  {item.spentMin > 0 && (
                    <span className="text-sm text-on-surface-variant">
                      {t("today.spent", {
                        spent: formatMinutes(item.spentMin),
                      })}
                    </span>
                  )}
                  {item.reason === "planned" && (
                    <span className="ml-auto flex">
                      <IconButton
                        label={t("today.unplan", { title })}
                        onClick={() => void planTodo(todo.id, null)}
                      >
                        <CloseIcon />
                      </IconButton>
                      <IconButton
                        label={t("today.postpone", { title })}
                        onClick={() => void planTodo(todo.id, nextDay(day))}
                      >
                        <ChevronRightIcon />
                      </IconButton>
                    </span>
                  )}
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
