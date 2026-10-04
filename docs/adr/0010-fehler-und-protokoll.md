# ADR 0010 – Fehlerbehandlung und Protokoll

- Status: akzeptiert
- Datum: 2026-10-04

## Kontext

Seit M3 gehen alle Fehler als `AppError { code, message }` zur Oberfläche (`thiserror` im Kern, `From`-Umwandlungen in
`src-tauri/src/error.rs`). Die Aufrufe von `log::…` im Backend hatten aber keinen Empfänger, Fehler auf Geräten von
Nutzern waren nicht nachvollziehbar.

## Entscheidung

- **Ein Fehlertyp** bleibt: stabiler `code` (für Logik und Übersetzung in der UI), `message` ist der deutsche
  Standardtext. Die Oberfläche übersetzt über den `code` (ADR 0011) und fällt sonst auf die `message` zurück.
- **Protokoll mit `tauri-plugin-log`**: Datei `nunc.log` im Log-Ordner des Systems (`app_log_dir`, macOS
  `~/Library/Logs/com.afry.nunc`), ab 1 MB rotiert, fünf Dateien bleiben liegen. Release ab `Info`, Debug ab `Debug`
  (zusätzlich Stdout). HTTP-Bibliotheken nur ab `Warn`.
- **Jeder `AppError` wird beim Erzeugen protokolliert** (`AppError::new`): Eingabefehler auf `debug`, Datenbank-/E/A-/
  interne Fehler auf `error`, alles andere auf `warn`. So muss kein Command das einzeln tun.
- Secrets gehören weiter nie in Meldungen: Fehlertexte der Sync-Clients enthalten Status und Aktion, keine Header oder Bodys.

## Folgen

- Nutzer können `nunc.log` an Fehlerberichte anhängen (Pfad steht im README).
- Frontend-Fehler (`window.onerror`) werden nicht ins Protokoll weitergeleitet; die UI zeigt sie als Hinweis.
