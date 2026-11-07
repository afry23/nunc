# ADR 0031 – Informationsarchitektur: Heute · Aufgaben · Verlauf · Rückblick

- Status: vorgeschlagen
- Datum: 2026-11-06
- Ergänzt: ADR 0025 (Rückblick), ADR 0027 (Tagesplaner)
- Plan: [ui-umbau-plan.md](../ui-umbau-plan.md)

## Kontext

Die Navigation zeigte Werkzeuge (Timer, Aufgaben, Zeiten, Statistik) statt des Arbeitsablaufs. „Heute“ war nur über
einen Link erreichbar, „Zeiten“ mischte Wochenübersicht und Rückblick, der Timer war die Eingangstür der App.

## Entscheidung

Vier gleichrangige Hauptbereiche, auf Desktop (Seitenleiste) und Android (untere Leiste) gleich:

| Route | Bereich | Frage |
|---|---|---|
| `/` | Heute (Startseite) | Was nehme ich mir heute vor? |
| `/tasks` | Aufgaben | Was gibt es zu tun? |
| `/log` | Verlauf | Was ist wirklich passiert? |
| `/review` | Rückblick | Was ist dabei herausgekommen? |

- Der Timer ist keine Startseite mehr, sondern der **Fokusmodus** unter `/focus` (E1: über „Timer“ in Heute erreichbar;
  E2: globale Aktiv-Leiste).
- Eine laufende Session zeigt die **Aktiv-Leiste** (Titel, Zeit, Pause/Fortsetzen, Stop) in allen Bereichen außer dem
  Fokusmodus; sie ist nur ein Abbild des Timer-Snapshots (ADR 0004). Antippen öffnet `/focus`.
- Arbeit startet aus einer Aufgabe („Starten“ im Tagesplan; Zuordnung steht fest, zuletzt benutzter Modus) oder als
  **freie Tätigkeit** (Dialog: Tätigkeit, Projekte, Kontexte, Modus; ADR 0023). Das Feld „Zuordnung ohne Aufgabe“ im
  Timer entfällt.
- **Aufgabenzeile:** Erledigen, Titel, Metadaten (Fortschritt der Unteraufgaben, „Heute“, erfasste Zeit), „Starten“ und ein
  „⋯“-Menü (einplanen, Unteraufgabe, verschieben, einrücken, duplizieren, bearbeiten, löschen). Ein Klick auf den Titel
  öffnet die Details (`/tasks?task=ID`): neben der Liste auf breiten, als eigener Screen auf schmalen Fenstern; Filter
  und Sortierung liegen im Store und bleiben beim Zurück erhalten. Import und Export stehen im Überlaufmenü.
- **Aufgaben auswählen** (`/pick`): der Bestand als Mehrfachauswahl für den Tag; Aufgaben werden nur eingeplant
  (`plan:`), nicht verschoben (ADR 0027).
- **Schmal (unter 768 px):** Die untere Leiste führt nur die vier Hauptbereiche; Einstellungen und Hilfe stehen im
  Menü der Kopfzeile. „Heute“ zeigt Plan, Tagesverlauf und Notizen als Segmente nacheinander, breit nebeneinander.
  „Verlauf“ beginnt mit der Liste eines Tages (die Wochenmatrix ist die Ansicht „Woche“, breit der Standard), im
  „Rückblick“ klappen die Abschnitte auf. Der Tagesverlauf in Heute und im Verlauf ist dieselbe Komponente
  (`DayLog`) auf derselben Datenquelle.
- **Rückblick (E5):** Er beginnt mit den Ergebnissen, nicht mit Timerzahlen. Reihenfolge: Erfasste Arbeitszeit, Ergebnisse
  und Beiträge, Arbeit nach Projekten/Kontexten, Erledigte Aufgaben, Bearbeitet-aber-noch-offen, Notizen aus den Tagen,
  Reflexion (Leitfragen „Was lief gut? Was war schwierig? Was nehme ich mit?“, Wochennotiz), Für den nächsten Zeitraum
  (Folgeaufgabe), zuletzt der eingeklappte Abschnitt „Arbeitsweise“ (Pomodoros, Sessions, Zeitmessung, Tage) mit Link zur
  Timerstatistik. „Wochenrückblick beginnen“ springt in die Reflexion. „Tag abschließen“ liegt im Tagesmenü von Heute.
- **Begriffe:** Die Summe des Rückblicks heißt **Erfasste Arbeitszeit** (alle Modi, auch Stoppuhr und Nachgetragenes);
  **Fokuszeit** ist nur die Teilmenge aus Pomodoro und Flowtime (`focusOnlyMinutes`, aus `modes` abgeleitet). Das Feld
  `focus_min` des Backends behält seinen Namen, der Markdown-Export nennt es „Erfasste Arbeitszeit“.
- Die Timerstatistik (`/stats`) bleibt vorerst bestehen und hängt als „Arbeitsweise“ am Rückblick (E5 ordnet sie ein).
- Alte Adressen `/today`, `/todos`, `/times` leiten auf die neuen um.
- Tastenkürzel 1–4 folgen den vier Bereichen.
- „Heute“ trennt **Geplanter Aufwand** (Schätzungen gegen Budget) von **Erfasster Arbeit** (Vereinigung der
  Arbeitsblöcke des Tages, Pausen zählen nicht, Überlappungen einmal). Die Zahl wird aus den Sessions berechnet, nicht
  gespeichert.

### Folgearbeiten nach E6

- **Ergebnis / nächste Schritte:** Nach einer beendeten Arbeitssession (Ablauf oder Stopp, mindestens eine Minute) fragt
  `SessionEndDialog` optional nach einer Notiz. Sie wird mit `set_session_note` nur als Notiz der Session gespeichert
  (eigener Stempel, Dauer und Zuordnung bleiben); eine vorhandene Notiz wird vorbelegt. Escape und „Überspringen“
  speichern nichts. Die Einstellung `ask_session_result` (Standard an, Abschnitt „Session-Ende“) schaltet das Feld ab; eine
  empfohlene Pause erscheint weiterhin.
- **Mini-Timer:** „Zur App“ öffnet das Hauptfenster auf `/focus` (Ereignis `app:navigate`, nur die Hauptfenster folgen).
- **Aufgabendetails:** `get_task_time` liefert die abgeschlossenen Arbeitssessions einer Aufgabe, neueste zuerst, mit
  Summe und Notiz (pur in `nunc_core::task_time`).
- **Rückblick:** erledigte Aufgaben tragen ihre erfasste Zeit im Zeitraum (`CompletedTodo.minutes`, auch im Markdown);
  der Tagesfokus erscheint als „Fokus der Tage“ (`ReviewJournal.day_focus`, ADR 0032 nimmt „nicht im Rückblick“ zurück).
- **Navigation nach Breite:** unter 600 px Tab-Leiste, 600 bis 1023 px schmale Leiste mit Icons (immer eingeklappt, kein
  Umschalter, unabhängig von der gespeicherten Wahl), ab 1024 px die einklappbare Seitenleiste. Die zweispaltigen
  Layouts der Ansichten gelten weiterhin ab 768 px (`useIsWide`).

## Folgen

- Kein Backend- und kein Schema-Eingriff in dieser Stufe.
- Maestro- und WebdriverIO-Flows starten auf „Heute“ und gehen für den Timer über „Timer“.
- Weitere Etappen: [ui-umbau-plan.md](../ui-umbau-plan.md) (E2–E6, das Entfernen des Habit-Trackers steht in [ADR 0033](./0033-habits-entfernt.md)).

## Nachtrag (ADR 0034)

- „Heute“ und „Timer“ sind zwei Reiter (`DayTabs`, mittig oben): `/` zeigt die Tagesübersicht, `/focus` den Timer.
  „Starten“ an einer Aufgabe wählt sie im Timer vor (`selectTodo`) und öffnet den Reiter „Timer“; Modus wählen und
  Starten passiert dort. Die Schätzung in „Heute“ ist nur für die Planung, nicht die Timer-Dauer.
- Der Tagesfokus (ADR 0032) und „Fokus der Tage“ im Rückblick entfallen.
- **Starten (Änderung):** „Starten“ an einer Aufgabe ist ein geteilter Knopf (`StartButton`): ▶ startet sofort im zuletzt
  benutzten Modus (`activeMode`), der Pfeil daneben wählt Pomodoro, Flowtime oder Stoppuhr, startet ebenfalls sofort und
  merkt den Modus. Danach öffnet sich der Timer. Das ersetzt die Vorwahl der Aufgabe oben; gilt in „Heute“ und in „Aufgaben“.
