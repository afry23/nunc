-- Habit-Tracker entfernt (ADR 0033, ersetzt ADR 0028). Die Tabellen aus Migration 015 entfallen samt Daten;
-- Migration 015 selbst bleibt unverändert (Bestandsdatenbanken legen sie noch an und räumen sie hier wieder ab).
DROP TABLE IF EXISTS habit_logs;
DROP TABLE IF EXISTS habit_field_clock;
DROP TABLE IF EXISTS habits;
