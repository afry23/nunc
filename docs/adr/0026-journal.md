# ADR 0026 – Journal: Tages-/Wochennotizen und Erfolge

- Status: vorgeschlagen
- Datum: 2026-11-01
- Ergänzt: ADR 0025 (Rückblick), ADR 0016 (CRDT-Sync), ADR 0017 (Sessions als CRDT)

## Kontext

Der Rückblick (ADR 0025) rechnet nur aus Roh-Sessions und Todos. Für Wochen-/Monatsberichte und die Vorbereitung
von Feedback-Gesprächen fehlt, was sich nicht messen lässt: ein kurzer Text zum Tag oder zur Woche und Erfolge, die
man später nachlesen will („Release ausgeliefert“, „Kunden-Workshop geleitet“). Sie sollen unabhängig von Todos und
Sessions existieren (ein Todo kann gelöscht werden, ein Erfolg bleibt) und über alle Geräte synchron sein.

## Entscheidung

**Ein gemeinsamer Eintragstyp `JournalEntry`** (rein in `nunc-core`, zustandsbasierter CRDT wie `TodoCrdt` und
`SessionCrdt`) mit drei Arten:

| Art (`kind`) | Bedeutung | `date` | Identität (`sync_id`) |
|---|---|---|---|
| `day_note` | Notiz zu einem Kalendertag | der Tag | `note:day:YYYY-MM-DD` |
| `week_note` | Wochenrückblick einer Kalenderwoche | **Sonntag** der Woche | `note:week:YYYY-MM-DD` |
| `achievement` | markierter Erfolg, beliebig viele je Tag | Tag des Erfolgs | UUID (vom Gerät vergeben) |

- **Felder:** `sync_id`, `kind` und die Identität gelten ab der Anlage und ändern sich nicht. `date`, `text` und
  `deleted` sind LWW-Register `(Wert, HLC)` (`hlc.rs`). Der Merge ist kommutativ, assoziativ und idempotent;
  Gleichstand entscheidet wie bei Todos der größere Wert bzw. die `device_id`.
- **Deterministische Identität für Notizen:** Zwei Geräte, die dieselbe Tages- oder Wochennotiz anlegen, bilden
  dieselbe `sync_id`; es entstehen keine Dubletten, der spätere Schreibvorgang gewinnt. Erfolge sind dagegen
  unabhängige Einträge (UUID).
- **Wochennotiz am Sonntag:** Die Woche läuft Montag bis Sonntag (wie `PeriodKind::Week`, ADR 0025). Die Notiz wird
  unter dem **Sonntag** der Woche abgelegt, also dem Ende des Rückblick-Zeitraums, nicht unter dem Montag: Man
  schreibt sie am Ende der Woche. Bearbeiten lässt sie sich die ganze Woche; die Oberfläche hebt den Wochenrückblick
  ab Freitag hervor. Wer freitags schreibt und sonntags ergänzt, bearbeitet dieselbe Notiz. `week_end(day)` ist eine
  pure Funktion in `nunc-core`.
- **Leerer Text löscht:** Ein leerer (nach `trim`) Text einer Notiz setzt `deleted`; es gibt keine leeren Notizen. Wird
  der Tag später wieder beschrieben, setzt das jüngere `deleted = false` die Notiz zurück. Gelöschte Einträge sind
  Tombstones und werden wie bei Todos nach 90 Tagen eingesammelt (`TOMBSTONE_TTL_MS`, `is_collectable`). Dasselbe
  Risiko wie in ADR 0016: Ein länger als 90 Tage offline gewesenes Gerät legt den Eintrag wieder an.
- **Grenzen:** Text höchstens 4000 Zeichen (Validierung im Command; `sanitized` kappt Remote-Texte und Stempel mit
  mehr als 24 h Drift wie bei Todos). Erfolge tragen keine eigenen Tags; `+projekt`/`@kontext` im Text sind
  gewöhnlicher Text.
- **Store:** additive Migration `014_journal.sql` (001–005 bleiben unberührt):
  `journal_entries(id INTEGER PK, sync_id UNIQUE, kind CHECK IN ('day_note','week_note','achievement'), entry_date,
  text, deleted)` und `journal_field_clock(entry_id, field, hlc…)` nach dem Muster von `todo_field_clock`.
  Mehrschrittige Schreibvorgänge laufen in einer Transaktion; `list_journal` blendet Tombstones aus. Backups älterer
  Versionen bleiben restorebar (die Tabelle entsteht per Migration).
- **Sync:** `SyncState.journal: BTreeMap<sync_id, JournalEntry>`, in der Datei `nunc-sync.v2.json` die Liste
  `journal` (`serde(default)`). `schema_version` wird **7**; gelesen werden 2 bis 7, geschrieben wird 7. Ältere
  Clients lehnen v7 ab („Bitte nunc aktualisieren“, strikter Versionscheck wie in ADR 0017), statt das Journal still
  zu verwerfen und beim Zurückschreiben zu löschen. Der Dateiname bleibt.
- **Einbindung in den Rückblick:** `build_review` bekommt die Einträge als Eingabe (pur, nichts abgeleitet gespeichert)
  und liefert `ReviewJournal`: Tagesnotizen mit `date` im Zeitraum, Wochennotizen mit `date` (Sonntag) im Zeitraum,
  Erfolge mit `date` im Zeitraum, jeweils nach Datum sortiert. Ein Monats- oder Quartalsrückblick zeigt damit die
  Wochennotizen der Wochen, die in ihm **enden**. Der Markdown-Export (mehrsprachig, ADR 0025) bekommt die Abschnitte
  „Erfolge“, „Wochenrückblick“ und „Tagesnotizen“ (nur wenn vorhanden).
- **Commands** (dünn, sperren nur `Db`): `get_journal(from, to)`, `save_journal_note(kind, date, text)` (nur
  `day_note`/`week_note`; das Datum einer Wochennotiz wird auf den Sonntag normalisiert),
  `add_achievement(date, text)`, `update_achievement(id, date, text)`, `delete_achievement(id)`. Fehler als
  `AppError` (`error.<code>` in `de.ts`/`en.ts`).
- **Oberfläche:** Bearbeiten im Rückblick, keine eigene Ansicht (die Tab-Leiste ist bei ≈ 411 px schon eng):
  Notizfeld je Tag in der Liste „Tage“, Wochennotiz oben bei Wochenrückblicken, Liste „Erfolge“ mit Hinzufügen,
  Bearbeiten, Löschen. Speichern beim Verlassen des Felds.

## Verworfene Alternativen

- **Erfolg als Markierung an Todo oder Session:** braucht ein neues Feld im Todo-CRDT bzw. in `session_meta`, bindet
  den Erfolg an Dinge, die gelöscht werden können, und deckt Erfolge ohne Aufgabe (Gespräche, Entscheidungen) nicht ab.
- **Wochennotiz am Montag:** Der Montag ist der Beginn der Woche, an dem es noch nichts zurückzublicken gibt.
- **Je Art eine eigene Tabelle und ein eigener CRDT:** dreimal derselbe Merge, derselbe Tombstone und dieselbe
  Sync-Verdrahtung; die Arten unterscheiden sich nur in Datum und Identität.
- **Notizen in der todo.txt oder als Pseudo-Todos:** vermüllt die Aufgabenliste und die Exporte.
- **Eigene Journal-Ansicht:** mehr Oberfläche, ein weiterer Tab; der Rückblick ist der Ort, an dem man die Einträge
  braucht.

## Folgen

- Downgrade: Ein Gerät mit nunc < Version dieser Änderung liest `nunc-sync.v2.json` im Format 7 nicht.
- Reihenfolge der Umsetzung (je ein PR): Kern-CRDT und Merge-Szenarien → Migration und Store → Sync-Einbindung und
  Format 7 → Commands und Review-Anteil → Oberfläche und Markdown.
- Offen: Erinnerung an den Wochenrückblick (Notification), Tags an Erfolgen, Suche über alle Einträge. Der MCP-Server
  (später) bekommt das Journal nur lesend.
