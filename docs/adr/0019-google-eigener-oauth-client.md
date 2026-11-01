# ADR 0019 – Google Drive nur mit eigenem OAuth-Client der Nutzer

- Status: ersetzt durch ADR 0024 (Google Drive entfernt)
- Datum: 2026-10-11
- Ersetzt teilweise: ADR 0009 (Abschnitt „Anmeldung“, Zugangsdaten für Google)
- Referenz: ADR 0005 (SecretStore), `src-tauri/src/credentials.rs`, `crates/nunc-sync/src/drive/mod.rs`

## Kontext

Bisher setzt der Release-Build Client-ID und -Secret eines Google-„Desktop“-Clients des Maintainers ein
(`option_env!`, GitHub-Secrets in `ci.yml`/`release.yml`). Das Secret eines Desktop-Clients ist laut Google nicht
vertraulich und steckt in jeder ausgelieferten Binary. Für eine Open-Source-App bleiben trotzdem Lasten beim
Maintainer:

- Jeder kann den Client aus der Binary ziehen und unter dem Namen „nunc“ im Zustimmungsdialog auftreten.
- Alle Nutzer teilen sich ein API-Kontingent; Missbrauch oder eine Sperre durch Google trifft alle.
- Pflege des Google-Cloud-Projekts (Zustimmungsbildschirm, Markenprüfung, Kontakt bei Beschwerden).
- Forks und Paketierer (Flathub, F-Droid) bauen ohne die Secrets; Drive ist dort ohnehin abgeschaltet.

Der Maintainer möchte für Google keinen eigenen Client mehr betreiben.

## Entscheidung

- **Kein eingebauter Google-Client mehr.** `credentials::google()` liest keinen Build-Wert (`option_env!`) mehr. Die
  GitHub-Secrets `NUNC_GOOGLE_CLIENT_ID`/`NUNC_GOOGLE_CLIENT_SECRET` werden aus `ci.yml` und `release.yml` entfernt und
  im Repository gelöscht; der bisherige Client wird im Google-Cloud-Projekt nach dem Release gelöscht.
- **Nutzer tragen ihren eigenen Client ein:** In den Einstellungen unter „Google Drive“ gibt es die Felder
  „Client-ID“ und „Client-Secret“ (Secret als Passwortfeld). Ohne Client-ID ist „Mit Google Drive verbinden“
  deaktiviert (wie heute bei fehlenden Zugangsdaten), daneben steht die Anleitung.
- **Ablage nur im Tresor** (ADR 0005): Dienst `nunc`, Konto `google-drive-client`, Wert als JSON
  `{ "clientId": …, "clientSecret": … }`. Nicht in Settings, DB, Backup oder Sync-Datei. Damit gehört der Client zum
  Gerät; jedes Gerät braucht ihn einmal (derselbe Client darf auf mehreren Geräten genutzt werden).
- **Laufzeitänderung:** `DriveConfig` wird nicht mehr nur beim Start gebaut. `DriveService` liest den Client bei jeder
  Anmeldung und Token-Erneuerung aus dem Tresor (oder hält ihn hinter einem `RwLock` und bekommt ein `set_client`).
  Ändert sich die Client-ID, wird die bestehende Verbindung getrennt (Tokens gelten nur für ihren Client) und der
  Status zeigt „Bitte neu verbinden“. Neue Commands `set_google_client` / `clear_google_client`; der Drive-Status
  zeigt mit `configured` und `clientId`, ob und welcher Client eingetragen ist. Das Secret geht nie zurück ans Frontend.
- **Validierung:** Client-ID muss auf `.apps.googleusercontent.com` enden (sonst `AppError`
  `google_client_invalid`). Das Secret ist Pflicht: Google verlangt es bei Desktop-Clients auch mit PKCE beim
  Tokentausch.
- **Entwicklung:** Debug-Builds lesen weiter Umgebung und `.env` (`NUNC_GOOGLE_CLIENT_ID` …) als Vorrang vor dem
  Tresor, damit Tests und `tauri dev` ohne Eingabe laufen. Release-Builds lesen nur den Tresor.
- **Anleitung in der App** (Texte in `de.ts`/`en.ts`, Link über `UrlOpener`), aufklappbar unter den Feldern:

  1. In der Google Cloud Console ein Projekt anlegen
     (`https://console.cloud.google.com/projectcreate`).
  2. „Google Drive API“ aktivieren
     (`https://console.cloud.google.com/apis/library/drive.googleapis.com`).
  3. OAuth-Zustimmungsbildschirm: Typ „Extern“, App-Name frei wählbar, eigene E-Mail. Scope
     `…/auth/drive.file` hinzufügen.
  4. Veröffentlichungsstatus auf **„In Produktion“** setzen. Im Status „Testen“ laufen Refresh-Tokens nach 7 Tagen
     ab und die Verbindung bricht wöchentlich. `drive.file` ist kein sensibler Scope, eine Prüfung durch Google ist
     nicht nötig.
  5. Anmeldedaten → „OAuth-Client-ID erstellen“ → Typ **„Desktop-App“**
     (`https://console.cloud.google.com/apis/credentials`). Client-ID und -Secret hier eintragen.

  Dazu ein Satz, warum: „nunc bringt keinen eigenen Google-Zugang mit. Mit deinem eigenen Client bleiben Zugriff und
  Kontingent bei dir.“ Ausführlicher steht dasselbe in `docs/google-drive.md`.

## Folgen

- **Bestehende Nutzer** verlieren mit dem Update die Drive-Verbindung (ihre Tokens gehören zum alten Client). Beim
  ersten Start nach dem Update: Hinweis „Google Drive braucht jetzt einen eigenen Zugang“ mit Link zur Anleitung. Die
  Sync-Datei in Drive bleibt erhalten; mit dem neuen Client ist sie **nicht** sichtbar (`drive.file` sieht nur Dateien,
  die derselbe Client angelegt hat). Der erste Lauf mit neuem Client legt `nunc/nunc-sync.v2.json` neu an und lädt den
  lokalen Stand hoch. Andere Geräte müssen denselben neuen Client nutzen, sonst sehen sie die Datei nicht. Das steht in
  den Release-Notes und in der Anleitung („auf allen Geräten dieselbe Client-ID“).
- Einstiegshürde für Drive steigt deutlich (Cloud-Konsole). WebDAV bleibt der einfache Weg; die Anleitung nennt ihn als
  Alternative.
- Kein Google-Projekt, keine Markenprüfung, kein geteiltes Kontingent beim Maintainer.
- Tests: `@ui`-Szenarien für Eingabe, Deaktivierung ohne Client, Anzeige der Anleitung; `@integration @sync` für
  Client aus dem Tresor, Trennen bei geänderter Client-ID. Bestehende Drive-Szenarien bekommen den Client über den
  Tresor statt über `DriveConfig::google`.
- Offen: Todoist (ADR 0020), Drive auf Mobilgeräten (dort bräuchte es Plattform-Clients pro Nutzer, Backlog).
