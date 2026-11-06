import { CollapsibleSection } from "./CollapsibleSection";
import { useState, type FormEvent, type ReactNode } from "react";
import { useT } from "../i18n";
import { commands, unwrap, type JournalItem, type Review } from "../lib/api";
import { clampDay } from "../lib/review";
import { shortDate } from "../lib/format";
import { useUiStore } from "../stores/uiStore";
import { Button } from "./ui";

/** Längster Text eines Eintrags (`MAX_TEXT_CHARS` in `nunc-core`). */
const MAX_TEXT = 4000;

async function run(action: () => Promise<unknown>, done: () => void) {
  try {
    await action();
    done();
  } catch (error) {
    useUiStore.getState().fail(error);
  }
}

function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <CollapsibleSection title={title} defaultOpen>
      <div className="flex flex-col gap-3">{children}</div>
    </CollapsibleSection>
  );
}

/** Kennung des Reflexionsfelds, damit „Wochenrückblick beginnen“ dorthin springen kann. */
export const REFLECTION_FIELD_ID = "reflection-note";

interface EntryFormProps {
  id?: string;
  /** Beschriftung des Textfelds (zugänglicher Name). */
  label: string;
  submitLabel: string;
  initialText: string;
  /** Tag zum Wählen; `null`: kein Datumsfeld (der Tag steht fest). */
  day: { value: string; from: string; to: string } | null;
  /** Nach dem Speichern leeren (Formular zum Hinzufügen). */
  clearOnSubmit?: boolean;
  onSubmit: (day: string, text: string) => Promise<void>;
  onCancel?: () => void;
}

/** Textfeld mit Speichern-Knopf und optionaler Tageswahl, für Notizen und Erfolge. */
function EntryForm({
  id,
  label,
  submitLabel,
  initialText,
  day,
  clearOnSubmit,
  onSubmit,
  onCancel,
}: EntryFormProps) {
  const t = useT();
  const [text, setText] = useState(initialText);
  const [date, setDate] = useState(day?.value ?? "");
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      await onSubmit(date, text);
      if (clearOnSubmit) setText("");
    } finally {
      setBusy(false);
    }
  };

  return (
    <form
      onSubmit={(event) => void submit(event)}
      className="flex min-w-0 flex-col gap-2"
    >
      {day && (
        <label className="flex flex-col gap-1">
          <span className="label-md text-on-surface-variant">
            {t("journal.date")}
          </span>
          <input
            type="date"
            value={date}
            min={day.from}
            max={day.to}
            required
            onChange={(event) => setDate(event.target.value)}
            className="min-h-11 w-full rounded-lg bg-surface-container-high px-3"
          />
        </label>
      )}
      <textarea
        id={id}
        aria-label={label}
        value={text}
        maxLength={MAX_TEXT}
        rows={3}
        onChange={(event) => setText(event.target.value)}
        className="w-full min-w-0 rounded-lg bg-surface-container-high p-3"
      />
      <div className="flex flex-wrap gap-2">
        <Button
          type="submit"
          disabled={
            busy ||
            (clearOnSubmit
              ? text.trim() === ""
              : text.trim() === initialText.trim())
          }
        >
          {submitLabel}
        </Button>
        {onCancel && (
          <Button variant="ghost" onClick={onCancel}>
            {t("common.cancel")}
          </Button>
        )}
      </div>
    </form>
  );
}

function ItemDate({ date }: { date: string }) {
  return (
    <span className="text-on-surface-variant tabular-nums">
      {shortDate(date)}
    </span>
  );
}

function AchievementRow({
  item,
  from,
  to,
  onChanged,
}: {
  item: JournalItem;
  from: string;
  to: string;
  onChanged: () => void;
}) {
  const t = useT();
  const [editing, setEditing] = useState(false);
  if (editing) {
    return (
      <li>
        <EntryForm
          label={t("journal.text")}
          submitLabel={t("common.save")}
          initialText={item.text}
          day={{ value: item.date, from, to }}
          onSubmit={(date, text) =>
            run(
              () => unwrap(commands.updateAchievement(item.id, date, text)),
              () => {
                setEditing(false);
                onChanged();
              },
            )
          }
          onCancel={() => setEditing(false)}
        />
      </li>
    );
  }
  return (
    <li className="flex flex-wrap items-start justify-between gap-2">
      <p className="min-w-0 flex-1 break-words whitespace-pre-wrap">
        <ItemDate date={item.date} /> {item.text}
      </p>
      <div className="flex gap-1">
        <Button variant="ghost" onClick={() => setEditing(true)}>
          {t("journal.edit")}
        </Button>
        <Button
          variant="danger"
          onClick={() =>
            void run(
              () => unwrap(commands.deleteAchievement(item.id)),
              onChanged,
            )
          }
        >
          {t("journal.delete")}
        </Button>
      </div>
    </li>
  );
}

interface JournalProps {
  review: Review;
  /** Heutiger Tag (`YYYY-MM-DD`) als Vorgabe beim Hinzufügen. */
  today: string;
  onChanged: () => void;
}

/** Ergebnisse und Beiträge des Zeitraums (Erfolge, ADR 0026), direkt bearbeitbar. */
export function ResultsCard({ review, today, onChanged }: JournalProps) {
  const t = useT();
  const { from, to } = review.period;
  const { achievements } = review.journal;
  const start = clampDay(today, from, to);
  return (
    <Card title={t("review.section.results")}>
      {achievements.length === 0 ? (
        <p className="text-on-surface-variant">{t("journal.noAchievements")}</p>
      ) : (
        <ul className="flex flex-col gap-2">
          {achievements.map((item) => (
            <AchievementRow
              key={`${item.id}|${item.date}|${item.text}`}
              item={item}
              from={from}
              to={to}
              onChanged={onChanged}
            />
          ))}
        </ul>
      )}
      <EntryForm
        label={t("journal.newAchievement")}
        submitLabel={t("journal.add")}
        initialText=""
        day={{ value: start, from, to }}
        clearOnSubmit
        onSubmit={(date, text) =>
          run(() => unwrap(commands.addAchievement(date, text)), onChanged)
        }
      />
    </Card>
  );
}

/** Notizen aus den Tagen des Zeitraums, direkt bearbeitbar. */
export function DayNotesCard({ review, today, onChanged }: JournalProps) {
  const t = useT();
  const { from, to } = review.period;
  const { dayNotes } = review.journal;
  const start = clampDay(today, from, to);
  return (
    <Card title={t("review.section.notes")}>
      {dayNotes.length === 0 ? (
        <p className="text-on-surface-variant">{t("journal.noDayNotes")}</p>
      ) : (
        <ul className="flex flex-col gap-3">
          {dayNotes.map((note) => (
            <li key={`${note.id}|${note.text}`} className="flex flex-col gap-1">
              <ItemDate date={note.date} />
              <EntryForm
                label={`${t("review.section.notes")} ${shortDate(note.date)}`}
                submitLabel={t("common.save")}
                initialText={note.text}
                day={null}
                onSubmit={(_, text) =>
                  run(
                    () =>
                      unwrap(
                        commands.saveJournalNote("day_note", note.date, text),
                      ),
                    onChanged,
                  )
                }
              />
            </li>
          ))}
        </ul>
      )}
      <EntryForm
        label={t("journal.newDayNote")}
        submitLabel={t("journal.add")}
        initialText=""
        day={{ value: start, from, to }}
        clearOnSubmit
        onSubmit={(date, text) =>
          run(
            () => unwrap(commands.saveJournalNote("day_note", date, text)),
            onChanged,
          )
        }
      />
    </Card>
  );
}

/**
 * Reflexion der Woche (Wochennotiz) mit den drei Leitfragen. Monats- und Quartalsrückblicke zeigen die Wochennotizen
 * ihrer Wochen nur zum Lesen. `weekHint` blendet ab Freitag der aktuellen Woche den Hinweis ein.
 */
export function ReflectionCard({
  review,
  isWeek,
  weekHint,
  onChanged,
}: Omit<JournalProps, "today"> & { isWeek: boolean; weekHint: boolean }) {
  const t = useT();
  const { to } = review.period;
  const { weekNotes } = review.journal;
  if (!isWeek) {
    return weekNotes.length > 0 ? (
      <Card title={t("journal.weekNotes")}>
        <ul className="flex flex-col gap-2">
          {weekNotes.map((note) => (
            <li key={note.id} className="break-words whitespace-pre-wrap">
              <ItemDate date={note.date} /> {note.text}
            </li>
          ))}
        </ul>
      </Card>
    ) : null;
  }
  return (
    <Card title={t("journal.weekNote")}>
      {weekHint && (
        <p className="text-on-surface-variant">{t("journal.weekHint")}</p>
      )}
      <ul className="list-disc pl-5 text-on-surface-variant">
        <li>{t("review.reflection.good")}</li>
        <li>{t("review.reflection.hard")}</li>
        <li>{t("review.reflection.take")}</li>
      </ul>
      <EntryForm
        key={weekNotes[0]?.text ?? ""}
        id={REFLECTION_FIELD_ID}
        label={t("journal.weekNote")}
        submitLabel={t("common.save")}
        initialText={weekNotes[0]?.text ?? ""}
        day={null}
        onSubmit={(_, text) =>
          run(
            () => unwrap(commands.saveJournalNote("week_note", to, text)),
            onChanged,
          )
        }
      />
    </Card>
  );
}
