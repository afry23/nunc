-- Journal (ADR 0026): Tages-/Wochennotizen und Erfolge als CRDT. Additiv. Die Spalten halten die rohen Registerwerte
-- (`date`, `text`, `deleted`); ob ein Eintrag gilt, ergibt sich erst aus den Stempeln (`journal_field_clock`).
CREATE TABLE IF NOT EXISTS journal_entries (
    sync_id TEXT    PRIMARY KEY,
    kind    TEXT    NOT NULL CHECK (kind IN ('day_note', 'week_note', 'achievement')),
    date    TEXT    NOT NULL,
    text    TEXT    NOT NULL DEFAULT '',
    deleted INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_journal_entries_date ON journal_entries(date);

-- Stempel (Hybrid Logical Clock) je Feld. Fehlt eine Zeile, gilt das Feld als uralt (Stempel 0, Gerät 'legacy').
CREATE TABLE IF NOT EXISTS journal_field_clock (
    sync_id     TEXT    NOT NULL REFERENCES journal_entries(sync_id) ON DELETE CASCADE,
    field       TEXT    NOT NULL,
    hlc_ms      INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    hlc_device  TEXT    NOT NULL,
    PRIMARY KEY (sync_id, field)
);
