# ADR 0035 – Aufwandsschätzung und Soll/Ist-Vergleich

- Status: vorgeschlagen
- Datum: 2026-11-07
- Ergänzt: ADR 0027 (Tagesplaner), ADR 0025 (Rückblick)

## Kontext

ADR 0027 kennt nur `est:N` (Minuten) und hat Pomodoros als Einheit verworfen. Im Alltag planen Nutzer Aufgaben aber
oft in Pomodoros, und der Rückblick beantwortet nicht, ob die Schätzung gestimmt hat.

## Entscheidung

- **Zweites Planungswort `estp:N`** (Pomodoros, 1 bis 999) neben `est:N`, ebenfalls in der Beschreibung. Keine
  Migration, kein neues Sync-Format; der Preis ist derselbe wie bei `est:` (gemeinsames LWW-Register der Beschreibung).
- **Eine Einheit je Aufgabe.** Der Bearbeiten-Dialog setzt die Zeile aus Text und Zeitbedarf zusammen
  (`withEstimate`, ein `update_todo`-Aufruf); `set_todo_estimate` (Tagesplan) entfernt eine Pomodoro-Schätzung;
  `apply_plan` lehnt beide Angaben zugleich ab. Steht von Hand doch beides in der Zeile, gewinnen die Minuten.
- **Vergleich in der Einheit der Schätzung** (`nunc_core::estimate`): Minuten gegen die Fokuszeit, Pomodoros gegen
  die abgeschlossenen Pomodoro-Sessions der Aufgabe. Der tatsächliche Aufwand ist der der **ganzen Lebenszeit** der
  Aufgabe (abgeschlossene Arbeitssessions, wie `task_time`); abgebrochene Sessions zählen nicht. Die Planansicht
  (`spent_by_todo`) zählt weiterhin auch abgebrochene Intervalle; sie zeigt Arbeit am heutigen Tag, nicht die Güte
  der Schätzung.
- **Urteil:** Abweichung bis 20 % der Schätzung ist „im Rahmen“, darunter „schneller“, darüber „länger“.
- **Anzeige:** Rückblick (erledigte Aufgaben mit Soll/Ist, Summenzeile, Markdown-Export) und Aufgabendetails
  (Fortschritt gegen Schätzung). Ein Klick auf eine Aufgabe öffnet einen modalen Dialog (ersetzt das Detail-Panel und die Route `/tasks?task=` aus ADR 0031). Er zeigt oben Start-Knopf, Soll/Ist-Balken, Unteraufgaben und Zeiteinträge und darunter die Eingaben: todo.txt-Zeile ohne Tags und Schätzung, Projekte, Kontexte und Zeitbedarf.

## Verworfen

- Eigene Spalte/CRDT-Register für die Schätzung: Migration und `SCHEMA_VERSION`-Anhebung ohne Nutzen gegenüber `est:`.
- Pomodoros in Minuten umrechnen: Der Wert hinge an der Pomodoro-Länge der Einstellungen zum Speicherzeitpunkt.
