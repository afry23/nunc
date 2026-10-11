# ADR 0020 – Todoist mit persönlichem API-Token

- Status: vorgeschlagen
- Datum: 2026-10-11
- Ergänzt: ADR 0009 (Abschnitt „Todoist“, OAuth), ADR 0019 (eigener Google-Client)
- Referenz: ADR 0005 (SecretStore), `crates/nunc-sync/src/todoist/mod.rs`, `src-tauri/src/credentials.rs`

## Kontext

Todoist meldet sich heute per OAuth mit einem Client des Maintainers an (Build-Wert, GitHub-Secrets). Anders als bei
Google gilt das Todoist-Client-Secret offiziell als vertraulich, steckt aber zwangsläufig in jeder Binary. Dazu kommen
dieselben Lasten wie bei Google (ADR 0019): App-Registrierung, fünf feste Rückruf-Ports (42813–42817), Builds aus Forks
ohne Todoist.

Todoist bietet jedem Konto ein **persönliches API-Token** (Einstellungen → Integrationen → Entwickler). Es gilt für
dieselbe API v1, wird als `Authorization: Bearer …` gesendet, läuft nicht ab und kann dort jederzeit erneuert
(widerrufen) werden.

## Entscheidung

- **Persönliches Token als Anmeldeweg.** Unter „Todoist“ gibt es ein Passwortfeld „API-Token“ und „Verbinden“. Die App
  prüft das Token mit `GET /api/v1/user` (zeigt danach „Verbunden als …“ wie bisher) und legt es im Tresor ab: Dienst
  `nunc`, Konto `todoist`, im bestehenden `Tokens`-Format **ohne** `expires_in` und Refresh-Token. `TokenSource`
  behandelt das schon heute als „läuft nie ab“ (ADR 0009, ältere Apps); bei `401` gibt es nichts zu erneuern, der
  Status zeigt „Token ungültig – bitte neu eintragen“.
- **OAuth mit eingebautem Client entfällt** (gleiche Begründung wie ADR 0019). `credentials::todoist()`, die
  Build-Werte und die GitHub-Secrets `NUNC_TODOIST_CLIENT_*` werden entfernt, ebenso die feste Port-Liste für Todoist im
  Loopback. Die App-Registrierung bei Todoist wird nach dem Release gelöscht. Das OAuth-Modul bleibt für Google.
- **Anleitung in der App** (Texte in `de.ts`/`en.ts`, Link über `UrlOpener`): „Öffne in Todoist Einstellungen →
  Integrationen → Entwickler und kopiere das API-Token.“ Link: `https://app.todoist.com/app/settings/integrations/developer`.
  Hinweis: Das Token hat vollen Zugriff auf das Todoist-Konto; nunc speichert es nur im Schlüsselbund des Geräts.
- **Neues Command** `connect_todoist_token(token)` statt des OAuth-Starts; `disconnect_todoist` löscht wie bisher den
  Tresor-Eintrag. Das Token geht nie zurück ans Frontend und erscheint nie im Protokoll.
- Leerzeichen am Rand werden entfernt; ein leeres Token oder `401` bei der Prüfung ergibt `AppError`
  `todoist_token_invalid`.

## Folgen

- **Bestehende Nutzer:** Ihr OAuth-Token liegt unter demselben Tresor-Konto `todoist` und funktioniert weiter, bis es
  abläuft (neue Apps: eine Stunde) und die Erneuerung ohne Client scheitert. Dann erscheint „Bitte neu verbinden“ mit
  dem Token-Feld. Mappings (`todoist_mappings`) bleiben, der nächste Lauf setzt nahtlos fort, weil das Konto dasselbe
  ist.
- Einfacher als OAuth: kein Browser, kein Rückruf-Server, keine Ports, funktioniert grundsätzlich auch mobil (Todoist
  mobil bleibt trotzdem Backlog, bis die Oberfläche dafür steht).
- Breiterer Zugriff als OAuth-Scope `data:read_write` (das Token darf alles, z. B. auch Konto-Einstellungen). Wer das
  nicht will, kann das Token nach dem Trennen in Todoist erneuern.
- Tests: `@integration @sync` gegen den Todoist-Ersatz (`todoist_server.rs`) mit Token statt OAuth-Flow, `401` bei der
  Prüfung, `401` im Lauf. `@ui` für Eingabe, Fehlermeldung, Anleitung. OAuth-Szenarien für Todoist werden entfernt.
