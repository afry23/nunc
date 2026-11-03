# ADR 0027 – Tagesplaner („Heute“)

- Status: vorgeschlagen
- Datum: 2026-11-02
- Ergänzt: ADR 0016 (CRDT-Sync), ADR 0022 (Subtasks), ADR 0025 (Rückblick)

## Kontext

nunc kennt Todos und misst Zeit, plant aber keinen Tag: Es gibt keine Frist, keine Schätzung und keine Ansicht, die
zeigt, was heute ansteht und ob es in den Tag passt. Der Rückblick (ADR 0025/0026) beantwortet „Was habe ich getan?“,
der Tagesplaner „Was nehme ich mir vor?“. Soll und Ist (Schätzung gegen gemessene Zeit) sind mit den vorhandenen
Sessions ohne neue Erfassung möglich.

## Entscheidung

**Planung steht als todo.txt-Erweiterung in der Beschreibung**, nicht in neuen Spalten:

| Feld | Bedeutung |
|---|---|
| `due:YYYY-MM-DD` | Frist (todo.txt-üblich) |
| `plan:YYYY-MM-DD` | für diesen Tag eingeplant |
| `est:N` | Schätzung in Minuten, 1 bis 10080 |

- **Keine Migration, kein neues Sync-Format.** Die Beschreibung ist bereits ein LWW-Register des Todo-CRDT
  (ADR 0016) und Teil der exportierten todo.txt; die Felder reisen damit durch Sync, Backup, Import und Export, und
  andere todo.txt-Programme sehen `due:` ebenfalls. Der Preis: Planung und Text sind ein gemeinsames Register; wer auf
  zwei Geräten gleichzeitig Text und Frist ändert, bekommt die jüngere Beschreibung als Ganzes (wie heute bei
  gleichzeitigen Textänderungen).
- **Lesen und Schreiben sind pur** (`nunc_core::day_plan`): `plan_fields` liest, `with_field` setzt, ersetzt oder
  entfernt ein Feld und lässt den übrigen Text wortgleich. Ungültige Werte (`due:morgen`, `est:0`, `due:2026-3-4`)
  zählen nicht und bleiben gewöhnlicher Text; mehrere Vorkommen: das erste gültige gilt, `with_field` ersetzt es und
  entfernt die übrigen.
- **Ansicht „Heute“** (`day_plan(todos, sessions, today, capacity_min)`, pur, `today` bringt der Aufrufer): offene
  Todos, die **überfällig** (`due` vor heute), **heute fällig** oder **eingeplant** sind (`plan` heute oder früher;
  liegen gebliebene Pläne rutschen automatisch auf heute, ohne dass etwas umgeschrieben wird). Reihenfolge: überfällig,
  fällig, eingeplant; darin Priorität (ohne zuletzt), dann Position. Erledigte Todos stehen nie darauf.
- **Soll/Ist:** Summe der Schätzungen gegen die **Tageskapazität**; `remaining_min` wird negativ, wenn der Plan zu voll
  ist; Einträge ohne Schätzung werden gezählt, nicht mit 0 mitgerechnet. Je Eintrag die bisher erfasste Arbeitszeit
  (`spent_min`: Arbeitsintervalle abgeschlossen oder abgebrochen, nicht übersprungen).
- **Tageskapazität** ist eine Einstellung (`day_capacity_min`, Standard 240 = vier Stunden Fokuszeit), keine
  abgeleitete Größe.
- **Tagesabschluss:** Die noch **eingeplanten** Einträge von heute werden auf morgen verschoben (`plan:` morgen),
  vom Plan genommen (`plan:` entfernt) oder bleiben (rutschen von selbst auf den nächsten Tag); erledigt wird wie
  sonst per Haken. Überfällige und heute fällige Einträge hängen an ihrer Frist (`due:`) und bleiben unberührt.
  `closing_changes(plan, action)` liefert pur die Änderungen. Dazu gehört die **Tagesnotiz** (ADR 0026): Sie lässt sich
  in „Heute“ jederzeit schreiben und steht automatisch im Rückblick.

## Umsetzung in Stufen

1. `nunc-core`: Felder und Tagesplan (diese Stufe, mit `features/todos/day_plan.feature`).
2. Einstellung `day_capacity_min` (1–1440, nur nunc-Schlüssel in `settings`), Commands `get_day_plan()` (Plan für den
   lokalen heutigen Tag), `plan_todo(id, day)` (Plan-Tag setzen oder mit `null` entfernen) und
   `set_todo_estimate(id, minutes)`. Jeder Command ändert genau ein Feld über die pure `apply_plan` und lässt Frist und
   übrige Felder stehen; die Zeile geht über `update_todo`. Eine Schätzung außerhalb 1–10080 ist `invalid_input`. Die
   Frist ändert man im Text der Aufgabe (`due:`). Die Oberfläche verbindet die Einträge des Plans über `todoId` mit der Todo-Liste.
3. UI: Ansicht „Heute“ (`/today`, erreichbar über „Heute“ in der Aufgabenliste), Kapazität in den Einstellungen, Schnellaktion
   „Für heute einplanen“ an jeder offenen Aufgabe. Die Ansicht zeigt Schätzung, erfasste Zeit und Grund je Eintrag;
   „Auf morgen verschieben“ und „Vom Plan nehmen“ gibt es nur für eingeplante Einträge (überfällige und heute fällige
   ändert man über `due:`). Frist und Plan-Tag trägt man im Text der Aufgabe ein (`due:`, `plan:`); Titel blenden die
   Planungsfelder aus.
4. Tagesabschluss: Core `CloseAction`/`closing_changes`; Commands `close_day(action, note)` (alles oder nichts in einem
   Sicherungspunkt: Einträge verschieben/abnehmen und Tagesnotiz speichern; liefert die Zahl geänderter Einträge) und
   `get_day_note(date)`; UI: Notiz und Dialog „Tag abschließen“ in „Heute“ (eigener PR).

## Verworfene Alternativen

- **Eigene Spalten/Tabelle für Plan und Schätzung:** Migration und neues Sync-Format, obwohl die Beschreibung schon
  ein synchronisiertes Register ist; die todo.txt-Datei verlöre die Information.
- **Schätzung in Pomodoros:** hängt an der eingestellten Pomodoro-Länge und ist im Flowtime-/Stoppuhr-Modus sinnlos;
  Minuten gelten für alle Modi.
- **Fristen als einziges Kriterium:** Die meisten Aufgaben haben keine Frist; `plan:` trennt „wann ich es tun will“
  von „wann es fällig ist“.

## Folgen

- Die Tag-Listen (`+projekt`, `@kontext`) bleiben unberührt; `key:value`-Wörter sind weder Projekt noch Kontext.
- Das Filtern nach Freitext findet `due:2026-03-04` (die Beschreibung enthält sie).
