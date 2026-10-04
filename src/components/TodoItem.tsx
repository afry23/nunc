import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useState, type FormEvent } from "react";
import { useT } from "../i18n";
import type { Todo } from "../lib/api";
import { useSwipeToComplete } from "../hooks/useSwipeToComplete";
import { priorityTone, splitSegments, titleOf } from "../lib/todos";
import { useTodoStore } from "../stores/todoStore";
import {
  CheckIcon,
  ChevronDownIcon,
  ChevronUpIcon,
  GripIcon,
  PencilIcon,
  TrashIcon,
} from "./icons";
import { IconButton } from "./ui";

const TONE_CLASS = {
  high: "text-primary",
  medium: "text-orange-300",
  low: "text-on-surface-variant",
  none: "",
} as const;

/** Eine Zeile der Aufgabenliste: abhaken, bearbeiten (todo.txt-Syntax), löschen, umsortieren. */
export function TodoItem({
  todo,
  canReorder,
  isFirst,
  isLast,
}: {
  todo: Todo;
  canReorder: boolean;
  isFirst: boolean;
  isLast: boolean;
}) {
  const t = useT();
  const { toggle, remove, update, move } = useTodoStore.getState();
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(todo.rawText);
  const {
    setNodeRef,
    setActivatorNodeRef,
    attributes,
    listeners,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: todo.id, disabled: !canReorder });
  const { offset, handlers } = useSwipeToComplete(() => void toggle(todo.id));
  const title = titleOf(todo);
  const tone = priorityTone(todo.priority);

  const style = {
    transform: CSS.Transform.toString(transform),
    transition: transition,
    opacity: isDragging ? 0.6 : 1,
  };

  const startEditing = (): void => {
    setDraft(todo.rawText);
    setEditing(true);
  };

  const submit = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    if (draft.trim() === todo.rawText.trim()) {
      setEditing(false);
      return;
    }
    if (await update(todo.id, draft)) setEditing(false);
  };

  return (
    <li
      ref={setNodeRef}
      style={style}
      data-testid="todo-item"
      data-raw={todo.rawText}
      data-completed={todo.completed}
      className="relative overflow-hidden rounded-xl bg-surface-container-low"
    >
      <span
        aria-hidden="true"
        className={`absolute inset-y-0 left-0 flex w-24 items-center bg-primary pl-4 text-on-primary ${offset > 0 ? "" : "invisible"}`}
      >
        <CheckIcon />
      </span>
      <div
        {...handlers}
        data-testid="todo-row"
        style={{
          transform: offset > 0 ? `translateX(${offset}px)` : undefined,
        }}
        className="relative flex touch-pan-y items-center gap-2 bg-surface-container-low py-2 pr-2 pl-3"
      >
        {canReorder && (
          <button
            type="button"
            ref={setActivatorNodeRef}
            aria-label={t("todos.item.move", { title })}
            className="inline-flex size-11 shrink-0 cursor-grab touch-none items-center justify-center rounded-lg text-on-surface-variant hover:bg-surface-bright"
            {...attributes}
            {...listeners}
          >
            <GripIcon />
          </button>
        )}

        <label className="inline-flex size-11 shrink-0 items-center justify-center">
          <input
            type="checkbox"
            checked={todo.completed}
            onChange={() => void toggle(todo.id)}
            aria-label={t("todos.item.done", { title })}
            className="size-6 accent-[var(--color-primary)]"
          />
        </label>

        {editing ? (
          <form
            onSubmit={(event) => void submit(event)}
            className="flex flex-1 items-center gap-2"
          >
            <input
              autoFocus
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Escape") {
                  event.stopPropagation();
                  setEditing(false);
                }
              }}
              aria-label={t("todos.item.editLabel", { title })}
              className="min-h-11 flex-1 rounded-sm bg-surface-container-highest px-3 focus:bg-secondary-container"
            />
          </form>
        ) : (
          <div
            className={`flex min-w-0 flex-1 flex-wrap items-baseline gap-x-2 ${todo.completed ? "text-on-surface-variant line-through" : ""}`}
            onDoubleClick={startEditing}
          >
            {todo.priority && (
              <span
                className={`font-semibold ${TONE_CLASS[tone]}`}
                aria-label={t("todos.item.priority", {
                  priority: todo.priority,
                })}
              >
                ({todo.priority})
              </span>
            )}
            <span className="break-words">
              {splitSegments(todo.description).map((segment, index) => (
                <span
                  key={index}
                  className={
                    segment.kind === "project"
                      ? "text-primary"
                      : segment.kind === "context"
                        ? "text-secondary-fixed-dim"
                        : undefined
                  }
                >
                  {segment.text}
                </span>
              ))}
            </span>
          </div>
        )}

        {todo.pomodoroCount > 0 && (
          <span
            className="shrink-0 rounded-lg bg-surface-container-high px-2 py-1 text-sm tabular-nums"
            aria-label={t("todos.item.pomodoros", {
              count: todo.pomodoroCount,
            })}
          >
            🍅 {todo.pomodoroCount}
          </span>
        )}

        <div className="flex shrink-0">
          {canReorder && (
            <>
              <IconButton
                label={t("todos.item.up", { title })}
                disabled={isFirst}
                onClick={() => void move(todo.id, -1)}
              >
                <ChevronUpIcon />
              </IconButton>
              <IconButton
                label={t("todos.item.down", { title })}
                disabled={isLast}
                onClick={() => void move(todo.id, 1)}
              >
                <ChevronDownIcon />
              </IconButton>
            </>
          )}
          <IconButton
            label={t("todos.item.edit", { title })}
            onClick={startEditing}
          >
            <PencilIcon />
          </IconButton>
          <IconButton
            label={t("todos.item.delete", { title })}
            onClick={() => void remove(todo.id)}
          >
            <TrashIcon />
          </IconButton>
        </div>
      </div>
    </li>
  );
}
