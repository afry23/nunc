-- Settings as key-value store
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Flowtime break rules (configurable thresholds)
CREATE TABLE IF NOT EXISTS flowtime_rules (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    min_work_min INTEGER NOT NULL, -- inclusive lower bound (0 = start)
    max_work_min INTEGER,          -- exclusive upper bound (NULL = unbounded)
    break_min    INTEGER NOT NULL
);

-- Todos (parsed from todo.txt format)
CREATE TABLE IF NOT EXISTS todos (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    raw_text       TEXT    NOT NULL,
    priority       TEXT,                    -- "A"-"Z" or NULL
    completed      INTEGER NOT NULL DEFAULT 0,
    completed_at   DATE,                    -- YYYY-MM-DD
    created_at     DATE,                    -- YYYY-MM-DD
    description    TEXT    NOT NULL,
    project_tags   TEXT    NOT NULL DEFAULT '[]', -- JSON array
    context_tags   TEXT    NOT NULL DEFAULT '[]', -- JSON array
    pomodoro_count INTEGER NOT NULL DEFAULT 0,
    position       INTEGER NOT NULL
);

-- Work and break sessions
CREATE TABLE IF NOT EXISTS sessions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    mode         TEXT    NOT NULL CHECK (mode IN ('pomodoro', 'flowtime')),
    session_type TEXT    NOT NULL CHECK (session_type IN ('work', 'short_break', 'long_break')),
    status       TEXT    NOT NULL CHECK (status IN ('completed', 'aborted', 'skipped')),
    started_at   DATETIME NOT NULL,
    ended_at     DATETIME,
    planned_min  INTEGER,          -- target duration (Pomodoro only)
    actual_min   INTEGER,          -- actual elapsed minutes
    todo_id      INTEGER REFERENCES todos(id) ON DELETE SET NULL,
    break_min    INTEGER           -- recommended / used break duration
);

-- Seed default settings
INSERT OR IGNORE INTO settings (key, value) VALUES
    ('pomodoro_min',            '25'),
    ('short_break_min',         '5'),
    ('long_break_min',          '25'),
    ('long_break_after',        '4'),
    ('auto_start_breaks',       'false'),
    ('auto_start_next_pomodoro','false'),
    ('notifications_enabled',   'true'),
    ('sound_enabled',           'true'),
    ('sound_volume',            '80'),
    ('sound_end_pomodoro',      'bell'),
    ('sound_end_break',         'ding'),
    ('active_mode',             'pomodoro'),
    ('system_tray_enabled',     'true');

-- Seed default flowtime rules
INSERT OR IGNORE INTO flowtime_rules (id, min_work_min, max_work_min, break_min) VALUES
    (1, 0,  25,   5),
    (2, 25, 50,   8),
    (3, 50, 90,  10),
    (4, 90, NULL,15);
