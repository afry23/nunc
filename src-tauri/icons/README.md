# App-Icon

Alle Icons werden aus SVG-Quellen erzeugt, nie von Hand bearbeitet.

| Datei | Zweck |
|---|---|
| `source.svg` | Hauptquelle, randlos (iOS, Windows, Linux, Android-Legacy) |
| `source-macos.svg` | macOS-Variante nach Apple-Raster (824 px abgerundetes Quadrat, 100 px Rand), nur für `icon.icns` |
| `source-android-fg.svg` / `source-android-bg.svg` | Android-Adaptive-Icon: Vordergrund (transparent) und Hintergrund |
| `icon-manifest.json` | Manifest für `tauri icon` (Hintergrundfarbe, Android-Ebenen) |

Motiv: Lucide „hourglass“ (ISC-Lizenz), Farben aus den Tokens in `src/styles.css`.

## Neu erzeugen

```bash
npm run tauri icon src-tauri/icons/icon-manifest.json
npm run tauri icon src-tauri/icons/source-macos.svg -- -o /tmp/nunc-icon-mac
cp /tmp/nunc-icon-mac/icon.icns src-tauri/icons/icon.icns
```

Der erste Lauf überschreibt auch `src-tauri/gen/android` und `src-tauri/gen/apple`.
Danach `values/ic_launcher_background.xml` prüfen (soll `#131313` sein, nicht `#FFFFFF`).
