# ADR 0029 – Kompakte Sync-Datei (Format 9)

- Status: angenommen
- Datum: 2026-11-03
- Ergänzt: ADR 0016, 0017

## Kontext

`nunc-sync.v2.json` ist eine einzige Datei, die jeder Lauf komplett lädt und hochlädt. Sie wächst unbegrenzt (nur
gelöschte Einträge räumt die GC ab). Messung (`crates/nunc-core/tests/sync_size.rs`, synthetisch): ca. 3 MB pro Jahr bei
moderater, ca. 10 MB pro Jahr bei starker Nutzung; Parse und Merge dauern auch bei 50 MB unter 150 ms. Der Engpass ist
die Übertragung, nicht die CPU.

## Entscheidung

- **Schema 9.** Dateiname und Merge-Regeln bleiben; Format 2 bis 8 werden weiter gelesen.
- **HLC-Stempel als Tupel** `[wall_ms, counter, device_id]` statt Objekt (`Hlc` hat eigenes Serde, liest beides).
- **Kompaktes JSON, gzip-komprimiert** (`SyncDocument::encode`/`decode` in `nunc-core`, `flate2` mit Rust-Backend, auch
  für Mobil). `decode` erkennt gzip an den Magic Bytes und liest sonst Klartext-JSON; entpackt höchstens 512 MiB.
- Upload mit `Content-Type: application/gzip`; Download/ETag/`If-Match` unverändert.

## Folgen

- Messung synthetisch: Datei 3,2 → 2,4 MB (nur Stempel), mit gzip < 0,1 MB. Echte Daten sind weniger gleichförmig;
  erwartet wird trotzdem ein Vielfaches kleiner.
- Ältere nunc-Versionen können die Datei nicht mehr lesen (Parsefehler statt Formatmeldung) – Hinweis in den
  Release-Notes. Die Datei ist nicht mehr von Hand lesbar (`gunzip -c`).
- Offen (nicht Teil dieser ADR): bedingter Download per ETag und Partitionierung nach Jahr, falls nötig.
