# ADR 0036 – Wiederkehrende Aufgaben

- Status: vorgeschlagen
- Datum: 2026-11-07
- Ergänzt: ADR 0027 (Planungswörter in der Beschreibung), ADR 0033 (Habits entfernt), ADR 0016 (CRDT-Sync)

## Kontext

ADR 0033 hat die Wiederholung von Aufgaben (`rec:`) bewusst offen gelassen. Wer regelmäßige Aufgaben hat
(Gießen, Miete, Wochenbericht), legt sie bisher von Hand neu an.

## Entscheidung

- **Syntax** nach todo.txt: `rec:[+]<n><d|b|w|m|y>` in der Beschreibung (Tage, Werktage, Wochen, Monate, Jahre;
  n = 1 bis 999). Ungültiges bleibt gewöhnlicher Text. Keine Migration, kein neues Sync-Format; `rec:` ist wie
  `due:`/`est:` ein Planungswort (`PlanKey::Recurrence`, Modul `nunc_core::recurrence`).
- **Strikt mit `+`:** neue Frist = bisherige Frist + Intervall (fehlt `due:`, gilt das Abschlussdatum). **Ohne `+`:**
  Abschlussdatum + Intervall. Werktage überspringen Samstag und Sonntag. Monate und Jahre enden am letzten Tag des
  Zielmonats, wenn der Tag dort fehlt (31.1. + 1m = 28.2.; 29.2.2024 + 1y = 28.2.2025).
- **Folgeaufgabe beim Abhaken**, im Store (`set_todo_completed`) in derselben Transaktion: gleiche Zeile mit neuer
  Frist, gleiche Priorität, Projekte und Kontexte, Erstellungsdatum = heute, offen; `plan:` entfällt (gehörte zum
  alten Durchlauf). Die Folgeaufgabe wird hinten angehängt. Das Command liefert weiter nur das abgehakte Todo; die UI
  lädt über das Ereignis `todos_changed` neu.
- **Nur Aufgaben der obersten Ebene** wiederholen. Eine Unteraufgabe mit `rec:` und Unteraufgaben, die beim Abhaken
  des Elternknotens miterledigt werden, erzeugen nichts (ADR 0022). Unteraufgaben werden nicht kopiert.
- **Zurücknehmen** löscht die Folgeaufgabe nicht; erneutes Abhaken erzeugt sie nicht ein zweites Mal.
- **Sync:** Die Sync-ID der Folgeaufgabe ist deterministisch aus der Sync-ID der Vorlage und der neuen Frist
  (FNV-1a 128 Bit, UUID-förmig). Haken zwei Geräte dieselbe Aufgabe unabhängig ab, entsteht dieselbe Folgeaufgabe
  und der CRDT-Merge führt sie zusammen. Haken sie mit verschiedenem Abschlussdatum ab (nur ohne `+`), entstehen
  zwei Folgeaufgaben mit verschiedener Frist; das ist hinnehmbar und von Hand löschbar.
- **UI:** Der Bearbeiten-Dialog hat ein Feld „Wiederholung“ (Anzahl, Einheit, Basis); `rec:` erscheint nicht im
  Titel der Listen und steht in den Aufgabendetails.

## Verworfen

- Folgeaufgabe im Frontend anlegen: bricht die Atomarität, doppelt sich bei jedem Client (API, UI).
- Eigene Tabelle oder Spalte für die Regel: Migration ohne Nutzen gegenüber einem Planungswort.
- Unteraufgaben mitkopieren: braucht Regeln für Teilabschlüsse; nicht Teil der ersten Version.
