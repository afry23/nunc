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
- **Todo** = Map Feld → LWW-Register `(Wert, HLC)`: `priority`, `completed`, `completed_at`, `created_at`,
  `description`, `position` (Fractional-Index als String), `deleted` (Tombstone). **Zeile (`raw_text`) und Tags sind
  abgeleitet**, keine Register: Sie werden nach jedem Merge aus den Feldern neu aufgebaut, sonst könnten sie von
  ihnen abweichen (A hakt ab, B ändert den Text). Ein Todo gilt nur als gelöscht, wenn das Löschen jünger ist als
  jede Bearbeitung; wer ein gleichzeitig gelöschtes Todo bearbeitet, behält es.
- **`pomodoro_count`** = G-Counter pro Gerät, Wert = Summe.
- **Sessions** bleiben ein G-Set (`(device_id, started_at)`).
- Der Merge ist kommutativ, assoziativ und idempotent. Gleichstand entscheidet `device_id`, nicht „lokal gewinnt“.
- **Format:** `nunc-sync.v2.json`, eine Datei, Upload wie bisher mit ETag/If-Match. `nunc-sync.json` wird nicht mehr
  geschrieben. Existiert beim ersten Lauf nur die alte Datei, wird sie einmalig als LWW-Write importiert (HLC aus
  `updated_at`) und danach ignoriert.
- **Store:** additive Migration `007_todo_crdt.sql` (`todo_field_clock`, `todo_pomodoro_counter`,
  `todo_position_key`, `device_clock`); die Tabellen von 001–005 bleiben unverändert. Die Spalten von `todos` sind
  der aufgelöste Stand, Statistik liest weiter `todos`/`sessions`. Jede lokale Mutation läuft wie bisher als SQL; danach
  vergleicht `crdt_repo::stamp_changes` den Stand vor und nach der Änderung und stempelt **nur die geänderten Felder**
  (in einem Sicherungspunkt). Bestandsdaten bekommen Stempel aus `updated_at` mit dem Pseudo-Gerät `legacy`, damit
  zwei Geräte mit demselben Stand identische Register haben. `todos.position` ist der abgeleitete, eindeutige Rang
  nach `todo_position_key`; beim Umordnen behält die längste schon passende Folge ihre Schlüssel, nur verschobene
  Todos werden gestempelt.
- **Einspielen:** `local_state` liest den Stand als `SyncState`, `apply_state` schreibt den zusammengeführten Stand
  in einer Transaktion zurück (neue Todos anlegen, gelöschte nicht, `raw_text`/Tags neu aufbauen, Sessions anfügen,
  `device_clock` auf den größten gesehenen Stempel setzen).
- **Tombstone-GC:** Ein gelöschtes Todo, dessen Löschen älter als 90 Tage ist (`TOMBSTONE_TTL_MS`), verschwindet nach dem
  Zusammenführen aus der Sync-Datei (`SyncState::collect_garbage`, im Lauf nach `merge_remote`). Die Zeilen in der
  Datenbank bleiben als weich gelöschte Todos, weil Statistik und Export sie für die Sessions brauchen. **Risiko:** Ein
  Gerät, das länger als 90 Tage offline war und das Todo noch lebend hat, legt es beim nächsten Sync wieder an. Ein
  Wasserstand aller Geräte wäre genauer, bräuchte aber eine Geräteliste in der Datei; bei einer privaten Handvoll
  Geräte steht der Aufwand nicht dafür.
- Drive nutzt dieselbe Merge-Funktion. Das Todoist-Mapping liest nur aufgelöste Werte.

## Verworfene Alternativen

- Automerge/Yrs/Loro: siehe oben.
- Op-Log (CmRDT): Kompaktion, Zustellgarantie.
- Zwei Dateien mit Go-Projektion: nur nötig für Go-Kompatibilität, die entfällt.

## Folgen

- Breaking im Mischbetrieb: ältere nunc-Versionen sehen die v2-Datei nicht. Hinweis in den Release-Notes; die Alt-Datei
  bleibt unangetastet liegen.
- Dateigröße wächst durch Register; Tombstones räumt die GC nach 90 Tagen ab.
- Tests: `@domain @property` (proptest: Kommutativität, Assoziativität, Idempotenz), `@integration @sync` für
  Zwei-Geräte-Läufe und Alt-Import. Die Go-Interop-Skripte (`tests/interop/`), die Sync-Szenarien von `compat_go.feature` und der LWW-Merge (`sync_merge`)
  sind entfernt; `compat_go.feature` heißt `existing_database.feature` und prüft nur noch Bestandsdatenbanken.
- `conflictedTodos` zählt künftig Todos, bei denen Felder beider Seiten zusammengeführt wurden.
