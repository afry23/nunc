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
import { useState, type FormEvent } from "react";
import { PlusIcon } from "../components/icons";
import { TodoItem } from "../components/TodoItem";
import { Button, Chip, SectionLabel } from "../components/ui";
import type { TodoSort } from "../lib/api";
import { collectTags, moveItem } from "../lib/todos";
import { useTodoStore } from "../stores/todoStore";

const SORTS: { value: TodoSort; label: string }[] = [
  { value: "manual", label: "Manuell" },
  { value: "priority", label: "Priorität" },
  { value: "created_at", label: "Erstellt am" },
  { value: "pomodoro_count", label: "Pomodoros" },
  { value: "alphabetical", label: "Alphabetisch" },
];

export function TodoView() {
  const all = useTodoStore((s) => s.all);
  const visible = useTodoStore((s) => s.visible);
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
  } = useTodoStore.getState();
  const [draft, setDraft] = useState("");

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
      all.map((t) => t.priority).filter((p): p is string => p !== null),
    ),
  ].sort();
  const filtering =
    filter.projects.length + filter.contexts.length + filter.priorities.length >
      0 || filter.text !== "";
  const canReorder =
    sort === "manual" && direction === "ascending" && !filtering;
  const completedCount = all.filter((t) => t.completed).length;

  const submit = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    if (draft.trim() === "") return;
    if (await add(draft)) setDraft("");
  };

  const onDragEnd = (event: DragEndEvent): void => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;
    const ids = visible.map((t) => t.id);
    void reorderVisible(
      moveItem(
        ids,
        ids.indexOf(Number(active.id)),
        ids.indexOf(Number(over.id)),
      ),
    );
  };

  return (
    <section
      aria-labelledby="todos-heading"
      className="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 id="todos-heading" className="headline-md">
            Aufgaben
          </h1>
          <p className="text-on-surface-variant">
            Prioritätenliste in todo.txt-Syntax.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <label htmlFor="todo-sort" className="sr-only">
            Sortierung
          </label>
          <select
            id="todo-sort"
            value={sort}
            onChange={(event) => void setSort(event.target.value as TodoSort)}
            className="min-h-11 rounded-lg bg-surface-container-high px-3"
          >
            {SORTS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          <Button
            aria-label={
              direction === "ascending"
                ? "Aufsteigend sortiert"
                : "Absteigend sortiert"
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
          Neue Aufgabe
        </label>
        <input
          id="todo-input"
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          placeholder="(A) Dokumentation fertig +projekt @kontext"
          autoComplete="off"
          className="min-h-11 flex-1 bg-transparent px-2 placeholder:text-on-surface-variant focus:outline-none"
        />
        <Button type="submit" variant="primary" disabled={draft.trim() === ""}>
          Hinzufügen
        </Button>
      </form>

      <div className="flex flex-col gap-3" role="group" aria-label="Filter">
        <div className="flex flex-wrap items-center gap-2">
          <label htmlFor="todo-search" className="sr-only">
            Aufgaben durchsuchen
          </label>
          <input
            id="todo-search"
            type="search"
            value={filter.text}
            onChange={(event) => void setText(event.target.value)}
            placeholder="Suchen …"
            className="min-h-11 min-w-48 flex-1 rounded-sm bg-surface-container-highest px-3 focus:bg-secondary-container"
          />
          {filtering && (
            <Button variant="ghost" onClick={() => void clearFilters()}>
              Filter zurücksetzen
            </Button>
          )}
        </div>
        {(tags.projects.length > 0 ||
          tags.contexts.length > 0 ||
          priorities.length > 0) && (
          <div className="flex flex-wrap items-center gap-2">
            <SectionLabel>Filter</SectionLabel>
            {priorities.map((p) => (
              <Chip
                key={p}
                selected={filter.priorities.includes(p)}
                onClick={() => void toggleTag("priorities", p)}
              >
                ({p})
              </Chip>
            ))}
            {tags.projects.map((tag) => (
              <Chip
                key={tag}
                selected={filter.projects.includes(tag)}
                onClick={() => void toggleTag("projects", tag)}
              >
                {tag}
              </Chip>
            ))}
            {tags.contexts.map((tag) => (
              <Chip
                key={tag}
                selected={filter.contexts.includes(tag)}
                onClick={() => void toggleTag("contexts", tag)}
              >
                {tag}
              </Chip>
            ))}
          </div>
        )}
      </div>

      {visible.length === 0 ? (
        <p className="rounded-xl bg-surface-container-low p-6 text-center text-on-surface-variant">
          {filtering
            ? "Keine Aufgabe passt zu den Filtern."
            : "Noch keine Aufgaben. Lege oben die erste an."}
        </p>
      ) : (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragEnd={onDragEnd}
        >
          <SortableContext
            items={visible.map((t) => t.id)}
            strategy={verticalListSortingStrategy}
          >
            <ul aria-label="Aufgabenliste" className="flex flex-col gap-3">
              {visible.map((todo, index) => (
                <TodoItem
                  key={todo.id}
                  todo={todo}
                  canReorder={canReorder}
                  isFirst={index === 0}
                  isLast={index === visible.length - 1}
                />
              ))}
            </ul>
          </SortableContext>
        </DndContext>
      )}

      {completedCount > 0 && (
        <Button
          variant="ghost"
          aria-pressed={showCompleted}
          onClick={() => void setShowCompleted(!showCompleted)}
        >
          {showCompleted
            ? "Erledigte ausblenden"
            : `Erledigte anzeigen (${completedCount})`}
        </Button>
      )}
    </section>
  );
}
