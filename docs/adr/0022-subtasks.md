# ADR 0022 – Subtasks (Todo-Baum)

- Status: vorgeschlagen
- Datum: 2026-10-28
- Ergänzt: ADR 0016 (CRDT-Sync), ADR 0003 (todo.txt)

## Kontext

Todos sind flach. Gewünscht sind Unteraufgaben, die im UI als Baum erscheinen. todo.txt kennt keine Hierarchie.

## Entscheidung

- **Neues Feld `parent`** am Todo: `sync_id` des Elternknotens oder leer. Es steht nicht in `raw_text`. Die lokale `id`
  taugt nicht, sie unterscheidet sich je Gerät. Es gibt keinen Fremdschlüssel: Ein Kind kann vor dem Elternknoten ankommen.
- **Sicht der Domäne und des UI:** `Todo.parent_id` (lokale `id`, `null` für Wurzeln). Der Store löst `parent` beim Lesen
  per `sync_id` in die lokale `id` auf; ein nicht auflösbarer Verweis liefert `null`, bleibt aber gespeichert und wird
  nach dem Sync wieder gültig.
- **Beliebige Tiefe.** Fehlt der Elternknoten (gelöscht, noch nicht synchronisiert) oder bilden Verweise einen Zyklus
  (gleichzeitige Umhängung auf zwei Geräten), gilt das Todo als Wurzel. Der Baumaufbau liegt rein in `nunc-core`.
- **Reihenfolge:** der globale Positionsschlüssel bleibt. Geschwister sortieren nach diesem Schlüssel.
- **Sync:** `parent` ist ein LWW-Register in `TodoCrdt` (`#[serde(default)]`). `schema_version` wird 5, gelesen werden
  2 bis 5. Ältere Clients lehnen v5 ab (ADR 0017).
- **Persistenz:** additive Migration `011_todo_parent.sql`.
- **Kaskade:** Abhaken und Löschen eines Elternknotens trifft alle Nachfahren. Wieder öffnen öffnet Kinder nicht.
- **Filter:** Ein Treffer zeigt seine Vorfahren als Kontext .
- **Pomodoros:** der Zähler bleibt je Todo; die Summe über den Teilbaum ist nur Anzeige.
- **Export/Import todo.txt, lokale API (text/plain):** Subtasks werden weggelassen, Import erzeugt nur Wurzeln.
- **Todoist:** Subtasks werden nicht synchronisiert (kein Mapping, kein Anlegen als flache Aufgabe).

## Verworfene Alternativen

- **Lokale `id` als Verweis:** nach dem Sync auf anderen Geräten ungültig.
- **Schlüssel je Elternknoten:** ändert Neunummerierung und Sync-Positionslogik ohne Gewinn.
- **`parent:`-Tag in der todo.txt-Zeile:** vermischt Format und Struktur, verfälscht die Beschreibung.

## Folgen

- Ältere nunc-Versionen können nach dem ersten v5-Sync nicht mehr mit der Sync-Datei arbeiten.
- Subtasks existieren nur in nunc (und dessen Sync), nicht in todo.txt oder Todoist.
