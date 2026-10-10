-- Nachgetragene Einträge zählen als Stoppuhr (ADR 0018): `kind` folgt der Herkunft. Additiv, nur Daten.
UPDATE session_meta SET kind = 'stopwatch' WHERE origin = 'manual';
