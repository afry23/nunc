# Nicht-funktionale Anforderungen – Messung (M8-01)

| Anforderung | Ziel | Wie geprüft | Ergebnis |
|---|---|---|---|
| Start | < 2 s | `tests/nfr/measure-desktop.sh` (Release-Build, leere Daten): Protokollzeile „Oberfläche geladen nach N ms“ | 2026-10-04, macOS 26 (Apple Silicon): Einrichtung 200–350 ms, Oberfläche geladen 400–510 ms |
| Arbeitsspeicher (Desktop) | < 100 MB | dasselbe Skript, Summe aus App- und WebView-Prozessen (`footprint`/PSS, nicht RSS) | 88–93 MB im Leerlauf (App ≈ 27 MB, WebKit-Prozesse der Rest) |
| Timer-Drift | < 1 s/h | Szenario „Irregular and missing ticks do not add up to a drift over an hour“ (`@domain`): Zeit ist ein Zeitstempel, kein Zähler (ADR 0004) | 0 s, konstruktionsbedingt |
| Kein Datenverlust bei Kill | Timer-Zustand bleibt | Szenario „A running timer survives the process being killed without a clean shutdown“ (`@integration`: Verbindung ohne Schließen verworfen, WAL bleibt liegen) und Maestro-Flow `kill-resume` (Mobil) | grün; Maestro nicht ausgeführt |

**Einschränkungen.** Der RAM-Wert gilt für eine leere Datenbank und ruhende App; mit sehr vielen Aufgaben oder offener
Statistik-Ansicht steigt er. Das Skript misst auf macOS und Linux; Windows ist nicht gemessen. Der Wert liegt knapp unter dem
Ziel (WebKit ist der größte Posten), eine Regression von ≈ 7 MB würde es reißen: nach größeren Frontend-Änderungen neu messen.
Mobil sind Start und Speicher nicht gemessen.

Wiederholen:

```bash
npm run tauri build -- --no-bundle
tests/nfr/measure-desktop.sh            # Binary und Runden optional: <Binary> <Runden>
```
