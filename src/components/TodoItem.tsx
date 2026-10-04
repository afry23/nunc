import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useState, type FormEvent } from "react";
import type { Todo } from "../lib/api";
import { priorityTone, splitSegments, titleOf } from "../lib/todos";
import { useTodoStore } from "../stores/todoStore";
import {
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
      className="flex items-center gap-2 rounded-xl bg-surface-container-low py-2 pr-2 pl-3"
    >
      {canReorder && (
        <button
          type="button"
          ref={setActivatorNodeRef}
          aria-label={`Verschieben: ${title}`}
          className="inline-flex size-11 shrink-0 cursor-grab touch-none items-center justify-center rounded-lg text-on-surface-variant hover:bg-surface-bright"
          {...attributes}
          {...listeners}
        >
          <GripIcon />
        </button>
      )}

      <input
        type="checkbox"
        checked={todo.completed}
        onChange={() => void toggle(todo.id)}
        aria-label={`Erledigt: ${title}`}
        className="size-6 shrink-0 accent-[var(--color-primary)]"
      />

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
            aria-label={`Aufgabe bearbeiten: ${title}`}
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
              aria-label={`Priorität ${todo.priority}`}
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
          aria-label={`${todo.pomodoroCount} Pomodoros`}
        >
          🍅 {todo.pomodoroCount}
        </span>
      )}

      <div className="flex shrink-0">
        {canReorder && (
          <>
            <IconButton
              label={`Nach oben: ${title}`}
              disabled={isFirst}
              onClick={() => void move(todo.id, -1)}
            >
              <ChevronUpIcon />
            </IconButton>
            <IconButton
              label={`Nach unten: ${title}`}
              disabled={isLast}
              onClick={() => void move(todo.id, 1)}
            >
              <ChevronDownIcon />
            </IconButton>
          </>
        )}
        <IconButton label={`Bearbeiten: ${title}`} onClick={startEditing}>
          <PencilIcon />
        </IconButton>
        <IconButton
          label={`Löschen: ${title}`}
          onClick={() => void remove(todo.id)}
        >
          <TrashIcon />
        </IconButton>
      </div>
    </li>
  );
}
