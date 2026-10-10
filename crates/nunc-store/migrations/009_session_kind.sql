-- Art einer Session (ADR 0018): `regular` oder `stopwatch`. Eine Stopwatch-Session steht in `sessions.mode` als
-- `flowtime` (der CHECK aus Migration 001 bleibt unberührt); nur `kind` unterscheidet sie. Additiv.
ALTER TABLE session_meta ADD COLUMN kind TEXT NOT NULL DEFAULT 'regular' CHECK (kind IN ('regular', 'stopwatch'));
