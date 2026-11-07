# ADR 0034 – Tagesfokus entfernt

- Status: angenommen
- Datum: 2026-11-07
- Ersetzt: ADR 0032 (Tagesfokus)
- Ergänzt: ADR 0031 (Informationsarchitektur), ADR 0033 (gleiches Muster wie bei den Gewohnheiten)

## Kontext

Der Tagesfokus („Was ist heute das Wichtigste?“) wird nicht gebraucht. „Heute“ soll eine ruhige Übersicht des Tages
sein (erfasste Arbeit, Plan gegen Kapazität, Aufgaben, freie Tätigkeiten); der Timer liegt im Reiter „Timer“.
Entscheidung des Nutzers: der Fokus wird **vollständig** entfernt, auch aus dem Modell.

## Entscheidung

- **Oberfläche:** Feld, Store-Zustand (`focus`, `loadFocus`, `saveFocus`), `DayFocus` und der Abschnitt „Fokus der Tage“
  im Rückblick entfallen.
- **Fachlogik:** `JournalKind::DayFocus`, `JournalEntry::day_focus`, `day_focus_id`, `ReviewJournal.day_focus` und der
  Markdown-Abschnitt entfallen; `nunc_store::save_day_focus` und der Command `get_day_focus` ebenfalls.
  `save_journal_note` kennt nur noch `day_note` und `week_note`.
- **Store:** Migration `018_drop_day_focus.sql` löscht die Einträge der Art `day_focus` samt Stempeln und baut
  `journal_entries` ohne diese Art im `CHECK` neu (Stempel werden gesichert, weil ein `DROP TABLE` sie über
  `ON DELETE CASCADE` leeren würde). Migration 016 bleibt unverändert. Der Verlust der Fokus-Texte ist gewollt.
- **Sync:** `SCHEMA_VERSION` wird **12** (gelesen 2 bis 12). Journal-Einträge mit unbekannter Art (der Fokus in Dateien
  bis Format 11) werden beim Lesen verworfen, statt die ganze Datei abzulehnen. Ältere Clients lehnen Format 12 mit
  „Bitte nunc aktualisieren“ ab (strikter Versionscheck wie in ADR 0017).

## Folgen

- Backups mit Tagesfokus bleiben restorebar (Migration 018 läuft nach dem Restore, der Fokus verschwindet).
- Geräte müssen vor dem nächsten Sync aktualisiert werden (Release Notes).
