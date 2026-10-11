# Google Drive mit eigenem Google-Zugang

nunc bringt keinen eigenen Google-Zugang mit ([ADR 0019](./adr/0019-google-eigener-oauth-client.md)). Wer über Google
Drive synchronisieren möchte, legt einmal einen eigenen OAuth-Client an und trägt ihn in nunc ein. Zugriff und
Kontingent bleiben so beim eigenen Google-Konto. Die Einrichtung dauert etwa zehn Minuten.

Ohne Google-Konsole geht es mit WebDAV (z. B. Nextcloud), siehe *Einstellungen → Sync*.

## 1. Projekt anlegen

In der Google Cloud Console ein neues Projekt anlegen: <https://console.cloud.google.com/projectcreate>.
Der Name ist frei wählbar, z. B. „nunc-sync“.

## 2. Google Drive API aktivieren

Im neuen Projekt die „Google Drive API“ aktivieren:
<https://console.cloud.google.com/apis/library/drive.googleapis.com>.

## 3. OAuth-Zustimmungsbildschirm

Unter „APIs und Dienste → OAuth-Zustimmungsbildschirm“ (bzw. „Google Auth Platform“):

- Nutzertyp **Extern**.
- App-Name frei wählbar (er erscheint später im Anmeldedialog), eigene E-Mail-Adresse als Support- und
  Entwickler-Kontakt.
- Unter „Datenzugriff“ bzw. „Bereiche“ den Scope `https://www.googleapis.com/auth/drive.file` hinzufügen. Mehr braucht
  nunc nicht: Die App sieht nur Dateien, die sie selbst angelegt hat.

## 4. Veröffentlichen

Den Veröffentlichungsstatus auf **„In Produktion“** setzen.

Im Status „Testen“ laufen Anmeldungen nach **7 Tagen** ab, die Verbindung bricht dann jede Woche ab. `drive.file` ist
kein sensibler Scope; eine Prüfung durch Google ist für die eigene Nutzung nicht nötig.

## 5. Client anlegen

Unter „Anmeldedaten“ (<https://console.cloud.google.com/apis/credentials>) → „Anmeldedaten erstellen“ →
„OAuth-Client-ID“:

- Anwendungstyp **Desktop-App**, Name frei wählbar.
- Nach dem Erstellen **Client-ID** (endet auf `.apps.googleusercontent.com`) und **Clientschlüssel** (Client-Secret)
  kopieren.

## 6. In nunc eintragen

*Einstellungen → Sync → Google Drive*: Client-ID und Client-Secret eintragen, „Zugang speichern“, dann „Mit Google
Drive verbinden“. Der Browser öffnet die Google-Anmeldung.

- Beide Werte liegen nur im Schlüsselbund des Geräts (Dienst `nunc`, Konto `google-drive-client`), nicht in der
  Datenbank, im Backup oder in der Sync-Datei.
- **Auf allen Geräten dieselbe Client-ID eintragen.** Mit `drive.file` sieht jeder Client nur seine eigenen Dateien;
  Geräte mit verschiedenen Clients sehen die Sync-Datei der anderen nicht.
- Eine andere Client-ID trennt die bestehende Verbindung (Tokens gelten nur für ihren Client). Ein neues Secret zur
  selben Client-ID behält sie.

## Umstieg von einer Version mit eingebautem Zugang

Bis nunc 0.3 war ein Google-Client eingebaut. Nach dem Update fragt nunc nach einem eigenen Zugang. Die alte
`nunc/nunc-sync.v2.json` in Drive ist für den neuen Client nicht sichtbar: Der erste Sync legt eine neue Datei an und
lädt den lokalen Stand hoch. Ein Gerät mit den vollständigsten Daten zuerst verbinden, danach die anderen (mit
derselben Client-ID). Die alte Datei und den Ordner kann man danach in Drive löschen.
