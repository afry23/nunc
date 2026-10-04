# Umstieg von der Go-App (`nunc-go`) auf nunc (Tauri)

nunc ersetzt die Go/Wails-App. Aufgaben, Sessions, Einstellungen und Sicherungen bleiben **kompatibel**; Zugänge zu
Konten müssen neu verbunden werden.

## Was übernommen wird

- **Datenbank:** beim ersten Start sucht nunc die Datenbank der Go-App (`nunc/nunc.db` im Benutzer-Konfigurationsordner:
  macOS `~/Library/Application Support/nunc/`, Windows `%AppData%\nunc\`, Linux `~/.config/nunc/`) und **kopiert** sie in den
  Datenordner von nunc (`…/com.afry.nunc/`). Das Original bleibt unverändert; die Kopie wird geprüft und migriert, erst dann
  an ihren Platz gelegt. Gibt es in nunc schon eine Datenbank, passiert nichts.
- **Backup-Ordner:** ein in der Go-App eigens gewählter Ordner wird übernommen. Backups im Standardordner der Go-App
  (`…/nunc/backups`) werden **nicht** verschoben; sie lassen sich über *Einstellungen → Datensicherung → Aus Datei
  wiederherstellen* einspielen.
- **Sicherungen der Go-App** (`.db`) sind in nunc wiederherstellbar.
- **Sync-Datei** `nunc-sync.json` (Google Drive, WebDAV) hat dasselbe Format: nunc und die Go-App können abwechselnd
  über dasselbe Konto synchronisieren (Interop-Test: `tests/interop/webdav-go-nunc.sh`).

## Was neu eingerichtet werden muss

Die Go-App legt Tokens und Passwörter im Schlüsselspeicher ab. nunc übernimmt sie **bewusst nicht** (ADR 0005) – Konten
neu verbinden:

1. *Einstellungen → Synchronisierung:* WebDAV (Passwort neu eingeben), Google Drive (Anmeldung im Browser).
2. *Einstellungen → Todoist:* neu anmelden. Danach gleicht nunc die Aufgaben über die vorhandene Zuordnung ab; es sollten
   keine Duplikate entstehen.

## Schritt für Schritt

1. Go-App beenden, ggf. eine letzte Sicherung anlegen (*Jetzt sichern*).
2. nunc installieren und starten. Aufgaben, Statistik und Einstellungen sind da.
3. Konten neu verbinden (siehe oben), Sync einmal manuell auslösen und die Statusleiste prüfen.
4. Go-App erst löschen, wenn alles stimmt – sie und nunc können parallel auf **getrennten** Datenbanken laufen, aber nicht
   auf derselben Datei. Zwei Geräte gleichen sich nur über den Sync ab.

## Unterschiede

| | Go-App | nunc |
|---|---|---|
| Datenordner | `…/nunc/` | `…/com.afry.nunc/` |
| Standard-Sicherungsordner | `…/nunc/backups` | `…/com.afry.nunc/backups` |
| Schlüsselspeicher | Eigener Eintrag der Go-App | Dienst `nunc`, Konten `webdav`, `google-drive`, `todoist` |
| Google Drive Scope | `drive.file` + `userinfo.email` | nur `drive.file` (E-Mail über Drive `about`) |
| Plattformen | Desktop | Desktop, Android, iOS |
| Flowtime-Bucket-Modus | vorhanden (`flowtime_rules`) | nicht übernommen (Backlog) |
| Sprache | Deutsch | Deutsch, Englisch |

Dateien der Go-App (Datenbank, Sicherungen) bleiben unangetastet; ein Zurück zur Go-App ist jederzeit möglich, solange die
Datenbank nicht in nunc **und** der Go-App gleichzeitig geändert wurde.
