-- Habit-Tracker (ADR 0028): Gewohnheiten und Haken als CRDT. Additiv. Die Spalten halten die rohen Registerwerte;
-- ob eine Gewohnheit gilt, ergibt sich erst aus den Stempeln (`habit_field_clock`, bzw. `hlc_*` beim Haken).
CREATE TABLE IF NOT EXISTS habits (
    sync_id  TEXT    PRIMARY KEY,
    created  TEXT    NOT NULL,
    name     TEXT    NOT NULL,
    schedule TEXT    NOT NULL,
    goal     TEXT    NOT NULL,
    archived INTEGER NOT NULL DEFAULT 0,
    deleted  INTEGER NOT NULL DEFAULT 0
);

-- Stempel (Hybrid Logical Clock) je Feld. Fehlt eine Zeile, gilt das Feld als uralt (Stempel 0, Gerät 'legacy').
CREATE TABLE IF NOT EXISTS habit_field_clock (
    sync_id     TEXT    NOT NULL REFERENCES habits(sync_id) ON DELETE CASCADE,
    field       TEXT    NOT NULL,
    hlc_ms      INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    hlc_device  TEXT    NOT NULL,
    PRIMARY KEY (sync_id, field)
);

-- Ein Haken je (Gewohnheit, Tag); `sync_id` ist `log:<habit>:<YYYY-MM-DD>`. `done = 0` ist der Tombstone.
-- Kein Fremdschlüssel: Haken einer eingesammelten Gewohnheit räumt der Sync-Lauf selbst ab.
CREATE TABLE IF NOT EXISTS habit_logs (
    sync_id     TEXT    PRIMARY KEY,
    habit       TEXT    NOT NULL,
    day         TEXT    NOT NULL,
    done        INTEGER NOT NULL DEFAULT 0,
    hlc_ms      INTEGER NOT NULL DEFAULT 0,
    hlc_counter INTEGER NOT NULL DEFAULT 0,
    hlc_device  TEXT    NOT NULL DEFAULT 'legacy'
);
CREATE INDEX IF NOT EXISTS idx_habit_logs_day ON habit_logs(day);
CREATE INDEX IF NOT EXISTS idx_habit_logs_habit ON habit_logs(habit);
