import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useState, type FormEvent } from "react";
import { useT } from "../i18n";
import { useSwipeToComplete } from "../hooks/useSwipeToComplete";
import {
  priorityTone,
  splitSegments,
  titleOf,
  type TreeRow,
} from "../lib/todos";
import { useTodoStore } from "../stores/todoStore";
import {
  CheckIcon,
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronUpIcon,
  GripIcon,
  PencilIcon,
  PlusIcon,
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
  row,
  collapsed,
  canReorder,
  canIndent,
  canOutdent,
  isFirst,
  isLast,
}: {
  row: TreeRow;
  collapsed: boolean;
  canReorder: boolean;
  canIndent: boolean;
  canOutdent: boolean;
  isFirst: boolean;
  isLast: boolean;
}) {
  const t = useT();
  const { todo, depth, hasChildren, pomodoroTotal } = row;
  const {
    toggle,
    remove,
    update,
    move,
    addSubtask,
    toggleCollapsed,
    indent,
    outdent,
  } = useTodoStore.getState();
  const [editing, setEditing] = useState(false);
  const [addingSub, setAddingSub] = useState(false);
  const [subDraft, setSubDraft] = useState("");
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
    marginLeft: `${depth * 1.5}rem`,
  };

  const submitSub = async (event: FormEvent): Promise<void> => {
    event.preventDefault();
    if (subDraft.trim() === "") return;
    if (await addSubtask(todo.id, subDraft)) {
      setSubDraft("");
      setAddingSub(false);
    }
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
      data-depth={depth}
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
        className="relative flex touch-pan-y flex-col gap-1 bg-surface-container-low py-2 pr-2 pl-3 sm:flex-row sm:items-center sm:gap-2"
      >
        <div className="flex min-w-0 flex-1 items-center gap-2">
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

          {hasChildren && (
            <IconButton
              label={
                collapsed
                  ? t("todos.item.expand", { title })
                  : t("todos.item.collapse", { title })
              }
              onClick={() => toggleCollapsed(todo.id)}
            >
              {collapsed ? <ChevronRightIcon /> : <ChevronDownIcon />}
            </IconButton>
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

          {pomodoroTotal > 0 && (
            <span
              className="shrink-0 rounded-lg bg-surface-container-high px-2 py-1 text-sm tabular-nums"
              aria-label={t("todos.item.pomodoros", {
                count: pomodoroTotal,
              })}
            >
              🍅 {pomodoroTotal}
            </span>
          )}
        </div>

        <div
          className={`flex shrink-0 max-sm:justify-end ${editing ? "max-sm:hidden" : ""}`}
        >
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
          {canReorder && (
            <>
              <IconButton
                label={t("todos.item.indent", { title })}
                disabled={!canIndent}
                onClick={() => void indent(todo.id)}
              >
                <ChevronRightIcon />
              </IconButton>
              <IconButton
                label={t("todos.item.outdent", { title })}
                disabled={!canOutdent}
                onClick={() => void outdent(todo.id)}
              >
                <ChevronLeftIcon />
              </IconButton>
            </>
          )}
          <IconButton
            label={t("todos.item.addSub", { title })}
            onClick={() => setAddingSub((open) => !open)}
          >
            <PlusIcon />
          </IconButton>
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
      {addingSub && (
        <form
          onSubmit={(event) => void submitSub(event)}
          className="flex items-center gap-2 bg-surface-container-low px-3 pb-2"
        >
          <input
            autoFocus
            value={subDraft}
            onChange={(event) => setSubDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.stopPropagation();
                setAddingSub(false);
              }
            }}
            placeholder={t("todos.item.subPlaceholder")}
            aria-label={t("todos.item.subLabel", { title })}
            className="min-h-11 flex-1 rounded-sm bg-surface-container-highest px-3 focus:bg-secondary-container"
          />
        </form>
      )}
    </li>
  );
}
