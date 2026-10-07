# ADR 0016 – CRDT-Sync

- Status: vorgeschlagen
- Datum: 2026-10-07
- Ersetzt teilweise: ADR 0008 (Merge-Regeln und Dateiformat; Transport, ETag-Upload und Steuerung bleiben)

## Kontext

Der Sync führte Todos per Last-Write-Wins **pro Gesamtzeile** zusammen (`updated_at`), Sessions append-only.
Gleichzeitige Änderungen verschiedener Felder desselben Todos gingen verloren, `pomodoro_count` und `position`
kollidierten, und Uhren-Drift entschied den Sieger. Die Bindung an das Go-Format `nunc-sync.json` entfällt: Auf die
Go-App wird keine Rücksicht mehr genommen.

## Entscheidung

**Zustandsbasierter CRDT, selbst gebaut, pur in `nunc-core`.** Keine Library (Automerge/Yrs/Loro: schwere
Dependency, Mobile-Build, Overkill für flache Records; ein Op-Log bräuchte Kompaktion und passt schlecht zu Datei-Sync).

- **HLC** `(wall_ms, counter, device_id)`, total geordnet, Zeit über die injizierte `Clock`. Remote-Werte, die mehr
  als 24 h in der Zukunft liegen, werden gekappt.
- **Todo** = Map Feld → LWW-Register `(Wert, HLC)`: `raw_text`, `priority`, `completed`, `completed_at`,
  `description`, Tags, `position` (Fractional-Index als String), `deleted` (Tombstone; Wiederbeleben = neuer Write).
- **`pomodoro_count`** = G-Counter pro Gerät, Wert = Summe.
- **Sessions** bleiben ein G-Set (`(device_id, started_at)`).
- Der Merge ist kommutativ, assoziativ und idempotent. Gleichstand entscheidet `device_id`, nicht „lokal gewinnt“.
- **Format:** `nunc-sync.v2.json`, eine Datei, Upload wie bisher mit ETag/If-Match. `nunc-sync.json` wird nicht mehr
  geschrieben. Existiert beim ersten Lauf nur die alte Datei, wird sie einmalig als LWW-Write importiert (HLC aus
  `updated_at`) und danach ignoriert.
- **Store:** additive Migration `007_todo_crdt.sql` (`todo_field_clock`, `todo_pomodoro_counter`, `device_clock`).
  Bestehende Spalten bleiben, Statistik liest weiter `todos`/`sessions`. Jede lokale Mutation stempelt **nur die
  geänderten Felder**, in einer Transaktion. Bestandsdaten bekommen HLCs aus `updated_at`.
- **Tombstone-GC** nach 90 Tagen bzw. Wasserstand aller Geräte (spätere Phase).
- Drive nutzt dieselbe Merge-Funktion. Das Todoist-Mapping liest nur aufgelöste Werte.

## Verworfene Alternativen

- Automerge/Yrs/Loro: siehe oben.
- Op-Log (CmRDT): Kompaktion, Zustellgarantie.
- Zwei Dateien mit Go-Projektion: nur nötig für Go-Kompatibilität, die entfällt.

## Folgen

- Breaking im Mischbetrieb: ältere nunc-Versionen sehen die v2-Datei nicht. Hinweis in den Release-Notes; die Alt-Datei
  bleibt unangetastet liegen.
- Dateigröße wächst durch Register und Tombstones (GC).
- Tests: `@domain @property` (proptest: Kommutativität, Assoziativität, Idempotenz), `@integration @sync` für
  Zwei-Geräte-Läufe und Alt-Import. Go-Interop-Skripte und `compat_go.feature` entfallen.
- `conflictedTodos` zählt künftig Todos, bei denen Felder beider Seiten zusammengeführt wurden.
