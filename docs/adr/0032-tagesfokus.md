# ADR 0032 – Tagesfokus

- Status: ersetzt durch ADR 0034
- Datum: 2026-11-06
- Ergänzt: ADR 0026 (Journal), ADR 0027 (Tagesplaner), ADR 0031 (Informationsarchitektur)

## Kontext

„Heute“ soll mit einem kurzen Satz beginnen, was an diesem Tag das Wichtigste ist („API fertigstellen“). Er ist
kein Plan und keine Notiz: Er steht oben, ist ein Satz lang und wird morgens gesetzt. Die Tagesnotiz (ADR 0027)
ist dafür zu groß, sie ist der Raum für den Tagesabschluss.

## Entscheidung

- **Neue Journal-Art `day_focus`** (ADR 0026): `JournalEntry::day_focus`, deterministische Kennung
  `note:focus:YYYY-MM-DD`, damit zwei Geräte für denselben Tag denselben Eintrag bilden (der spätere Schreibvorgang
  gewinnt). Ein leerer Text löscht den Fokus, ein späterer Text stellt ihn wieder her – wie bei den Notizen.
- **Grenze:** Die Oberfläche begrenzt den Fokus auf 200 Zeichen; das Backend prüft wie bei allen Journal-Texten
  auf 4000.
- **Im Rückblick (seit den Folgearbeiten zu ADR 0031):** `build_review` führt die Fokus-Einträge des Zeitraums als `day_focus`. Ursprünglich: `build_review` ignorierte die Art. Der Fokus gehört zur Planung des Tages, nicht zu den
  Ergebnissen eines Zeitraums.
- **Store:** Migration `016_journal_day_focus.sql`. SQLite kann ein `CHECK` nicht ändern, deshalb wird
  `journal_entries` neu gebaut (Daten und Stempel bleiben erhalten; die Stempel werden gesichert, weil ein
  `DROP TABLE` ihre Fremdschlüssel kaskadierend leeren würde).
- **Sync:** `SCHEMA_VERSION` wird **10** (gelesen 2 bis 10). Ältere Clients lehnen das Format mit „Bitte nunc
  aktualisieren“ ab, statt die unbekannte Art zu verwerfen oder beim Zurückschreiben zu löschen (strikter
  Versionscheck wie in ADR 0017). Das Entfernen des Habit-Trackers (docs/ui-umbau-plan.md E6) erhöht das Format
  danach auf 11 (ADR 0033).
- **Commands:** `get_day_focus(date)`; gespeichert wird über `save_journal_note(kind = "day_focus", …)`.
- **Oberfläche:** Feld „Tagesfokus“ unter der Überschrift von „Heute“, auf dem Telefon über allen Segmenten.
  Gespeichert wird beim Verlassen des Feldes und mit Enter, nur bei geänderter Fassung.

## Folgen

- Backups älterer Versionen bleiben restorebar (die Migration läuft nach dem Restore).
- Geräte mit älterer Version müssen vor dem nächsten Sync aktualisiert werden (in die Release Notes).
