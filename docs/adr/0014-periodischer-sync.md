# ADR 0014 – Periodischer Sync (Desktop)

- Status: akzeptiert
- Datum: 2026-10-05
- Referenz: ADR 0008 (WebDAV-Sync), ADR 0009 (Drive, Todoist), `features/sync/periodic.feature`

## Kontext

Der Sync lief nur auf Anlass: Start, Ende einer Session, Todo-Änderung, Beenden (Desktop) sowie Wechsel in und aus dem
Hintergrund (Mobil). Eine Desktop-App, die tagelang offen bleibt, sah Änderungen eines zweiten Geräts daher erst bei
der nächsten eigenen Aktion.

## Entscheidung

- **Takt im Koordinator.** `SyncCoordinator::set_interval(Option<Duration>)` startet eine Schleife (`sleep`, dann
  `sync_now`). Ein neuer Aufruf ersetzt sie (Generationszähler wie beim Entprellen), `None` schaltet sie ab. Der erste
  Lauf folgt nach einem vollen Intervall, weil beim Start ohnehin synchronisiert wird. Der nächste Abstand beginnt erst
  nach dem Ende eines Laufs; dadurch überlappen Läufe nicht, und die vorhandene Serialisierung (`gate`) bleibt die
  einzige Sperre. Die Schleife hält den Koordinator nur schwach (`Weak`) und endet mit ihm.
- **Nur Desktop.** Mobil ist `set_interval` wirkungslos; dort frieren Betriebssysteme Hintergrundaufgaben ein, und
  Resume/Background bleiben die Auslöser.
- **Nur der Haupt-Sync.** Todoist (Rate-Limits, Zuordnung per Anfragen) reagiert weiter nur auf Start,
  Todo-Änderung und Verbinden.
- **Einstellung.** `syncIntervalMin` in den Einstellungen (Schlüssel `sync_interval_min`), erlaubt 0 (aus), 5, 15, 30;
  Standard 15. Kein Schema-Eingriff: `settings` ist Schlüssel/Wert, die Go-App ignoriert den Schlüssel (wie `language`
  und `api_*`). `Settings::sync_interval()` rechnet in eine `Duration` um (pur).
- **Verdrahtung.** `AppState::apply_sync_interval` wird beim Start, nach `save_settings` und nach einem Restore
  aufgerufen. Ein Restore kann den gespeicherten Wert ändern.
- **Kein Push.** Weder WebDAV noch Drive melden Änderungen von sich aus; ein Dienst im Hintergrund oder Webhooks
  wären für eine lokale App unverhältnismäßig.

## Folgen

- Ohne Änderung auf dem Server kostet jeder Lauf einen Abruf der Sync-Datei; deshalb Standard 15 Minuten und
  abschaltbar.
- Eine laufende Mobil-App sieht Änderungen weiterhin erst beim Wechsel in den Vordergrund.
- Fehler eines Takt-Laufs stehen wie bei jedem Lauf im Sync-Status; die Schleife läuft weiter.
