import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { HabitForm } from "../components/HabitForm";
import { HabitRow } from "../components/HabitRow";
import {
  ChevronLeftIcon,
  PencilIcon,
  PlusIcon,
  TrashIcon,
} from "../components/icons";
import { ActionButton, Button, IconButton } from "../components/ui";
import { useT } from "../i18n";
import type { HabitItem } from "../lib/api";
import {
  EMPTY_DRAFT,
  draftOf,
  goalOf,
  scheduleOf,
  splitHabits,
} from "../lib/habits";
import { useHabitStore } from "../stores/habitStore";

/** „Gewohnheiten“: anlegen, ändern, abhaken, archivieren und löschen (ADR 0028). */
export function HabitsView() {
  const t = useT();
  const items = useHabitStore((s) => s.items);
  /** `"new"`, die Kennung der Gewohnheit in Bearbeitung oder nichts. */
  const [editing, setEditing] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<HabitItem | null>(null);

  useEffect(() => {
    void useHabitStore.getState().load();
  }, []);

  const { active, archived } = splitHabits(items);
  const editedItem = items.find((item) => item.id === editing);

  const actions = (item: HabitItem) => (
    <>
      {!item.archived && (
        <IconButton
          label={t("habits.edit", { name: item.name })}
          onClick={() => setEditing(item.id)}
        >
          <PencilIcon />
        </IconButton>
      )}
      <Button
        variant="ghost"
        aria-label={t(item.archived ? "habits.restore" : "habits.archive", {
          name: item.name,
        })}
        onClick={() =>
          void useHabitStore.getState().archive(item.id, !item.archived)
        }
      >
        {t(item.archived ? "habits.restoreShort" : "habits.archiveShort")}
      </Button>
      <IconButton
        label={t("habits.delete", { name: item.name })}
        onClick={() => setDeleting(item)}
      >
        <TrashIcon />
      </IconButton>
    </>
  );

  return (
    <section
      aria-labelledby="habits-heading"
      className="mx-auto flex max-w-3xl flex-col gap-6 px-6 py-10"
    >
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 id="habits-heading" className="headline-md">
            {t("habits.title")}
          </h1>
          <p className="text-on-surface-variant">{t("habits.subtitle")}</p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Link
            to="/"
            className="inline-flex min-h-11 items-center gap-1 rounded-lg px-3 font-semibold text-on-surface-variant hover:bg-surface-container-high"
          >
            <ChevronLeftIcon />
            {t("habits.back")}
          </Link>
          <ActionButton icon={<PlusIcon />} onClick={() => setEditing("new")}>
            {t("habits.new")}
          </ActionButton>
        </div>
      </header>

      {editing === "new" && (
        <HabitForm
          initial={EMPTY_DRAFT}
          onSave={(draft) =>
            useHabitStore
              .getState()
              .save(null, draft.name, scheduleOf(draft), goalOf(draft))
          }
          onCancel={() => setEditing(null)}
        />
      )}
      {editedItem && (
        <HabitForm
          key={editedItem.id}
          initial={draftOf(editedItem)}
          onSave={(draft) =>
            useHabitStore
              .getState()
              .save(editedItem.id, draft.name, scheduleOf(draft), goalOf(draft))
          }
          onCancel={() => setEditing(null)}
        />
      )}

      {items.length === 0 ? (
        <p className="rounded-xl bg-surface-container-low p-6 text-center text-on-surface-variant">
          {t("habits.empty")}
        </p>
      ) : (
        <ul aria-label={t("habits.list")} className="flex flex-col gap-3">
          {active.map((item) => (
            <HabitRow key={item.id} item={item} actions={actions(item)} />
          ))}
        </ul>
      )}

      {archived.length > 0 && (
        <div className="flex flex-col gap-3">
          <h2 className="label-md text-on-surface-variant">
            {t("habits.archived")}
          </h2>
          <ul className="flex flex-col gap-3">
            {archived.map((item) => (
              <HabitRow
                key={item.id}
                item={item}
                actions={actions(item)}
                muted
              />
            ))}
          </ul>
        </div>
      )}

      {deleting && (
        <ConfirmDialog
          title={t("habits.delete.title")}
          confirmLabel={t("habits.delete.confirm")}
          danger
          onCancel={() => setDeleting(null)}
          onConfirm={() => {
            void useHabitStore.getState().remove(deleting.id);
            setDeleting(null);
          }}
        >
          <p>{t("habits.delete.body", { name: deleting.name })}</p>
        </ConfirmDialog>
      )}
    </section>
  );
}
