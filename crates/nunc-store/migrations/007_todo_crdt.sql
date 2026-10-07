-- CRDT-Sync (ADR 0016): Stempel (Hybrid Logical Clock) je Todo-Feld, Pomodoro-Zähler je Gerät, Reihenfolge als
-- Fractional-Index. Additiv; die Spalten von `todos` bleiben der aufgelöste Stand, Statistik und UI lesen sie weiter.

-- Reihenfolge als Textschlüssel (lexikografisch sortierbar). `todos.position` bleibt der daraus abgeleitete Rang.
-- Eigene Tabelle statt `ALTER TABLE todos`, damit die Tabellen der Go-App unverändert bleiben.
CREATE TABLE IF NOT EXISTS todo_position_key (
    todo_id INTEGER PRIMARY KEY REFERENCES todos(id) ON DELETE CASCADE,
    key     TEXT NOT NULL
);

-- Bestandsdaten: feste Breite, endet nie auf '0' (Voraussetzung für nunc_core::position::position_between).
INSERT INTO todo_position_key (todo_id, key)
SELECT id, printf('%06d5', ROW_NUMBER() OVER (ORDER BY position, id)) FROM todos;

-- Stempel des letzten Schreibvorgangs je Feld. Fehlt eine Zeile, gilt das Feld als uralt (Stempel 0).
CREATE TABLE IF NOT EXISTS todo_field_clock (
    todo_id     INTEGER NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
    field       TEXT    NOT NULL,
    hlc_ms      INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL,
    hlc_device  TEXT    NOT NULL,
    PRIMARY KEY (todo_id, field)
);

-- Pomodoros je Gerät (G-Counter); `todos.pomodoro_count` ist die Summe.
CREATE TABLE IF NOT EXISTS todo_pomodoro_counter (
    todo_id   INTEGER NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
    device_id TEXT    NOT NULL,
    count     INTEGER NOT NULL,
    PRIMARY KEY (todo_id, device_id)
);

-- Zuletzt vergebener oder gesehener Stempel dieses Geräts (eine Zeile), damit lokale Stempel immer weiter laufen.
CREATE TABLE IF NOT EXISTS device_clock (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    hlc_ms      INTEGER NOT NULL,
    hlc_counter INTEGER NOT NULL
);

-- Bestandsdaten: jedes Feld bekommt den Zeitpunkt `updated_at` und das Pseudo-Gerät 'legacy'. Zwei Geräte mit
-- demselben synchronisierten Stand erzeugen so identische Register und keinen Scheinkonflikt.
INSERT INTO todo_field_clock (todo_id, field, hlc_ms, hlc_counter, hlc_device)
SELECT t.id, f.field,
       COALESCE(CAST(strftime('%s', NULLIF(t.updated_at, '')) AS INTEGER), 0) * 1000, 0, 'legacy'
FROM todos t
CROSS JOIN (
    SELECT 'priority' AS field UNION ALL SELECT 'completed' UNION ALL SELECT 'completed_at'
    UNION ALL SELECT 'created_at' UNION ALL SELECT 'description' UNION ALL SELECT 'position'
    UNION ALL SELECT 'deleted'
) f;

INSERT INTO todo_pomodoro_counter (todo_id, device_id, count)
SELECT id, 'legacy', pomodoro_count FROM todos WHERE pomodoro_count > 0;
