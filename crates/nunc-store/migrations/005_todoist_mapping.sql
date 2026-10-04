CREATE TABLE IF NOT EXISTS todoist_mappings (
    nunc_todo_id    INTEGER NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
    todoist_task_id TEXT    NOT NULL,
    last_synced_at  DATETIME NOT NULL,
    PRIMARY KEY (nunc_todo_id)
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_todoist_mappings_task_id ON todoist_mappings(todoist_task_id);
