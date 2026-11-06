-- Tagesfokus (ADR 0032): `journal_entries.kind` darf zusätzlich 'day_focus' sein. SQLite kann ein CHECK nicht ändern,
-- die Tabelle wird neu gebaut. Das Kind `journal_field_clock` hängt per ON DELETE CASCADE an ihr: Ein `DROP TABLE`
-- der Eltern würde die Stempel löschen, deshalb werden sie vorher gesichert und danach zurückgeschrieben.
CREATE TABLE journal_field_clock_backup AS SELECT * FROM journal_field_clock;
DROP TABLE journal_field_clock;

CREATE TABLE journal_entries_new (
    sync_id TEXT    PRIMARY KEY,
    kind    TEXT    NOT NULL CHECK (kind IN ('day_note', 'week_note', 'achievement', 'day_focus')),
    date    TEXT    NOT NULL,
    text    TEXT    NOT NULL DEFAULT '',
    deleted INTEGER NOT NULL DEFAULT 0
);
INSERT INTO journal_entries_new (sync_id, kind, date, text, deleted)
    SELECT sync_id, kind, date, text, deleted FROM journal_entries;
DROP TABLE journal_entries;
ALTER TABLE journal_entries_new RENAME TO journal_entries;
CREATE INDEX IF NOT EXISTS idx_journal_entries_date ON journal_entries(date);

CREATE TABLE journal_field_clock (
    sync_id     TEXT    NOT NULL REFERENCES journal_entries(sync_id) ON DELETE CASCADE,
    field       TEXT    NOT NULL,
    hlc_ms      INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    hlc_device  TEXT    NOT NULL,
    PRIMARY KEY (sync_id, field)
);
INSERT INTO journal_field_clock SELECT * FROM journal_field_clock_backup;
DROP TABLE journal_field_clock_backup;
