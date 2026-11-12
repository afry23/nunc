# ADR 0038 – Jira-Import

- Status: akzeptiert
- Datum: 2026-11-12
- Referenz: ADR 0024 (nur WebDAV-Sync), ADR 0008 (WebDAV), Plan: `docs/jira-integration.md`

## Kontext

Aufgaben stehen oft als Tickets in Jira. Wer sie in nunc bearbeiten will, tippt Titel und Link bisher von Hand ab.
nunc rendert Markdown-Links in Beschreibungen bereits (`src/lib/todos.ts`, `LINK`); eine Aufgabe mit Link zum Ticket
braucht daher weder neues Schema noch neues Modell.

## Entscheidung

- Benutzer wählen ein Jira-Ticket in einem Dialog, nunc legt daraus eine normale todo.txt-Zeile an:
  `[PROJ-123](https://firma.atlassian.net/browse/PROJ-123) Login-Fehler beheben`. Sie synct per CRDT wie jede andere.
- Unterstützt sind Jira **Cloud** (E-Mail + API-Token, Basic Auth, `/rest/api/3`) und **Data Center/Server**
  (Personal Access Token, Bearer, `/rest/api/2`).
- Der Dialog zeigt zuerst „mir zugewiesen, nicht erledigt“; ein Suchfeld filtert per Text oder Ticket-Key.
- **Einmaliger Import:** kein Rück-Sync, kein Statusabgleich. Jira ist **kein** Sync-Anbieter (Abgrenzung zu ADR 0024).
- Der Zugang gilt pro Gerät und steht nicht im Sync-Dokument. Das Token liegt im Tresor (Dienst `nunc`, Konto `jira`),
  URL, Art und E-Mail in den Einstellungen (`jira.base_url`, `jira.kind`, `jira.email`). Keine Migration.
- Wörter mit `+`/`@` im Ticket-Titel werden zu todo.txt-Tags (bewusst, todo.txt-Semantik).
- Bereits importierte Tickets (offene Aufgabe mit `/browse/KEY`) werden im Dialog markiert, bleiben aber wählbar.
- Alle Plattformen (Desktop, Android, iOS).
- Neues Crate `nunc-jira` (Client, Konfiguration); Fachlogik (JQL, Zeilenformat, Parsing) pur in `nunc-core::jira`.
  Der HTTP-Client mit Systemzertifikaten zieht von `nunc-sync` nach `nunc-platform/src/http.rs` um (Firmen-Jira
  hinter TLS-Inspektion).

## Folgen

- Kein Schema-Eingriff, Backups bleiben kompatibel.
- Fehlercodes `jira_not_configured`, `jira_unauthorized`, `jira_unreachable`, `jira_bad_response`; Token nie in
  Meldungen oder Logs.
- Szenarien mit Tag `@jira`, damit der Store-Runner die Client-Szenarien nicht aufgreift.
