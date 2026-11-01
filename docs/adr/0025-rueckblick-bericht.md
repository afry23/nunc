# ADR 0025 – Rückblick-Bericht für Zeiträume

- Status: vorgeschlagen
- Datum: 2026-11-01
- Ergänzt: ADR 0017 (Zeiterfassung), ADR 0023 (Tags an Sessions)

## Kontext

nunc zeigt Zeiten je Projekt/Kontext und Statistiken, aber keinen Rückblick über Wochen oder Monate:
kein Vergleich mit der Vorperiode, keine Liste erledigter Aufgaben, keine Notizen. Für Wochen-/Monatsberichte
und die Vorbereitung von Feedback-Gesprächen soll ein Bericht aus den Rohdaten entstehen.

## Entscheidung

- **Pur in `nunc-core`:** `review::build_review(range, sessions, todos, notes)` liefert einen `Review`
  (Fokuszeit gesamt und je Projekt/Kontext, Vergleich zur gleich langen Vorperiode, erledigte Aufgaben nach
  `completed_at`, Fokus-Qualität: Anzahl Sessions, Ø Länge, Abbruchquote, stärkste Tage). Zeit kommt über
  den Zeitraum, nie über `now()`.
- **Keine abgeleiteten Werte speichern** (Regel aus CLAUDE.md): Der Bericht wird bei jedem Aufruf berechnet.
  Bausteine: `timesheet`, `stats`, `effective_tags`.
- **Zeiträume:** Woche (Montag bis Sonntag), Monat, Quartal, frei. Die Vorperiode ist bei Woche/Monat/Quartal die
  vorherige Kalenderperiode, bei „frei“ der Zeitraum gleicher Länge davor.
- **Notizen:** Tages-/Wochennotiz und markierte Erfolge sind eine spätere, eigene Stufe (neue Migration,
  CRDT-Felder, eigene ADR). Die erste Stufe liefert den Bericht ohne neue Tabellen.
- **Export:** `review_markdown(review, locale)` erzeugt Markdown (rein, ohne I/O); der Command speichert über
  den vorhandenen Dateidialog.
- **Command:** `get_review(from, to)` und `export_review_markdown(from, to)`, dünn, sperren nur `Db`.

## Verworfene Alternativen

- **Bericht in der DB cachen:** widerspricht der Regel „keine abgeleiteten Statistiken speichern“.
- **Bericht im Frontend berechnen:** Fachlogik gehört nach `nunc-core`.

## Folgen

- Keine Migration, kein Sync-Format-Wechsel in dieser Stufe.
- Grundlage für den späteren MCP-Server (read-only `/v1/review`).
