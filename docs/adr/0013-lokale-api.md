# ADR 0013 – Lokale HTTP-Schnittstelle

- Status: akzeptiert
- Datum: 2026-10-04

## Kontext

Eine andere App auf demselben Desktop (Skript, Launcher, Editor-Plugin, anderes todo.txt-Werkzeug) soll Aufgaben im
todo.txt-Format mit nunc austauschen und ihren Status abfragen können. Bisher ging das nur über die Dateidialoge
(Import/Export) oder den Sync über WebDAV, Google Drive und Todoist.

## Entscheidung

**Ein kleiner HTTP-Server auf `127.0.0.1`**, nur Desktop, standardmäßig **aus** (Einstellungen → Lokale Schnittstelle,
Schlüssel `api_enabled`, `api_port`, nur nunc; die Go-App ignoriert sie). Mobil gibt es ihn nicht (`cfg(desktop)`,
Commands melden `unsupported`).

- **Crate `nunc-api`:** `route(request, config, backend)` ist **pur** (Zugriffsschutz, Routing, Formate, Statuscodes), die
  Todos kommen über das Trait `TodoBackend`. `server` ist der dünne Rand (hyper 1, eine Anfrage je Verbindung, Zeitgrenzen,
  höchstens 32 Verbindungen). In der App adaptiert `AppBackend` (`src-tauri/src/api_service.rs`) das Trait auf Store und Sync.
- **Dienst:** `ApiService` startet/stoppt den Server gemäß `save_settings` (idempotent), beim Start wenn eingeschaltet.
  Ist der Port belegt, wird der nächste freie (bis +9) genommen; die Oberfläche zeigt den tatsächlichen Port.

### Schnittstelle (v1)

| Anfrage | Wirkung |
|---|---|
| `GET /v1/health` | `{ app, version, apiVersion }` |
| `GET /v1/todos` | todo.txt (`text/plain`); mit `Accept: application/json` die Todos samt Status (`completed`, `completedAt`, `pomodoroCount`, Tags, `position`). Filter: `completed=true\|false`, `project=`, `context=`, `priority=`, `q=` |
| `GET /v1/todos/{id}` | Status einer Aufgabe |
| `POST /v1/todos` | Body `text/plain` (mehrere Zeilen) oder `{"rawText": "…"}`: legt nur neue Zeilen an. `201` wenn etwas angelegt wurde, sonst `200`; `{ added, duplicates, todos }` |
| `PATCH /v1/todos/{id}` | `{"completed": true\|false}` |

Fehler sind `{ code, message }` auf Englisch (`unauthorized`, `forbidden_host`, `forbidden_origin`, `invalid_input`,
`not_found`, `method_not_allowed`, `payload_too_large`, `internal`). Änderungen laden die Oberfläche neu (`app:reload`) und
stoßen den Sync an, wie Änderungen in der App.

### Bedrohungsmodell

Angreifer: ein Webseitenskript im Browser des Nutzers, ein anderer Benutzer oder ein fremdes Programm auf demselben Rechner.

- **Nur Loopback**, nie `0.0.0.0`: von außen nicht erreichbar.
- **Bearer-Token** (256 Bit, URL-sicheres Base64) im Schlüsselspeicher (Dienst `nunc`, Konto `local-api`), Vergleich in
  konstanter Zeit, nach „Token anzeigen“ in den Einstellungen sichtbar, „Neu erzeugen“ macht das alte ungültig und startet
  den Server neu. Das Token steht in keiner Datenbank, keinem Log und keiner Fehlermeldung.
- **Browser:** Anfragen mit `Origin`-Header werden abgelehnt (Browser senden ihn bei Aufrufen aus Webseiten, Programme nicht),
  es gibt kein CORS. Der `Host`-Header muss `127.0.0.1:<Port>` oder `localhost:<Port>` sein (DNS-Rebinding).
- **Last:** Körper höchstens 5 MB (`413`, die angekündigte Länge wird vorab geprüft), Header-Zeitgrenze 10 s, Verbindung 30 s.
- **Grenzen:** Wer das Token kennt oder in den Schlüsselspeicher des Nutzers schaut, hat vollen Zugriff auf die Aufgaben
  (nicht auf Sessions, Einstellungen, Backups oder Konten). Der Verkehr ist unverschlüsselt (Loopback).

### Verworfene Alternativen

- **Gemeinsame todo.txt-Datei** (Dateisync): ohne Status und Token, aber mit Konflikten und ohne Rückmeldung; als spätere
  Ergänzung denkbar.
- **URL-Schema/Intents:** für Mobil sinnvoll, am Desktop ohne Rückkanal für den Status.
- **Eigenbau-HTTP-Server** wie beim OAuth-Loopback: dort genügt eine Anfrage, hier brauchen Keep-Alive/Body-Grenzen/Header
  eine geprüfte Bibliothek (hyper steckt ohnehin über `reqwest` im Build).

## Folgen

- Linux ohne Secret-Service: Das Token lässt sich nicht speichern, der Server startet nicht; die Einstellungen zeigen den Grund.
- Später: `GET /v1/timer`, Statistik, Benachrichtigung bei Änderungen (SSE/Webhook), Mobil über Intents/Kurzbefehle.
