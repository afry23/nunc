# Jira-Integration: Ticket → Aufgabe

Status: geplant · Datum: 2026-11-12 · ADR: 0038 (anzulegen)

## Kontext

Benutzer wählen ein Jira-Ticket aus, nunc legt daraus eine Aufgabe an: Titel wird übernommen, ein Link führt zum
Original-Ticket. nunc rendert Markdown-Links in Beschreibungen bereits (`src/lib/todos.ts`, `LINK`), deshalb ist
**kein Schema- oder Modell-Eingriff** nötig: die Aufgabe ist eine normale todo.txt-Zeile und synct per CRDT wie jede
andere.

### Entscheidungen

- Jira **Cloud** (E-Mail + API-Token, Basic Auth, `/rest/api/3`) **und Data Center/Server** (Personal Access Token,
  Bearer, `/rest/api/2`).
- Auswahl: Dialog zeigt zuerst „mir zugewiesen, nicht erledigt“; ein Suchfeld filtert per Text oder Ticket-Key
  (`PROJ-123`).
- Erzeugte Zeile: `[PROJ-123](https://firma.atlassian.net/browse/PROJ-123) Login-Fehler beheben`
- Alle Plattformen (Desktop, Android, iOS).
- Einmaliger Import: kein Rück-Sync, kein Statusabgleich mit Jira. Jira ist **kein** Sync-Anbieter (Abgrenzung zu
  ADR 0024).
- Zugang gilt pro Gerät (nicht im Sync-Dokument); Token im Tresor (Dienst `nunc`, Konto `jira`).
- Wörter mit `+`/`@` im Ticket-Titel werden zu todo.txt-Tags (bewusst so belassen, todo.txt-Semantik).

## Umsetzung (BDD, innen → außen), Branch `feat/jira-import`

### 0. ADR und Features

- `docs/adr/0038-jira-import.md` mit den Entscheidungen oben.
- Features (Story-Tags `@US-J01` …, zunächst `@wip`):
  - `features/jira/jira_core.feature` `@domain`: JQL-Bau, Escaping, URL, Zeilenformat, Duplikaterkennung,
    Antwort-Parsing.
  - `features/jira/jira_client.feature` `@integration`: wiremock – Cloud Basic, DC Bearer, 401/403, 5xx, kaputtes
    JSON, Verbindungstest.
  - `features/ui/jira.feature` `@ui`: Einstellungen speichern/testen, Picker öffnen, suchen, Ticket wählen → Aufgabe
    + Toast, „bereits vorhanden“.

### 1. `nunc-core/src/jira.rs` (pur, Schwerpunkt Mutation-Score)

- Typen (specta, camelCase): `JiraKind { Cloud, DataCenter }` (`text_enum!` wie `SyncProvider`),
  `JiraConfig { baseUrl, kind, email }` mit `is_complete()`, `JiraIssue { key, summary, url }`.
- `search_jql(query: &str) -> String`:
  - leer → `assignee = currentUser() AND statusCategory != Done ORDER BY updated DESC`
  - Key-Muster `^[A-Z][A-Z0-9_]+-\d+$` (Groß-/Kleinschreibung egal, normalisiert groß) → `key = "PROJ-123"`
  - sonst → `text ~ "<escaped>" AND statusCategory != Done ORDER BY updated DESC`
- `escape_jql(s)`: `\` und `"` escapen (Proptest: Ergebnis enthält kein unescaptes `"`).
- `issue_url(base, key)`: abschließenden Slash entfernen, `/browse/KEY` anhängen.
- `todo_line(issue)`: `[KEY](url) summary`; Zeilenumbrüche/Tabs → Leerzeichen, trimmen; leerer Titel → nur Link.
- `parse_search(json, base) -> Result<Vec<JiraIssue>, JiraParseError>`: `{ issues: [{ key, fields: { summary } }] }`
  (gleich für v2 und v3), höchstens 50 Treffer.
- `is_imported(todos, issue) -> bool`: offene Todos, deren Text `/browse/KEY` (mit Wortgrenze) enthält.
- `api_path(kind)`: Suche `/rest/api/3/search/jql` bzw. `/rest/api/2/search`; `myself` analog.

### 2. Gemeinsamer HTTP-Client

- `nunc-sync/src/http.rs` → `nunc-platform/src/http.rs` (`http_client() -> Result<reqwest::Client, String>`).
  Systemzertifikate bleiben – wichtig für Firmen-Jira hinter Zscaler. `nunc-sync` ruft ihn auf und mappt auf
  `SyncError::Transport`; reqwest/rustls-Abhängigkeiten wandern mit.

### 3. Neues Crate `crates/nunc-jira`

- Abhängigkeiten: nunc-core, nunc-store, nunc-platform, reqwest, serde_json, thiserror, log; dev: cucumber, wiremock,
  tokio.
- `config.rs` (Muster `nunc-sync/src/config.rs`): `save_jira(conn, secrets, cfg, token: Option)` (Tresor zuerst),
  `load_jira_credentials` → `NotConfigured`, `has_jira_token`, `clear_jira` (Einstellungen + Tresor). `Debug` ohne
  Token.
- Persistenz: Settings-Schlüssel `jira.base_url`, `jira.kind`, `jira.email` in `nunc-store` (neue `jira_repo.rs`,
  Muster `sync_repo::load_webdav_config`). Key-Value-Tabelle → **keine Migration**.
- `client.rs`: `JiraClient::new(creds)`; Cloud `.basic_auth(email, token)`, DC `.bearer_auth(token)`;
  `search(query)` (GET mit `jql`, `fields=summary`, `maxResults=50`), `test()` (GET `myself`).
- `JiraError` (thiserror): `NotConfigured`, `Unauthorized` (401/403), `Http(status)`, `Transport`, `BadResponse`,
  `Store`, `Secret`. Meldungen ohne Token oder Header.
- `tests/bdd/` mit eigener World (Muster `nunc-sync/tests/bdd`).

### 4. Commands `src-tauri/src/commands/jira.rs` (dünn)

- `get_jira_config` → `{ config, hasToken }`
- `save_jira_config(config, token: Option<String>)` (URL mit `is_http_url` prüfen)
- `clear_jira_config`
- `test_jira_connection(config, token: Option)` (ohne Token: gespeicherter Token)
- `search_jira_issues(query)` → `Vec<JiraIssueHit { issue, imported }>`
- `add_todo_from_jira(issue)` → `todo_line` + bestehender `nunc_store::add_todo`-Pfad wie `commands::todos::add_todo`,
  danach `state.todos_changed()`.
- `From<JiraError> for AppError` in `src-tauri/src/error.rs`: Codes `jira_not_configured`, `jira_unauthorized`,
  `jira_unreachable`, `jira_bad_response`; `error.<code>` in `src/i18n/de.ts` und `en.ts`.
- Registrieren in `specta_builder()` (`src-tauri/src/lib.rs`), `cargo test -p nunc export_bindings`,
  `src/bindings.ts` committen.
- Command-Tests in `commands/tests.rs` (`mock_builder`, Fake-SecretStore).

### 5. Frontend

- `src/lib/api.ts`: Wrapper für die sechs Commands.
- `src/components/JiraSection.tsx` in `SettingsView` (Muster `WebDavForm.tsx`/`SyncSection.tsx`): Typ (Cloud/DC),
  URL, E-Mail (nur Cloud), Token (Passwortfeld, Hinweis „gespeichert“), Testen, Speichern, Entfernen.
- `src/components/JiraPickerDialog.tsx`: Suchfeld (Debounce 300 ms), Liste `KEY · Titel`; „bereits vorhanden“
  ausgegraut, aber mit Hinweis wählbar; Klick → `addTodoFromJira`, Toast `todos.created`, Todo-Store neu laden;
  Fehler als AppError-Text, „nicht eingerichtet“ → Link zu den Einstellungen.
- Einstieg: Knopf „Aus Jira“ in `NewTaskDialog.tsx` (nur wenn eingerichtet), `uiStore.jiraPickerOpen`.
- Texte in `src/i18n/de.ts`/`en.ts`. Mobil: Dialog vollflächig, bei ≈ 411 px auf horizontalen Überlauf prüfen.
- Kurzer Abschnitt in `HelpView`.

### 6. Doku

- `roadmap.md`: Eintrag unter „Erweiterungen nach M8“.
- `PLAN.md` und CLAUDE.md-Layout um `nunc-jira` ergänzen; Stolperstein: Jira-Token im Tresor (`nunc`/`jira`).

## Wiederverwenden

- `nunc-sync/src/config.rs` (Tresor-/Settings-Muster), `nunc-store::sync_repo::load_webdav_config`, `is_http_url`
- `commands/sync.rs` (`test_webdav_connection`), `commands/todos.rs::add_todo`
- `WebDavForm.tsx`, `NewTaskDialog.tsx`, `src/lib/todos.ts` (Link-Rendering, keine Änderung nötig)

## Roadmap

| ID | Ticket |
|---|---|
| J-01 | ADR 0038 und `@wip`-Szenarien `features/jira/jira_core.feature`, `jira_client.feature`, `features/ui/jira.feature` |
| J-02 | `nunc-core::jira`: Typen, `search_jql`, `escape_jql`, `issue_url`, `todo_line`, `parse_search`, `is_imported`, `api_path` (`@domain`, Proptest für Escaping) |
| J-03 | HTTP-Client nach `nunc-platform/src/http.rs` verschieben, `nunc-sync` umstellen (reines Refactoring, Tests bleiben grün) |
| J-04 | `nunc-store::jira_repo` (Settings-Schlüssel) und Crate `nunc-jira`: `config.rs`, `client.rs`, `JiraError` (`@integration` mit wiremock: Cloud Basic, DC Bearer, 401/403, 5xx, kaputtes JSON) |
| J-05 | Commands `get/save/clear_jira_config`, `test_jira_connection`, `search_jira_issues`, `add_todo_from_jira`; `AppError`-Codes in beiden Katalogen; Bindings |
| J-06 | UI Einstellungen: `JiraSection` (`@ui`) |
| J-07 | UI Picker: `JiraPickerDialog`, Knopf „Aus Jira“ in `NewTaskDialog`, Hilfe-Abschnitt (`@ui`). **Manuell prüfen:** Dialog bei ≈ 411 px ohne horizontalen Überlauf |
| J-08 | Doku (`roadmap.md`, `PLAN.md`, CLAUDE.md), `@wip` entfernen, `docs:report` grün, `cargo mutants --in-diff` für `nunc-core`/`nunc-store`/`nunc-jira` |

## Verifikation

- `cargo test -p nunc-core --test bdd`, `cargo test -p nunc-jira --test bdd`, `npm run test:bdd`.
- Alle Pflicht-Checks aus CLAUDE.md; `cargo mutants --in-diff` für `nunc-core`, `nunc-store`, `nunc-jira`.
- Manuell: `npm run tauri dev`, Cloud-Instanz mit API-Token einrichten → Testen → Picker zeigt eigene offene Tickets;
  Suche nach Text und Key → Aufgabe erscheint als `PROJ-123`-Link + Titel, Link öffnet den Browser (opener). Gleiches
  Ticket erneut → „bereits vorhanden“. Falscher Token → Meldung `jira_unauthorized`, kein Token in `nunc.log`.
- Data Center mit PAT (falls Instanz verfügbar), sonst nur wiremock.
- Android-Emulator: Einstellungen und Picker bei 411 px ohne horizontalen Überlauf.
