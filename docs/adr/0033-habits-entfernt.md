# ADR 0033 – Habit-Tracker entfernt

- Status: vorgeschlagen
- Datum: 2026-11-06
- Ersetzt: ADR 0028 (Habit-Tracker)
- Ergänzt: ADR 0031 (Informationsarchitektur), ADR 0032 (Tagesfokus)

## Kontext

Der Umbau ([ADR 0031](./0031-ui-informationsarchitektur.md)) richtet nunc an einem Arbeitsablauf aus: Heute planen,
Aufgaben pflegen, Zeiten prüfen, zurückblicken. Gewohnheiten sind ein eigener Bereich ohne Platz in diesem Ablauf; sie
waren nur über einen Link erreichbar und erzeugten eine zweite Art von „Aufgabe“ neben den Todos. Entscheidung des
Nutzers: der Habit-Tracker wird **vollständig entfernt**.

## Entscheidung

- **Oberfläche:** Ansicht, Formular, Store, Route `/habits`, der Block in „Heute“ und der Abschnitt im Rückblick entfallen.
  Wiederkehrende konkrete Arbeit ist eine Aufgabe; Routinen liegen in Heute („Tag planen“, Tagesmenü „Tag abschließen“)
  und im Rückblick („Wochenrückblick beginnen“).
- **Fachlogik:** `nunc_core::habit`, der Habit-Anteil des Berichts (`Review.habits`, Markdown-Abschnitt), die
  Habit-Felder von `SyncState`/`SyncDocument`, `habit_repo` und die fünf Commands entfallen.
- **Store:** Migration `017_drop_habits.sql` löscht `habits`, `habit_field_clock` und `habit_logs`. Migration 015 bleibt
  unverändert (sie legt die Tabellen auf Bestandsdatenbanken weiter an, 017 räumt sie ab). Die Daten gehen verloren;
  ein Backup vor dem Update (Backup-Datei oder Auto-Backup) enthält sie weiter, und ein Restore einer solchen Datei
  funktioniert, weil die Migrationen danach laufen.
- **Sync:** `SCHEMA_VERSION` wird **11** (gelesen 2 bis 11). Die Listen `habits` und `habit_logs` älterer Dateien werden
  beim Lesen ignoriert (serde überspringt unbekannte Felder). Ältere Clients lehnen Format 11 mit „Bitte nunc
  aktualisieren“ ab (wie ADR 0017/0029); dort liegen noch Habits, die nicht mehr geschrieben werden.
- **Nicht entschieden:** Wiederholung von Aufgaben (`rec:`) gibt es in nunc nicht; sie wird hier nicht nachgerüstet.

## Folgen

- Alle Geräte müssen aktualisiert werden, sonst synchronisieren die älteren nicht mehr (Release-Notes in
  [release.md](../release.md)).
- Der Datenverlust der Habits ist gewollt und endgültig; ein Export vor dem Entfernen ist nicht eingebaut. Wer die Daten
  braucht, sichert vor dem Update.
