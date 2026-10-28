-- Subtasks (ADR 0022): `sync_id` des Elternknotens je Todo. Eigene Tabelle statt `ALTER TABLE todos`, ohne
-- Fremdschlüssel auf den Elternknoten (ein Kind kann im Sync vor ihm ankommen). Fehlt die Zeile, ist das Todo eine
-- Wurzel. Bestandsdaten haben keinen Elternknoten und brauchen keinen Stempel (fehlender Stempel gilt als uralt).
CREATE TABLE IF NOT EXISTS todo_parent (
    todo_id        INTEGER PRIMARY KEY REFERENCES todos(id) ON DELETE CASCADE,
    parent_sync_id TEXT NOT NULL
);
