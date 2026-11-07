import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { ChevronLeftIcon } from "../components/icons";
import { DescriptionText, PriorityMark } from "../components/TodoText";
import { Button } from "../components/ui";
import { useT } from "../i18n";
import { pickableTodos } from "../lib/dayPlan";
import { dateWithYear, toIsoDate } from "../lib/format";
import { dueOf, titleOf } from "../lib/todos";
import { usePlanStore } from "../stores/planStore";
import { useTodoStore } from "../stores/todoStore";

/**
 * „Aufgaben auswählen“: der Aufgabenbestand als Auswahl für den Tag. Die Aufgaben bleiben in ihren Projekten und
 * werden nur für heute eingeplant (ADR 0027).
 */
export function PickView() {
  const t = useT();
  const navigate = useNavigate();
  const all = useTodoStore((s) => s.all);
  const plan = usePlanStore((s) => s.plan);
  const [query, setQuery] = useState("");
  const [chosen, setChosen] = useState<ReadonlySet<number>>(new Set());
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void usePlanStore.getState().load();
  }, []);

  const today = toIsoDate(new Date());
  const offered = pickableTodos(all, plan, query);

  const toggle = (id: number): void =>
    setChosen((current) => {
      const next = new Set(current);
      if (!next.delete(id)) next.add(id);
      return next;
    });

  const confirm = async (): Promise<void> => {
    if (chosen.size === 0) return;
    setBusy(true);
    for (const id of chosen) await usePlanStore.getState().planTodo(id, today);
    await usePlanStore.getState().load();
    setBusy(false);
    void navigate({ to: "/" });
  };

  return (
    <section
      aria-labelledby="pick-heading"
      className="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <h1 id="pick-heading" className="headline-md">
          {t("pick.title")}
        </h1>
        <Button variant="ghost" onClick={() => void navigate({ to: "/" })}>
          <ChevronLeftIcon />
          {t("pick.back")}
        </Button>
      </header>

      <label className="flex flex-col gap-1">
        <span className="sr-only">{t("pick.search")}</span>
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t("pick.searchPlaceholder")}
          className="min-h-11 w-full rounded-sm bg-surface-container-highest px-3 focus:bg-secondary-container"
        />
      </label>

      {offered.length === 0 ? (
        <p className="rounded-xl bg-surface-container-low p-6 text-center text-on-surface-variant">
          {t("pick.empty")}
        </p>
      ) : (
        <ul aria-label={t("pick.list")} className="flex flex-col gap-2">
          {offered.map((todo) => {
            const title = titleOf(todo);
            const picked = chosen.has(todo.id);
            const due = dueOf(todo.description);
            return (
              <li key={todo.id}>
                <button
                  type="button"
                  aria-pressed={picked}
                  aria-label={t("pick.choose", { title })}
                  onClick={() => toggle(todo.id)}
                  className={`flex min-h-11 w-full cursor-pointer flex-col gap-1 rounded-xl p-3 text-left break-words ${picked ? "bg-secondary-container text-on-secondary-container" : "bg-surface-container-low hover:bg-surface-bright"}`}
                >
                  <span className="min-w-0">
                    <PriorityMark priority={todo.priority} />{" "}
                    <DescriptionText description={todo.description} />
                  </span>
                  {due && (
                    <span className="text-sm text-on-surface-variant">
                      {t("todos.item.due", { date: dateWithYear(due, today) })}
                    </span>
                  )}
                </button>
              </li>
            );
          })}
        </ul>
      )}

      <div className="safe-bottom sticky bottom-0 flex justify-end bg-surface py-2">
        <Button
          variant="primary"
          disabled={chosen.size === 0 || busy}
          onClick={() => void confirm()}
        >
          {t("pick.confirm", { count: chosen.size })}
        </Button>
      </div>
    </section>
  );
}
