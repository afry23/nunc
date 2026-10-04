# ADR 0012 – Release und Verteilung

- Status: akzeptiert (Signierung/Stores nicht ausprobiert)
- Datum: 2026-10-04

## Entscheidung

- **Ein Tag `vX.Y.Z` löst alles aus** (`.github/workflows/release.yml`). Der Job `verify` prüft vor dem Bauen, dass alle
  Versionsangaben gleich sind (`scripts/bump-version.mjs --check`) und `npm run docs:report` keine `@wip`-Szenarien
  findet. Die Version wird mit `npm run version:set X.Y.Z` an einer Stelle gesetzt.
- **Desktop über `tauri-action`**, Entwurf-Release; macOS wird mit Developer ID signiert und notarisiert, Windows und
  Linux liefern NSIS/MSI bzw. deb/rpm/AppImage. Fehlende Secrets führen zu unsignierten Artefakten statt zu einem Abbruch,
  damit Prüfbuilds auf Forks möglich bleiben.
- **Android:** Signierung in `app/build.gradle.kts` aus `keystore.properties` oder Umgebungsvariablen; AAB für die Play
  Console (interner Test-Track), APK zusätzlich als Artefakt. Fehlt der Schlüssel, bleibt das Release unsigniert.
- **iOS:** `tauri ios build --export-method app-store-connect`, Upload per `altool` mit API-Schlüssel. Keine Push-
  Entitlements, weil nur lokale Benachrichtigungen genutzt werden.
- **Kein Auto-Updater in 1.0.** Er braucht einen Update-Server und einen eigenen Signierschlüssel (Verlust = keine Updates);
  Desktop-Nutzer laden Releases von Hand, Mobil läuft über die Stores. Nachrüsten ist additiv (`docs/release.md`).
- **Secrets nie im Repository:** `.gitignore` schließt Keystores, `.p12`, `.p8`, Profile und `keystore.properties` aus.

## Folgen

- Alles Beschriebene ist bis auf die Skripte (Versions-Check, Berichte) **nicht gegen echte Konten und Zertifikate
  ausgeführt**. Der erste Release-Lauf ist der Test; `docs/release.md` hat die Checkliste.
