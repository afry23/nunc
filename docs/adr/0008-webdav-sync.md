# ADR 0008 – WebDAV-Sync (M6)

- Status: akzeptiert; Merge-Regeln und Dateiformat teilweise abgelöst durch [ADR 0016](./0016-crdt-sync.md)
- Datum: 2026-10-04
- Referenz: `nunc-go/internal/services/sync_webdav*.go`, `sync.go`, `sync_merge.go`

## Kontext

Zwei Geräte (Desktop und Handy) sollen über einen WebDAV-Server abgleichen. Die Go-App definiert das Format
(`nunc-sync.json`, `models.SyncFile`) und das Verhalten (Todos Last-Write-Wins, Sessions append-only). Beides bleibt
bindend: Go-App und nunc müssen abwechselnd gegen denselben Server arbeiten können.

## Entscheidung

**Aufteilung.** Die Merge-Regeln sind pur in `nunc-core::sync_merge` (M1). `nunc-store::sync_repo` liest den lokalen
Stand als `SyncFile` und spielt fremde Änderungen **in einer Transaktion** ein (Go: zwei). `nunc-sync` besitzt den
WebDAV-Client, den Sync-Lauf (`sync_webdav`) und die Steuerung (`SyncCoordinator`). Die Datenbank sieht `nunc-sync`
nur über das Trait `SyncStore`, damit nie eine Sperre über einen Netzwerkzugriff hinweg gehalten wird; `src-tauri`
implementiert es über `Db`.

**Client.** `reqwest` mit `PROPFIND`/`MKCOL`/`GET`/`PUT`. Der Remote-Ordner wird wie `MkdirAll` Segment für Segment
angelegt. Anmeldung Basic oder Digest (RFC 2617/7616, MD5 und SHA-256, `qop=auth`; selbst implementiert, `reqwest`
kann es nicht, die Rechenbeispiele der RFCs sind Unit-Tests). Anders als die Go-App (gowebdav verhandelt selbst) gilt die
eingestellte Methode; verlangt der Server Digest und ist Basic eingestellt, sagt die Fehlermeldung das.

**Verlorene Änderungen.** Der Upload trägt `If-Match: <ETag der geladenen Datei>`, bei einer noch nicht vorhandenen
Datei `If-None-Match: *`. Bei `412` wird bis zu dreimal neu geladen und zusammengeführt (bereits eingespielte
Änderungen sind dann idempotent und zählen nicht doppelt), danach Fehler. Server ohne ETag werden ohne Bedingung
beschrieben (Letzter gewinnt, wie in Go).

**Zählung.** `SyncResult` zählt, was **in der Datenbank ankam** (`Applied`), nicht, was der Merge als neu meldet:
abgebrochene Sessions sind lokal vorhanden, aber nicht exportierbar, und erscheinen deshalb im Merge als „neu vom
Server“; bereits gelöschte Todos legt der Sync nicht an.

**Passwort.** Im Tresor unter Dienst `nunc`, Konto `webdav` (nicht mehr `sha256(url+user)` wie in Go: die Go-Einträge
werden ohnehin nicht gelesen, ADR 0005). Die Oberfläche erfährt nur `hasPassword`; ein leeres Passwortfeld beim
Speichern behält das gespeicherte. Die übrigen Einstellungen liegen unter den Go-Schlüsseln `sync.webdav.*`.

**Steuerung.** Ein Lauf zur Zeit (asynchroner Mutex); weitere Aufrufe warten. Auslöser: App-Start, Ende oder Abbruch
einer Session, Todo-Änderung (3 s entprellt; ein bereits laufender Sync wird nie abgebrochen), Beenden (nur Desktop,
höchstens 5 s), Rückkehr in den und Wechsel in den Hintergrund (nur Mobil, das letzte über den Command
`sync_in_background`, weil Tauri kein Suspend-Ereignis liefert). Pull-to-refresh löst mobil ebenfalls einen Sync aus.
Zustand und Ergebnis gehen als `sync:start`, `sync:status`, `sync:complete` an die Oberfläche; der Zeitpunkt des
letzten Syncs steht unter `sync.webdav.last_sync_at` (Go-Schlüssel).

**Sync-Anbieter.** Genau einer (`none` · `google_drive` · `webdav`). Der Schlüssel `sync_provider` gehört
`set_sync_provider` (wirkt sofort); `save_settings` überschreibt ihn nicht, damit ein veralteter Formularentwurf die Wahl
nicht zurücksetzt. Google Drive ist bis M7 deaktiviert und mobil nie wählbar.

## Folgen

- Tests: `@sync`-Szenarien (`nunc-sync`, `cargo test -p nunc-sync --test bdd`) laufen gegen einen kleinen WebDAV-Server mit
  Zustand (wiremock, `tests/bdd/dav.rs`): Ordner, ETags, Basic/Digest, Protokoll aller Anfragen, „ein anderes Gerät war
  schneller“. Die Steuerung wird mit Echtzeit und kurzen Zeiten geprüft (pausierte Tokio-Uhr verträgt sich nicht mit
  Netzwerk-I/O im selben Runner).
- Interop: `tests/interop/webdav-go-nunc.sh` lässt die Go-App (echte `WebDAVSyncService`) und nunc abwechselnd gegen einen
  `x/net/webdav`-Server laufen und vergleicht beide Datenbanken. Docker ist nicht nötig.
- Offen: Konflikte werden nur als Zähler gemeldet (`conflictedTodos`); eine Liste der ersetzten Todos gibt es nicht.
  TLS-Zertifikatsfehler (selbstsignierte Server) lassen sich nicht übergehen.
