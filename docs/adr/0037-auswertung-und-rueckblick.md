# ADR 0037 – Auswertung und Rückblick getrennt

- Status: vorgeschlagen
- Datum: 2026-11-08
- Ergänzt: ADR 0025 (Rückblick), ADR 0031 (Informationsarchitektur)

## Kontext

Die Timerstatistik hing als Link „Timerstatistik öffnen“ am Ende des eingeklappten Abschnitts „Arbeitsweise“ im
Rückblick; ihre Diagramme waren dadurch kaum auffindbar. Der Rückblick war ein langer Stapel aus zehn Abschnitten. Es
fehlte die direkte Antwort auf „woran habe ich wie lange gearbeitet, ist es erledigt?“, freie Tätigkeiten erschienen nur
als Tag-Kombination, Quartal fehlte in der Statistik, und deren Zeiträume waren gleitende Fenster ohne Blättern.

## Entscheidung

Zwei Bereiche mit gleicher Zeitraumwahl (`PeriodNav`: Tag · Woche · Monat · Quartal · Jahr, Kalenderzeiträume, Blättern):

- **Auswertung** (`/stats`, fünfter Hauptbereich, Kürzel 5): erfasste Arbeitszeit mit Vorperiode, Balkendiagramm
  (Tage; Wochen im Quartal; Monate im Jahr; gestapelt nach Zeitmessung), Kreisdiagramme und Listen für Projekte und
  Kontexte, Tageszeitverteilung, Tabelle der Aufgaben (Zeit, Sessions, Status erledigt/offen, Schätzung) mit
  Projektfilter, freie Tätigkeiten je Session. Streaks, Heatmap und Pomodoro-Linien liegen im eingeklappten
  Abschnitt „Weitere Kennzahlen“. Export CSV/JSON bleibt.
- **Rückblick** (`/review`): bearbeitete Aufgaben mit Status, freie Tätigkeiten, Erfolge, Notizen aus den Tagen,
  Reflexion, Folgeaufgabe – ohne Diagramme und Zeiten. Link „Zahlen und Diagramme ansehen“ führt zur Auswertung.
  Abschnitte „Erfasste Arbeitszeit“, „Arbeit nach Projekten/Kontexten“, „Erledigte Aufgaben“, „Bearbeitet, aber noch
  offen“, Tag-Kombinationen und „Arbeitsweise“ entfallen im Rückblick (Zahlen stehen in der Auswertung).

Beide lesen denselben Bericht (`get_review`); die Statistik-Antwort (`get_stats`) liefert nur noch Streaks, Heatmap
und Export. `Review` wächst um `tasks[].completed/completedAt/projects/estimateCheck`, `activities` (Sessions ohne
Aufgabe, einzeln), `hours` (Minuten je lokaler Stunde, über Stunden- und Tagesgrenzen verteilt) und
`days[].pomodoroMin/flowtimeMin/stopwatchMin`. `PeriodKind` kennt Tag und Jahr. `build_review` bekommt die lokale Zeit
(`NaiveDateTime`) statt nur des lokalen Tages. Keine Schemaänderung, nichts wird abgeleitet gespeichert.

## Folgen

- Kürzel 4 bleibt der Rückblick, 5 ist neu (Auswertung).
- Im Rückblick fehlen erledigte Aufgaben, an denen im Zeitraum nicht gearbeitet wurde, sowie die Schätzungs-Trefferquote;
  `completed_todos` und `estimate_summary` bleiben im Bericht (Markdown-Export), sind aber nicht mehr in der Oberfläche.
