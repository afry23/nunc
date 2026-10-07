# Release-Anleitung (1.0 und folgende)

Ein Release entsteht aus einem Tag `vX.Y.Z`: [`.github/workflows/release.yml`](../.github/workflows/release.yml) prüft die
Abnahme und baut macOS (arm64 + x86_64), Windows, Linux, Android und iOS. Alles, was Signierschlüssel oder Konten braucht,
ist **nicht im Repository** und läuft nur, wenn das jeweilige Secret gesetzt ist; ohne Secret entsteht der Teil unsigniert
oder wird mit Warnung übersprungen. Nicht geprüft werden konnte bisher (kein Zugriff auf Zertifikate, Konten, Geräte):
Signierung, Notarisierung, Play-Console- und TestFlight-Upload.

## Ablauf

```bash
git switch -c release/1.0.0
npm run version:set 1.0.0        # package.json, Cargo.toml, tauri.conf.json, gen/apple/project.yml
cargo check                      # schreibt die Version in Cargo.lock
# package-lock.json: die zwei Versionszeilen am Anfang von Hand anpassen (npm install ergänzt sonst fremde Einträge)
git commit -am "chore: release 1.0.0"
# PR → main (Check „checks“), danach:
git tag v1.0.0 && git push origin v1.0.0
```

Der Workflow legt einen **Entwurf** an (GitHub Release); Installer prüfen, dann veröffentlichen. Der Job `verify` bricht ab,
wenn Versionen auseinanderlaufen (`bump-version.mjs --check`) oder `npm run docs:report` Szenarien mit `@wip` findet.

## Secrets

| Plattform | Secret | Inhalt |
|---|---|---|
| alle | `NUNC_GOOGLE_CLIENT_ID/_SECRET`, `NUNC_TODOIST_CLIENT_ID/_SECRET` | OAuth-Zugangsdaten (werden eingebaut, siehe CLAUDE.md) |
| macOS | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD` | „Developer ID Application“-Zertifikat als `.p12`, Base64 |
| macOS | `APPLE_SIGNING_IDENTITY` | z. B. `Developer ID Application: Name (TEAMID)` |
| macOS | `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | Notarisierung (App-spezifisches Passwort) |
| Android | `NUNC_ANDROID_KEYSTORE_BASE64`, `NUNC_ANDROID_KEYSTORE_PASSWORD`, `NUNC_ANDROID_KEY_ALIAS`, `NUNC_ANDROID_KEY_PASSWORD` | Upload-Schlüssel |
| Android | `PLAY_SERVICE_ACCOUNT_JSON` | Dienstkonto für den internen Test-Track |
| iOS | `IOS_CERTIFICATE`, `IOS_CERTIFICATE_PASSWORD`, `IOS_PROVISIONING_PROFILE` | „Apple Distribution“-Zertifikat (`.p12`) und App-Store-Profil, Base64 |
| iOS | `APPLE_DEVELOPMENT_TEAM` | Team-ID für `tauri ios build` |
| iOS | `APP_STORE_CONNECT_API_KEY`, `_KEY_ID`, `_ISSUER_ID` | API-Schlüssel (`.p8`) für den TestFlight-Upload |

## Desktop (M8-04)

- **macOS:** `bundle.macOS` setzt Hardened Runtime und `minimumSystemVersion 11.0`; Tauri signiert mit
  `APPLE_SIGNING_IDENTITY` und notarisiert mit `APPLE_ID`/`APPLE_PASSWORD`/`APPLE_TEAM_ID`. Ergebnis: `.app` und `.dmg`.
  Die App ist nicht sandboxed und braucht keine Entitlements (Netzwerk-Client und Keychain gehen ohne).
- **Windows:** NSIS-Installer (pro Benutzer, Deutsch/Englisch) und MSI. Ohne Code-Signing-Zertifikat warnt SmartScreen.
- **Linux:** `.deb`, `.rpm` und AppImage.
- **Updater:** `tauri-plugin-updater` ist bewusst nicht eingebaut (braucht einen Update-Server und einen eigenen
  Signierschlüssel). Später nachrüsten: Plugin, `bundle.createUpdaterArtifacts`, öffentlicher Schlüssel in `tauri.conf.json`.
- Lokal bauen: `npm run tauri build` (Installer in `target/release/bundle/`), nur Binary: `npm run tauri build -- --no-bundle`.

## Android (M8-05)

1. Upload-Schlüssel anlegen: `keytool -genkeypair -v -keystore nunc-release.jks -alias nunc -keyalg RSA -keysize 4096 -validity 10000`
   und sicher aufbewahren (Verlust = keine Updates mehr; mit „Play App Signing“ kann Google den App-Schlüssel verwalten).
2. Lokal: `src-tauri/gen/android/keystore.properties` mit `storeFile`, `storePassword`, `keyAlias`, `keyPassword`
   (ist in `.gitignore`). CI: die Secrets oben.
3. `npm run tauri android build -- --aab --apk` → `app/build/outputs/bundle/universalRelease/*.aab`.
4. Play Console: App `com.afry.nunc` anlegen, internen Test-Track mit Testern, erstes Bundle von Hand hochladen
   (danach kann der Workflow hochladen). `versionCode` leitet Tauri aus der Version ab und muss pro Upload steigen.
5. Test ohne Play Store: Der Job `android-asset` hängt die APK als `nunc_<version>_android.apk` an den Release-Entwurf
   (nur bei Tag-Läufen). Installieren per `adb install` oder Download auf dem Gerät. Ohne Signierschlüssel ist die APK
   unsigniert und lässt sich nicht installieren.
6. Berechtigungen: `USE_EXACT_ALARM` (ab Android 13, für den Timer-Wecker) und `POST_NOTIFICATIONS` (kommt aus dem Notification-Plugin) – die Play Console
   verlangt dafür eine Begründung (Zweck: Timer-Ende als Wecker-Benachrichtigung).

## iOS (M8-06)

1. Im Apple Developer Portal: App-ID `com.afry.nunc`, „Apple Distribution“-Zertifikat, App-Store-Provisioning-Profil;
   in App Store Connect die App anlegen.
2. `APPLE_DEVELOPMENT_TEAM=<TEAMID> npm run tauri ios build -- --export-method app-store-connect` → `.ipa` in
   `src-tauri/gen/apple/build/arm64/`.
3. Upload: `xcrun altool --upload-app --type ios --file nunc.ipa --apiKey <KEY_ID> --apiIssuer <ISSUER_ID>` (oder Transporter).
4. **Entitlements:** lokale Benachrichtigungen (Timer-Ende) brauchen **keine** Push-Entitlements, `aps-environment` bleibt
   weg. `nunc_iOS.entitlements` ist deshalb leer. Die Berechtigung fragt die App zur Laufzeit an.
5. Bekannt: `tauri-plugin-notification` plant auf iOS über einen Zeitzonen-Offset-Workaround (ADR 0004); vor dem Release
   auf einem echten Gerät prüfen, dass die Benachrichtigung pünktlich kommt.

## Release-Notes: Sync-Format (0.3.0, ADR 0016)

In die Notes des ersten Release mit CRDT-Sync aufnehmen:

- Der Sync verwendet die neue Datei `nunc-sync.v2.json`; Todos werden feldweise zusammengeführt (gleichzeitige
  Änderungen verschiedener Felder gehen nicht mehr verloren).
- **Alle Geräte auf dieselbe Version bringen.** Ältere Versionen kennen die v2-Datei nicht und würden die alte
  `nunc-sync.json` weiter beschreiben, ohne dass der Stand ankommt. Die alte Datei bleibt unangetastet liegen und
  wird beim ersten Sync einmal importiert.
- Die Go-App versteht das neue Format nicht (siehe [migration-from-go.md](./migration-from-go.md)).
- Gelöschte Aufgaben verschwinden nach 90 Tagen aus der Sync-Datei; ein Gerät, das länger offline war, legt sie beim
  nächsten Sync ggf. wieder an.

## Checkliste vor dem Tag

- [ ] `npm run docs:report` und alle Pflicht-Checks grün (CLAUDE.md)
- [ ] `tests/nfr/measure-desktop.sh` ([docs/nfr.md](./nfr.md)) auf dem Release-Build
- [ ] manuell: Anmeldung Google/Todoist mit echten Konten; Sync zwischen zwei Geräten (CRDT, ADR 0016)
- [ ] Release-Notes enthalten den Abschnitt zum Sync-Format (oben), solange es der erste Release mit ADR 0016 ist
- [ ] manuell: Android- und iOS-Gerät (Timer im Hintergrund, App-Kill, Notification, Restore)
