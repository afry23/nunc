-- Eigene Projekte und Kontexte einer Session (ADR 0023), als JSON-Array von Tags mit Präfix (`+name`, `@name`).
-- Sie zählen nur, wenn die Session kein Todo hat. Additiv: Migrationen 001–005 und der CHECK in 001 bleiben unberührt.
ALTER TABLE session_meta ADD COLUMN project_tags TEXT NOT NULL DEFAULT '[]';
ALTER TABLE session_meta ADD COLUMN context_tags TEXT NOT NULL DEFAULT '[]';
