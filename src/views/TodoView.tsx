import {
  DndContext,
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { useEffect, useState, type FormEvent } from "react";
import {
  CheckIcon,
  ChevronLeftIcon,
  CloseIcon,
  DotsIcon,
  PlusIcon,
} from "../components/icons";
import { ActionMenu } from "../components/ActionMenu";
import { FilterMenu } from "../components/FilterMenu";
import { TaskDetail } from "../components/TaskDetail";
import { TodoItem } from "../components/TodoItem";
import { ActionButton, Button, Chip } from "../components/ui";
import { useIsWide } from "../hooks/useMediaQuery";
import { useT } from "../i18n";
import type { TodoSort } from "../lib/api";
import {
  buildTree,
  collectTags,
  indentParent,
  moveItem,
  outdentParent,
} from "../lib/todos";
import { usePlanStore } from "../stores/planStore";
import { useTodoStore } from "../stores/todoStore";

/** Horizontale Zugstrecke, ab der ein Ziehen als Ein-/Ausrücken gilt. */
const INDENT_DRAG_PX = 40;

const SORTS: TodoSort[] = [
  "manual",
  "priority",
  "created_at",
  "pomodoro_count",
  "alphabetical",
];

export function TodoView() {
  const t = useT();
  const all = useTodoStore((s) => s.all);
  const visible = useTodoStore((s) => s.visible);
  const collapsed = useTodoStore((s) => s.collapsed);
  const filter = useTodoStore((s) => s.filter);
  const showCompleted = useTodoStore((s) => s.showCompleted);
  const sort = useTodoStore((s) => s.sort);
  const direction = useTodoStore((s) => s.direction);
  const {
    add,
    setText,
    toggleTag,
    clearFilters,
    setShowCompleted,
    setSort,
    reorderVisible,
    indent,
    outdent,
    importFile,
    exportFile,
  } = useTodoStore.getState();
  const [draft, setDraft] = useState("");
  const wide = useIsWide();
  const navigate = useNavigate();
  // geöffnete Aufgabe steht in der Adresse: Zurück führt zur Liste, deren Filter im Store bleiben
  const { task: openId } = useSearch({ strict: false }) as { task?: number };
  const openTask = (id: number): void =>
    void navigate({ to: "/tasks", search: { task: id } });
  const closeTask = (): void => void navigate({ to: "/tasks", search: {} });

  // „Heute“ und Zeiten der Zeilen kommen aus dem Tagesplan
  useEffect(() => {
    void usePlanStore.getState().load();
  }, []);

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 6 } }),
    useSensor(TouchSensor, {
      activationConstraint: { delay: 200, tolerance: 6 },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  const tags = collectTags(all);
  const priorities = [
    ...new Set(
      all.map((todo) => todo.priority).filter((p): p is string => p !== null),
    ),
  ].sort();
  const activeFilters = [
    ...filter.priorities.map((value) => ({
      kind: "priorities" as const,
      value,
      text: `(${value})`,
    })),
    ...filter.projects.map((value) => ({
      kind: "projects" as const,
      value,
      text: value,
    })),
    ...filter.contexts.map((value) => ({
      kind: "contexts" as const,
      value,
      text: value,
    })),
  ];
  const filtering =
    filter.projects.length + filter.contexts.length + filter.priorities.length >
      0 || filter.text !== "";
  const canReorder =
    sort === "manual" && direction === "ascending" && !filtering;
  const rows = buildTree(visible, collapsed);
  const completedCount = all.filter((todo) => todo.completed).length;

  const submit = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    if (draft.trim() === "") return;
    if (await add(draft)) setDraft("");
  };

  const onDragEnd = (event: DragEndEvent): void => {
    const { active, over, delta } = event;
    // Ziehen nach rechts/links ohne Platzwechsel rückt ein bzw. aus
    if (!over || active.id === over.id) {
      if (canReorder && Math.abs(delta.x) >= INDENT_DRAG_PX) {
        void (delta.x > 0
          ? indent(Number(active.id))
          : outdent(Number(active.id)));
      }
      return;
    }
    const ids = visible.map((todo) => todo.id);
    void reorderVisible(
      moveItem(
        ids,
        ids.indexOf(Number(active.id)),
        ids.indexOf(Number(over.id)),
      ),
    );
  };

  // schmal: die Details ersetzen die Liste (eigener Screen); breit: sie stehen daneben
  if (openId !== undefined && !wide) {
    return (
      <section className="mx-auto flex max-w-3xl flex-col gap-4 px-6 py-10">
        <div>
          <Button variant="ghost" onClick={closeTask}>
            <ChevronLeftIcon />
            {t("task.back")}
          </Button>
        </div>
        <TaskDetail id={openId} />
      </section>
    );
  }

  const list = (
    <section
      aria-labelledby="todos-heading"
      className="flex min-w-0 flex-1 flex-col gap-6"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 id="todos-heading" className="headline-md">
            {t("todos.title")}
          </h1>
          <p className="text-on-surface-variant">{t("todos.subtitle")}</p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <ActionMenu
            label={t("todos.more")}
            icon={<DotsIcon />}
            actions={[
              { label: t("todos.import"), onSelect: () => void importFile() },
              { label: t("todos.export"), onSelect: () => void exportFile() },
            ]}
          />
          <label htmlFor="todo-sort" className="sr-only">
            {t("todos.sort.label")}
          </label>
          <select
            id="todo-sort"
            value={sort}
            onChange={(event) => void setSort(event.target.value as TodoSort)}
            className="min-h-11 rounded-lg bg-surface-container-high px-3"
          >
            {SORTS.map((value) => (
              <option key={value} value={value}>
                {t(`todos.sort.${value}`)}
              </option>
            ))}
          </select>
          <Button
            className="max-sm:size-11 max-sm:px-0"
            aria-label={
              direction === "ascending"
                ? t("todos.sort.ascending")
                : t("todos.sort.descending")
            }
            onClick={() =>
              void setSort(
                sort,
                direction === "ascending" ? "descending" : "ascending",
              )
            }
          >
            {direction === "ascending" ? "↑" : "↓"}
          </Button>
        </div>
      </header>

      <form
        onSubmit={(event) => void submit(event)}
        className="flex items-center gap-2 rounded-xl bg-surface-container-highest p-2 focus-within:bg-secondary-container"
      >
        <PlusIcon />
        <label htmlFor="todo-input" className="sr-only">
          {t("todos.new")}
        </label>
        <input
          id="todo-input"
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          placeholder={t("todos.placeholder")}
          autoComplete="off"
          className="min-h-11 min-w-0 flex-1 bg-transparent px-2 placeholder:text-on-surface-variant focus:outline-none"
        />
        <ActionButton
          type="submit"
          icon={<PlusIcon />}
          variant="primary"
          className="shrink-0"
          disabled={draft.trim() === ""}
        >
          {t("todos.add")}
        </ActionButton>
      </form>

      <div
        className="flex flex-col gap-3"
        role="group"
        aria-label={t("todos.filter")}
      >
        <div className="flex flex-wrap items-center gap-2">
          <label htmlFor="todo-search" className="sr-only">
            {t("todos.search.label")}
          </label>
          <input
            id="todo-search"
            type="search"
            value={filter.text}
            onChange={(event) => void setText(event.target.value)}
            placeholder={t("todos.search.placeholder")}
            className="min-h-11 min-w-48 flex-1 rounded-sm bg-surface-container-highest px-3 focus:bg-secondary-container"
          />
          {filtering && (
            <ActionButton
              icon={<CloseIcon />}
              variant="ghost"
              onClick={() => void clearFilters()}
            >
              {t("todos.filter.reset")}
            </ActionButton>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <FilterMenu
            label={t("todos.filter.priorities")}
            options={priorities}
            selected={filter.priorities}
            onToggle={(p) => void toggleTag("priorities", p)}
          />
          <FilterMenu
            label={t("todos.filter.projects")}
            options={tags.projects}
            selected={filter.projects}
            onToggle={(tag) => void toggleTag("projects", tag)}
          />
          <FilterMenu
            label={t("todos.filter.contexts")}
            options={tags.contexts}
            selected={filter.contexts}
            onToggle={(tag) => void toggleTag("contexts", tag)}
          />
        </div>
        {activeFilters.length > 0 && (
          <ul
            aria-label={t("todos.filter")}
            className="flex flex-wrap items-center gap-2"
          >
            {activeFilters.map(({ kind, value, text }) => (
              <li key={`${kind}:${value}`}>
                <Chip
                  selected
                  label={t("todos.filter.remove", { tag: text })}
                  onClick={() => void toggleTag(kind, value)}
                >
                  {text} <CloseIcon size={14} />
                </Chip>
              </li>
            ))}
          </ul>
        )}
      </div>

      {visible.length === 0 ? (
        <p className="rounded-xl bg-surface-container-low p-6 text-center text-on-surface-variant">
          {filtering ? t("todos.emptyFiltered") : t("todos.empty")}
        </p>
      ) : (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={onDragEnd}
        >
          <SortableContext
            items={rows.map((row) => row.todo.id)}
            strategy={verticalListSortingStrategy}
          >
            <ul aria-label={t("todos.list")} className="flex flex-col gap-3">
              {rows.map((row, index) => {
                const siblings = rows.filter(
                  (other) =>
                    other.parentIndex === row.parentIndex &&
                    other.depth === row.depth,
                );
                return (
                  <TodoItem
                    key={row.todo.id}
                    row={row}
                    collapsed={collapsed.has(row.todo.id)}
                    canReorder={canReorder}
                    canIndent={indentParent(rows, index) !== null}
                    canOutdent={outdentParent(rows, index) !== undefined}
                    isFirst={siblings[0] === row}
                    isLast={siblings[siblings.length - 1] === row}
                    selected={row.todo.id === openId}
                    onOpen={openTask}
                  />
                );
              })}
            </ul>
          </SortableContext>
        </DndContext>
      )}

      {completedCount > 0 && (
        <ActionButton
          icon={<CheckIcon />}
          variant="ghost"
          aria-pressed={showCompleted}
          onClick={() => void setShowCompleted(!showCompleted)}
        >
          {showCompleted
            ? t("todos.completed.hide")
            : t("todos.completed.show", { count: completedCount })}
        </ActionButton>
      )}
    </section>
  );

  if (openId === undefined || !wide)
    return <div className="mx-auto flex max-w-3xl px-6 py-10">{list}</div>;
  return (
    <div className="mx-auto flex max-w-6xl items-start gap-6 px-6 py-10">
      {list}
      <aside
        aria-label={t("task.heading")}
        className="sticky top-6 w-96 shrink-0"
      >
        <div className="mb-2 flex justify-end">
          <Button variant="ghost" onClick={closeTask}>
            {t("task.close")}
          </Button>
        </div>
        <TaskDetail id={openId} />
      </aside>
    </div>
  );
}
