# ADR 0023 – Eigene Projekte und Kontexte an Sessions ohne Aufgabe

- Status: vorgeschlagen
- Datum: 2026-11-01
- Ergänzt: ADR 0017 (Sessions als CRDT), ADR 0018 (Stopwatch)

## Kontext

Die Stoppuhr läuft schon ohne Aufgabe (`todo_id` ist optional). Projekt und Kontext kamen aber nur aus den Tags des
Todos. Eine Session ohne Todo – etwa ein Meeting – fiel deshalb aus den Projekt-/Kontext-Auswertungen heraus und
landete im Timesheet in der Zeile „ohne Zuordnung“. nunc soll auch den Tagesablauf abbilden, nicht nur Aufgaben.

## Entscheidung

- **Eigene Tags an der Session:** `Session.project_tags` (`+name`) und `Session.context_tags` (`@name`), dazu die
  vorhandene Notiz. Die Tags liegen als JSON-Array in `session_meta.project_tags` / `context_tags`
  (Migration `012_session_tags.sql`, additiv, Default `[]`). Migration 001–005 bleiben unberührt.
- **Vorrang:** Hat die Session ein Todo, zählen dessen Projekte und Kontexte; eigene Tags werden ignoriert
  (`effective_tags` in `nunc-core`). Das verhindert doppeltes Zählen und macht die Regel an einer Stelle prüfbar.
  Beim Speichern über die Commands werden eigene Tags mit gesetztem Todo verworfen.
- **Normalisierung:** `normalize_tags` trimmt, setzt das Präfix, verwirft leere Tags und Tags mit Leerraum und
  entfernt Duplikate (Reihenfolge bleibt). Maßgeblich ist das Backend; das Frontend spiegelt die Regel nur für die Anzeige.
- **Timer:** `ActiveSession.labels` (`serde(default)`, ältere gespeicherte Timer laden weiter). Der Command
  `start_timer` nimmt optional `labels` (Projekte, Kontexte, Notiz); mit Todo bleiben sie leer. `FinishedSession` und
  `commit_finish` schreiben sie samt Stempeln in einer Transaktion. `TimerSnapshot.labels` zeigt sie in der Oberfläche.
- **Nachtragen/Bearbeiten:** `SessionInput`, `ManualSession` und `SessionEdit` tragen die Tags.
- **Sync:** zwei LWW-Register `project_tags` und `context_tags` (`Register<Vec<String>>`) in `SessionCrdt`, mit
  `serde(default)`. `schema_version` wird **6**; gelesen werden 2 bis 6. Ältere Clients lehnen v6 ab (strikter
  Versionscheck, ADR 0017). Die Tags werden als Ganzes überschrieben, nicht elementweise zusammengeführt.
- **Auswertung:** Statistik (Projekt-/Kontextkarten), Timesheet (Gruppierung), Export (Spalten `project_tags`,
  `context_tags`) und Timeline-Blöcke nutzen `effective_tags`. „Top-Aufgaben“ bleiben todo-bezogen.
- **Auswahl:** Der Command `get_known_tags` liefert alle Tags aus Todos und Sessions als Vorschläge; neue Namen
  können frei getippt werden. Es gibt weiterhin keine Stammdaten für Projekte.

## Verworfene Alternativen

- **Spalten in `sessions`:** bräuchte Änderungen an Migration 001 bzw. einen Tabellen-Neuaufbau.
- **Eigene Tags zusätzlich zu den Todo-Tags:** zählt eine Session doppelt und macht die Regel schwerer erklärbar.
- **Platzhalter-Todo je Meeting:** vermüllt die Aufgabenliste und die todo.txt.

## Folgen

- Downgrade: Ein Gerät mit nunc < 0.5 liest `nunc-sync.v2.json` im Format 6 nicht (Hinweis „Bitte nunc aktualisieren“).
- Backups älterer Versionen bleiben restorebar (additive Migration).
- Offen: Beim automatischen Start der nächsten Pomodoro-Runde ohne Todo gehen eigene Tags verloren; die
  Stoppuhr und manuelle Einträge sind der Hauptanwendungsfall.
