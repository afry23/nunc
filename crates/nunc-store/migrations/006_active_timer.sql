-- Persisted timer state (nunc only; the Go app ignores unknown tables).
-- Single row (id = 1): `state` is the JSON of nunc_core::timer::Timer, so the running or paused
-- session, the pomodoro cycle and the break suggestion survive an app restart (ADR 0004).
CREATE TABLE IF NOT EXISTS active_timer (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    state      TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
