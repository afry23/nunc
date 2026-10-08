-- Sessions als CRDT (ADR 0017): stabile Kennung, Herkunft, Notiz und Löschkennzeichen je Session sowie Stempel
-- (Hybrid Logical Clock) je Feld. Additiv; die Spalten von `sessions` bleiben der aufgelöste Stand, Statistik und
-- Export lesen sie weiter. Eine Zeile in `session_meta` entsteht für Bestandssessions beim ersten Sync-Zugriff
-- (Kennung aus Gerät und Beginn), für nachgetragene Einträge beim Anlegen.
CREATE TABLE IF NOT EXISTS session_meta (
    session_id INTEGER PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    sync_id    TEXT    NOT NULL UNIQUE,
    origin     TEXT    NOT NULL DEFAULT 'timer' CHECK (origin IN ('timer', 'manual')),
    note       TEXT    NOT NULL DEFAULT '',
    deleted    INTEGER NOT NULL DEFAULT 0
);

-- Stempel des letzten Schreibvorgangs je Feld. Fehlt eine Zeile, gilt das Feld als uralt (Stempel 0, Gerät 'legacy').
CREATE TABLE IF NOT EXISTS session_field_clock (
    session_id  INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    field       TEXT    NOT NULL,
    hlc_ms      INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    hlc_device  TEXT    NOT NULL,
    PRIMARY KEY (session_id, field)
);
