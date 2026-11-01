# ADR 0024 – Nur WebDAV-Sync

- Status: akzeptiert
- Datum: 2026-11-01
- Ersetzt: ADR 0009 (Google Drive und Todoist), ADR 0019 (eigener Google-Client), ADR 0020 (Todoist-Token)
- Referenz: ADR 0008 (WebDAV), ADR 0016 (CRDT-Sync)

## Kontext

Der einzige genutzte Sync-Weg ist WebDAV. Google Drive (OAuth, Loopback-Rückruf, eigener Client) und Todoist
(Token, Zuordnungstabelle, eigener Koordinator) kosten Code, Tests, Abhängigkeiten und Doku, ohne gebraucht zu werden.

## Entscheidung

- Google Drive und Todoist sind vollständig entfernt: Clients, OAuth, Commands, Events, UI, Features, Dokumentation.
  `SyncProvider` kennt nur noch `none` und `webdav`.
- Bewusst **inkompatible Bereinigung** der Bestandsdaten durch die neue Migration `013_drop_todoist_drive.sql`:
  Tabelle `todoist_mappings` wird gelöscht, die Einstellungen `last_synced_at`, `drive_folder_id` und
  `todoist.last_sync_at` ebenso, ein gespeicherter Anbieter `google_drive` wird zu `none`. Die Migrationen 001–005
  bleiben unverändert, sodass Backups älterer Versionen weiter einspielbar sind und danach auf 013 migrieren.
- Ein unbekannter Wert für `sync_provider` fällt beim Laden auf `none` zurück.
- Gespeicherte Tresor-Einträge (`google-drive`, `google-drive-client`, `todoist`) werden nicht aktiv gelöscht; sie
  bleiben ungenutzt im System-Schlüsselbund und können dort von Hand entfernt werden.
- Der HTTP-Client mit Systemzertifikaten liegt jetzt in `nunc-sync/src/http.rs`.

## Folgen

- Weniger Code und Abhängigkeiten (`base64`, `getrandom` in `nunc-sync`; `UrlOpener`-Abstraktion; Rückruf-Ports).
- Wer Drive oder Todoist nutzte, muss auf WebDAV wechseln oder bleibt bei einer älteren Version.
