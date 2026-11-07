import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { useState, type FormEvent } from "react";
import { useT } from "../i18n";
import { useSwipeToComplete } from "../hooks/useSwipeToComplete";
import {
  priorityTone,
  splitSegments,
  subtaskProgress,
  titleOf,
  type TreeRow,
} from "../lib/todos";
import { formatMinutes, toIsoDate } from "../lib/format";
import { useTimerStore } from "../stores/timerStore";
import { usePlanStore } from "../stores/planStore";
import { useTodoStore } from "../stores/todoStore";
import {
  CheckIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  DotsIcon,
  GripIcon,
  PauseIcon,
  PlayIcon,
} from "./icons";
import { ActionMenu, type MenuAction } from "./ActionMenu";
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
  selected,
  onOpen,
}: {
  row: TreeRow;
  collapsed: boolean;
  canReorder: boolean;
  canIndent: boolean;
  canOutdent: boolean;
  isFirst: boolean;
  isLast: boolean;
  /** Die Details dieser Aufgabe sind geöffnet. */
  selected: boolean;
  onOpen: (id: number) => void;
}) {
  const t = useT();
  const { todo, depth, hasChildren, pomodoroTotal } = row;
  /** Läuft der Timer für genau diese Aufgabe (Arbeitsphase), ersetzt Pause/Fortsetzen den Start-Knopf. */
  const timerState = useTimerStore((s) =>
    s.snapshot.todoId === todo.id && s.snapshot.sessionType === "work"
      ? s.snapshot.state
      : "idle",
  );
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
  const progress = subtaskProgress(
    useTodoStore((s) => s.all),
    todo.id,
  );
  const planItem = usePlanStore((s) =>
    s.plan?.items.find((item) => item.todoId === todo.id),
  );
  const today = toIsoDate(new Date());
  const meta = [
    progress
      ? t("todos.item.progress", {
          done: progress.done,
          total: progress.total,
        })
      : null,
    planItem ? t("todos.item.today") : null,
    planItem && planItem.spentMin > 0
      ? t("today.spent", { spent: formatMinutes(planItem.spentMin) })
      : null,
  ].filter((part): part is string => part !== null);

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

  const menuActions: MenuAction[] = [
    {
      label: t("todos.menu.planToday"),
      hidden: todo.completed || planItem?.reason === "planned",
      onSelect: () => void usePlanStore.getState().planTodo(todo.id, today),
    },
    {
      label: t("todos.menu.unplan"),
      hidden: todo.completed || planItem?.reason !== "planned",
      onSelect: () => void usePlanStore.getState().planTodo(todo.id, null),
    },
    {
      label: t("todos.menu.achievement"),
      hidden: !todo.completed,
      onSelect: () =>
        void useTodoStore.getState().markAchievement(todo.id, today),
    },
    {
      label: t("todos.menu.addSub"),
      onSelect: () => setAddingSub(true),
    },
    {
      label: t("todos.menu.up"),
      hidden: !canReorder,
      disabled: isFirst,
      onSelect: () => void move(todo.id, -1),
    },
    {
      label: t("todos.menu.down"),
      hidden: !canReorder,
      disabled: isLast,
      onSelect: () => void move(todo.id, 1),
    },
    {
      label: t("todos.menu.indent"),
      hidden: !canReorder,
      disabled: !canIndent,
      onSelect: () => void indent(todo.id),
    },
    {
      label: t("todos.menu.outdent"),
      hidden: !canReorder,
      disabled: !canOutdent,
      onSelect: () => void outdent(todo.id),
    },
    {
      label: t("todos.menu.duplicate"),
      onSelect: () => void useTodoStore.getState().duplicate(todo.id),
    },
    {
      label: t("todos.menu.edit"),
      onSelect: startEditing,
    },
    {
      label: t("todos.menu.delete"),
      onSelect: () => void remove(todo.id),
    },
  ];

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
              <button
                type="button"
                aria-label={t("todos.item.details", { title })}
                aria-current={selected ? "true" : undefined}
                onClick={() => onOpen(todo.id)}
                className="min-h-11 min-w-0 cursor-pointer text-left break-words"
              >
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
              </button>
              {meta.length > 0 && (
                <span
                  data-testid="todo-meta"
                  className="basis-full text-sm text-on-surface-variant no-underline"
                >
                  {meta.join(" · ")}
                </span>
              )}
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
          {!todo.completed &&
            (timerState === "running" ? (
              <IconButton
                label={t("todos.item.pause", { title })}
                onClick={() => void useTimerStore.getState().pause()}
              >
                <PauseIcon />
              </IconButton>
            ) : timerState === "paused" ? (
              <IconButton
                label={t("todos.item.resume", { title })}
                onClick={() => void useTimerStore.getState().resume()}
              >
                <PlayIcon />
              </IconButton>
            ) : (
              <IconButton
                label={t("todos.item.start", { title })}
                onClick={() => void useTimerStore.getState().startTodo(todo.id)}
              >
                <PlayIcon />
              </IconButton>
            ))}
          <ActionMenu
            label={t("todos.item.actions", { title })}
            icon={<DotsIcon />}
            actions={menuActions}
          />
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
