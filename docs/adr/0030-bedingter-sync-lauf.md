# ADR 0030 – Bedingter Sync-Lauf

- Status: angenommen
- Datum: 2026-11-03
- Ergänzt: ADR 0008, 0014, 0029

## Kontext

Auch mit gzip (ADR 0029) lädt und schreibt jeder Lauf die Sync-Datei komplett, selbst wenn sich seit dem letzten
Lauf nichts geändert hat (Takt bis 5 Minuten, ADR 0014). Der Leerlauf ist der Regelfall.

## Entscheidung

`sync_webdav_cached` merkt sich zwischen zwei Läufen (`SyncCache`, nur im Speicher, im `WebDavRunner`):
**Adresse** der Sync-Datei, deren **ETag** und einen **Fingerabdruck des lokalen Stands** (Hash des
`SyncState` ohne abgelaufene Tombstones), mit dem beide übereinstimmten.

- **Abkürzung:** Ist der lokale Stand unverändert (gleicher Fingerabdruck) und dieselbe Adresse im Spiel, fragt der Lauf
  mit `If-None-Match: <etag>`. Antwortet der Server mit `304`, ist nichts zu tun: kein Download, kein Merge, kein Upload.
- **Sonst unverändert:** Jede lokale Änderung, ein anderer Server/Pfad, ein leerer Cache (Neustart) oder ein Server, der
  die Bedingung ignoriert (`200`), führt den gewohnten vollen Lauf aus.
- **Kein Upload ohne Änderung:** Ist die geladene Datei schon im aktuellen Format und entspricht der zusammengeführte Stand
  genau dem Server-Stand, entfällt der Upload; der ETag des Downloads bleibt gültig.
- **ETag nach dem Upload:** Nennt der Server ihn in der `PUT`-Antwort, wird er gemerkt. Sonst lädt der nächste Lauf die
  Datei einmal ganz (und überspringt dann den Upload); ab dem übernächsten Lauf greift die Abkürzung.

## Sicherheit

Die Abkürzung gilt **nur bei unverändertem lokalem Stand**. Bei lokalen Änderungen oder nach einem Restore wird die Datei
immer ganz geladen und zusammengeführt. Nur so kann ein älterer lokaler Stand die Server-Datei nie überschreiben. Ein
falscher oder veralteter ETag führt höchstens zu einem vollen Lauf, nie zu Datenverlust; der Upload bleibt durch
`If-Match` geschützt.

## Folgen

- Der Leerlauf kostet eine `PROPFIND`- und eine `GET`-Anfrage ohne Datei-Inhalt und liest die lokale Datenbank einmal.
- Der Fingerabdruck serialisiert den lokalen Stand (Millisekunden, auch bei 50 MB Sync-Datei) und kostet keine
  Netzwerkzeit.
- Offen: Partitionierung nach Jahr, falls die Datei trotz gzip zu groß wird.
