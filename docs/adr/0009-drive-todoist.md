# ADR 0009 – Google Drive und Todoist (M7)

- Status: ersetzt durch ADR 0024 (Google Drive und Todoist entfernt)
- Datum: 2026-10-04
- Referenz: `nunc-go/internal/services/sync.go`, `sync_drive.go`, `todoist/*.go`, ADR 0005 (SecretStore), ADR 0008 (WebDAV)

## Kontext

M7 bringt die beiden übrigen Integrationen der Go-App auf den Desktop: Google Drive als zweiten Sync-Anbieter (gleiche
Sync-Datei wie WebDAV) und Todoist als eigenständige Todo-Quelle (nur Todos, bidirektional). Beide melden sich per
OAuth 2.0 an. Mobil bleiben sie ausgeschlossen (Backlog nach 1.0: Deep-Links, Plattform-Client-IDs).

## Entscheidungen

### Anmeldung (M7-01)

Ein gemeinsames Modul `nunc-sync::oauth`: Code-Flow mit **PKCE (S256)** und **Loopback-Rückruf** (RFC 8252). Ein
tokio-Server auf `127.0.0.1` nimmt den Rückruf an, der Browser wird über `UrlOpener` (Trait in `nunc-platform`, Adapter
auf `tauri-plugin-opener`) geöffnet. Fremde Anfragen (falscher Pfad oder `state`) beantwortet der Server mit 4xx, ohne
die Anmeldung abzubrechen; nach 120 s endet sie mit einer Fehlermeldung. Der Port kommt aus einer Liste: Google nimmt
einen beliebigen freien Port, Todoist verlangt registrierte Rückruf-Adressen und nutzt `42813` bis `42817` (alle fünf
in der App-Registrierung eintragen).

Tokens liegen als JSON (`Tokens` + Konto für die Anzeige) **nur im Tresor**: Dienst `nunc`, Konto `google-drive` bzw.
`todoist` (ADR 0005, nicht mehr `nunc-todoist` wie in Go; Go-Einträge werden nicht gelesen). `TokenSource` liefert ein
gültiges Access-Token, erneuert es kurz vor Ablauf oder nach einem `401` und legt das (gedrehte) Refresh-Token wieder ab.
Erneuerungen laufen nacheinander (Mutex): Dienste drehen das Refresh-Token, ein zweiter gleichzeitiger Versuch mit dem
alten schlüge fehl. Scheitert die Erneuerung, steht „Anmeldung abgelaufen – bitte neu verbinden“ im Status.

Client-ID und -Secret werden beim Bauen eingesetzt (`NUNC_GOOGLE_CLIENT_ID`, `NUNC_GOOGLE_CLIENT_SECRET`,
`NUNC_TODOIST_CLIENT_ID`, `NUNC_TODOIST_CLIENT_SECRET`, per `option_env!`); gleichnamige Umgebungsvariablen zur Laufzeit
überschreiben sie (Entwicklung). Debug-Builds lesen außerdem eine `.env` (aktueller Ordner oder darüber, auch die
Go-Namen `GOOGLE_CLIENT_ID` …); Release-Builds nie. Ohne Zugangsdaten ist „Verbinden“ deaktiviert und sagt warum. Desktop-Clients gelten
bei beiden Diensten nicht als vertraulich, gehören aber nicht ins Repository.

### Google Drive (M7-02, M7-03)

- **Scope nur `drive.file`.** Die Go-App fragte zusätzlich `userinfo.email` an, um das Konto anzuzeigen. Die Adresse
  liefert auch `about.get?fields=user(emailAddress)` mit `drive.file`; ein zweiter Scope entfällt, die Anzeige „Verbunden
  als …“ bleibt (schlägt der Aufruf fehl, ist die Verbindung trotzdem hergestellt).
- **Format und Merge wie WebDAV:** `nunc/nunc-sync.v2.json` im Drive-Stamm (CRDT-Format, ADR 0016), dieselbe
  Merge-Funktion (`engine::merge_remote`); gibt es nur die alte `nunc-sync.json`, wird sie einmal importiert. Der Ordner wird bei Bedarf angelegt, eine gelöschte Datei führt zum Hochladen des lokalen
  Stands.
- **Gleichzeitige Änderungen:** Drive kennt kein bedingtes Schreiben (WebDAV: `If-Match`). Vor dem Überschreiben liest der
  Lauf die Änderungszeit der Datei erneut und setzt (höchstens dreimal) neu an, wenn sie von der beim Laden abweicht.
  Ein Rest-Risiko zwischen Prüfung und Schreiben bleibt, wie in Go. Zwei Geräte, die gleichzeitig die **erste** Datei
  anlegen, erzeugen zwei gleichnamige Dateien; der nächste Lauf nimmt die erste und überschreibt sie.
- **Steuerung:** derselbe `SyncCoordinator` wie bei WebDAV, die Läufer stehen in einer `ChainRunner`-Kette; jeder prüft
  selbst, ob sein Anbieter gewählt und der Sync nicht pausiert ist. Jeder Anbieter hat seinen eigenen Zeitpunkt des
  letzten Syncs (`sync.webdav.last_sync_at`, `last_synced_at` – Go-Schlüssel), beim Wechsel zeigt die Statusleiste den
  des neuen Anbieters. Der Drive-Ordner steht unter `drive_folder_id` (Go-Schlüssel) für den Link „In Drive öffnen“.
- **Neues Gerät (US-S06):** Nach dem Verbinden prüft `check_sync_import`, ob dieses Gerät leer ist (keine nicht
  gelöschten Todos, keine Sessions) und Drive eine Sync-Datei hat. „Importieren“ ist ein normaler Sync, „Leer starten“
  schließt nur den Dialog – wie in Go; der Abgleich holt die Daten dann beim nächsten Auslöser. Gibt es nichts zu
  importieren, startet der erste Abgleich sofort (`finish_connect`).
- **Pausieren (US-S07):** Schalter `syncEnabled` (bestehende Einstellung); bei Pause greift kein Läufer auf Drive zu, das
  Token bleibt.

### Todoist (M7-04 bis M7-06)

**API-Version geprüft** (developer.todoist.com/api/v1, 2026-10-04): Die Go-App nutzt bereits `https://api.todoist.com/api/v1`
(nicht das ältere REST v2 der User-Story). v1 ist die aktuelle API; die Pfade `/tasks`, `/tasks/{id}` (POST = ändern),
`/tasks/{id}/close`, `/projects`, `/labels`, `/user` sind unverändert. Listen sind seitenweise (`next_cursor`); die
Antwort trägt die Einträge unter `results` (Go-Erfahrung), die Doku nennt an einer Stelle `items` – der Client nimmt
beide. Aufgaben tragen `checked`/`completed_at` statt `is_completed` und `added_at` neben `created_at`; der Client liest
beide Namen. `GET /tasks` liefert nur **aktive** Aufgaben: was dort fehlt, gilt als in Todoist erledigt oder gelöscht
(Story-Regel 11: lokal erledigt, nicht gelöscht).

**OAuth:** Autorisierung `https://app.todoist.com/oauth/authorize`, Token `https://api.todoist.com/oauth/access_token`,
Scope `data:read_write`, PKCE. Neue Apps erhalten Access-Tokens mit **einer Stunde** Lebensdauer und gedrehten
Refresh-Tokens (60 s Karenz); die Annahme der Go-App („Todoist stellt keine Refresh-Tokens aus“) trifft nicht mehr zu.
Ältere Apps haben unbefristete Tokens: ohne `expires_in` läuft das Token nie ab. Konto (E-Mail, Avatar) kommt von
`GET /api/v1/user` (`email`, `avatar_small`). Der Avatar liegt bei Todoist bzw. CloudFront; die CSP erlaubt dafür
`img-src https://*.todoist.com https://*.cloudfront.net` (fehlt der Eintrag, zeigt die Oberfläche nur den Alternativtext).

**Sync:** Merge-Regeln pur in `nunc-core::todoist_mapping` (M1). `nunc-sync::todoist` besitzt Client und Lauf, die
Datenbank sieht er nur über das Trait `TodoistStore` (keine Sperre über einen Netzwerkzugriff). Ablauf wie in Go
(Aufgaben, Projekte, Labels laden → mergen → Todoist schreiben → lokal in **einer Transaktion** schreiben), aber:

- Fehlende Projekte und Labels werden in Todoist angelegt (Namensvergleich ohne Beachtung der Großschreibung).
- Lokale Löschung schließt die Aufgabe (`close`), löscht sie nie.
- **Mappings** (`todoist_mappings`, Migration 005) entstehen beim Anlegen in Todoist und beim Import; ein lokal gelöschtes
  Todo behält sein Mapping (die Aufgabe wird einmal geschlossen, danach fehlt sie in `/tasks` und wird nicht erneut
  lokal angelegt, weil das Mapping bekannt ist).
- Nach einer Übernahme aus Todoist trägt das lokale Todo die **Änderungszeit der Aufgabe** (nicht „jetzt“): Beim
  nächsten Lauf herrscht Gleichstand und Todoist gewinnt, es gibt keinen Rück-Push derselben Daten (Go setzte „jetzt“
  und schob jede Übernahme zurück).
- Ein eigener Koordinator (Status, Ereignisse `todoist:status`/`todoist:complete`) läuft unabhängig vom Sync-Anbieter.
  Auslöser: App-Start, Todo-Änderung (entprellt), Verbinden, manuell; nicht beim Beenden. Änderungen, die Todoist lokal
  anlegt, lösen den Anbieter-Sync aus.
- Ein fehlgeschlagener Lauf (Netz, Anmeldung) steht im Status und erscheint als nicht blockierender Hinweis.

## Folgen

- Tests: Drive-Szenarien (`@integration @sync`) laufen gegen einen kleinen Google-Ersatz mit Zustand (wiremock,
  `tests/bdd/google.rs`: Token-Endpunkt, Dateien mit Versionen, Protokoll) und einen „Browser“, der den Rückruf selbst
  aufruft. Todoist entsprechend (`todoist_server.rs`).
- Manuell zu prüfen (kein Test erreicht echte Konten): Anmeldung mit echter Registrierung bei Google und Todoist,
  Rückruf-Ports 42813–42817 in der Todoist-App eintragen, Google-„Desktop“-Client anlegen.
- Offen: Drive/Todoist mobil (Backlog), Konflikt-Liste (nur Zähler), Todoist-Unteraufgaben/Fälligkeiten/Kommentare
  (Out of Scope laut Story).
