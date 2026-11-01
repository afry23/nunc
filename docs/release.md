# Release-Anleitung (1.0 und folgende)

Ein Release entsteht aus einem Tag `vX.Y.Z`: [`.github/workflows/release.yml`](../.github/workflows/release.yml) prüft die
Abnahme und baut macOS (arm64 + x86_64), Windows, Linux und Android. Verteilt wird **nur über GitHub Releases**: Es gibt
kein Apple-Developer-Konto, keinen App-Store- oder Play-Store-Upload und keinen iOS-Job. Alle Secrets sind optional;
ohne Apple-Zertifikat wird die macOS-App ad-hoc signiert (nicht notarisiert), ohne Android-Schlüssel bleibt die APK
unsigniert und lässt sich nicht installieren. Hinweise für Nutzer (Gatekeeper, SmartScreen) stehen in der README.

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

Die Secrets liegen im **Environment `release`** (Settings → Environments), nicht als Repository-Secrets. Dort ist ein
*Required reviewer* eingetragen: ein Release-Lauf wartet auf die Freigabe, bevor die Jobs `desktop`, `android` und `ios`
Zugriff bekommen. Zusätzlich schützt ein Tag-Ruleset für `v*` das Anlegen der Tags.

| Plattform | Secret | Inhalt |
|---|---|---|
| Android | `NUNC_ANDROID_KEYSTORE_BASE64`, `NUNC_ANDROID_KEYSTORE_PASSWORD`, `NUNC_ANDROID_KEY_ALIAS`, `NUNC_ANDROID_KEY_PASSWORD` | eigener Schlüssel, einmal selbst erzeugt und gesichert (Verlust = keine Updates mehr) |
| macOS (optional) | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` | nur mit Developer-Konto: „Developer ID Application“-Zertifikat als `.p12`, Base64 (eine Zeile) |
| macOS (optional) | `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | nur mit Developer-Konto: Notarisierung (App-spezifisches Passwort) |

**Ungültige Werte brechen den Lauf ab** (`base64: invalid input`, `SecKeychainItemImport: Unknown format`). Ein Secret
entweder korrekt setzen oder gar nicht anlegen. Wert prüfen: `echo "$WERT" | base64 --decode | openssl pkcs12 -info -noout`
bzw. `keytool -list -keystore nunc-release.jks`.

Schlüssel erzeugen:

```bash
keytool -genkeypair -v -keystore nunc-release.jks -alias nunc -keyalg RSA -keysize 4096 -validity 10000
base64 -i nunc-release.jks | tr -d '\n' | pbcopy     # → NUNC_ANDROID_KEYSTORE_BASE64
```

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
4. Play Console: nicht angebunden (der Workflow lädt nichts hoch). Das `.aab` entsteht weiter und kann bei Bedarf von Hand
   hochgeladen werden. `versionCode` leitet Tauri aus der Version ab und muss pro Upload steigen.
5. Test ohne Play Store: Der Job `android-asset` hängt die APK als `nunc_<version>_android.apk` (universal) und je ABI als
   `nunc_<version>_android_<abi>.apk` an den Release-Entwurf (nur bei Tag-Läufen). Installieren per `adb install` oder Download auf dem Gerät. Ohne Signierschlüssel ist die APK
   unsigniert und lässt sich nicht installieren.
6. Berechtigungen: `USE_EXACT_ALARM` (ab Android 13, für den Timer-Wecker) und `POST_NOTIFICATIONS` (kommt aus dem Notification-Plugin) – die Play Console
   verlangt dafür eine Begründung (Zweck: Timer-Ende als Wecker-Benachrichtigung).

## F-Droid / IzzyOnDroid (ADR 0021)

Metadaten (Texte, Icon, Screenshots, Changelogs) liegen in `fastlane/metadata/android/{de-DE,en-US}/`; IzzyOnDroid und
F-Droid lesen sie direkt aus dem Repository.

**Pro Release**

1. Changelog je Sprache anlegen: `fastlane/metadata/android/<sprache>/changelogs/<versionCode>.txt` (höchstens 500
   Zeichen, Klartext). `versionCode` = Major·1 000 000 + Minor·1 000 + Patch, z. B. 0.4.0 → `4000.txt`.
2. Den GitHub-Release **veröffentlichen** (nicht nur als Entwurf lassen); IzzyOnDroid holt die APKs je ABI
   (`nunc_<version>_android_arm64-v8a.apk` usw., der Job `android-asset` hängt sie an) von dort, meist innerhalb eines Tages.
3. Bei geänderten Funktionen `full_description.txt` mitpflegen.

**Einmalig: IzzyOnDroid**

1. Repository öffentlich, erster Release mit signierter APK veröffentlicht.
2. Signatur-Fingerprint ermitteln und hier eintragen:
   `apksigner verify --print-certs nunc_<version>_android_arm64-v8a.apk` (Zeile „SHA-256 digest“). Fingerprint: _noch offen_.
3. Aufnahme beantragen: Issue „New app“ unter <https://codeberg.org/IzzyOnDroid/repodata/issues> mit Repo-URL,
   Asset-Muster (`nunc_.*_android_(arm64-v8a|armeabi-v7a|x86|x86_64)\.apk`) und Fingerprint.
4. Screenshots: `fastlane/metadata/android/en-US/images/phoneScreenshots/1.png …` (Hochformat, z. B. vom Emulator per
   `adb exec-out screencap -p > 1.png`).

**Offizielles F-Droid** (baut selbst aus dem Quellcode, siehe ADR 0021)

1. `fdroid/fdroiddata` auf gitlab.com forken, `metadata/com.afry.nunc.yml` anlegen (Rust per `rustup`-srclib, `ndk:`,
   `tauri-cli` per `cargo install --locked` statt npm-Binary, `npm ci`, `cargo tauri android build --apk`,
   `UpdateCheckMode: Tags`).
2. Lokal prüfen: `fdroid readmeta`, `fdroid lint com.afry.nunc`, `fdroid build -v -l com.afry.nunc` (fdroidserver-Image).
3. Merge Request öffnen. Offene Punkte: native npm-Binaries im Build, reproduzierbarer Build, damit F-Droid die eigene
   Signatur übernimmt (`AllowedAPKSigningKeys`).

## iOS (M8-06, nicht im Release-Workflow)

Seit der Umstellung auf reine GitHub-Releases gibt es keinen iOS-Job mehr (braucht ein kostenpflichtiges Developer-Konto).
Die Schritte unten gelten, falls iOS später wieder aufgenommen wird; der Job muss dann in `release.yml` ergänzt werden.

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

## Release-Notes: Stopwatch (ADR 0018)

In die Notes des ersten Release mit der Stopwatch aufnehmen:

- Neuer Timer-Modus „Stoppuhr“ für die freie Zeiterfassung (keine Pause, kein Pomodoro-Zähler).
- Das Sync-Format hat jetzt `schema_version` 4. **Alle Geräte aktualisieren**: Ältere Versionen lehnen die Datei mit
  „Bitte nunc aktualisieren“ ab.
- Downgrade: Eine beim Wechsel auf eine ältere Version laufende Stoppuhr geht verloren (der gespeicherte Timer ist
  nicht lesbar); aufgezeichnete Sessions bleiben erhalten.

## Checkliste vor dem Tag

- [ ] `npm run docs:report` und alle Pflicht-Checks grün (CLAUDE.md)
- [ ] `tests/nfr/measure-desktop.sh` ([docs/nfr.md](./nfr.md)) auf dem Release-Build
- [ ] manuell: WebDAV-Sync zwischen zwei Geräten (CRDT, ADR 0016)
- [ ] Release-Notes enthalten den Abschnitt zum Sync-Format (oben), solange es der erste Release mit ADR 0016 ist
- [ ] manuell: Android- und iOS-Gerät (Timer im Hintergrund, App-Kill, Notification, Restore)
- [ ] Android-Changelogs in `fastlane/metadata/android/*/changelogs/<versionCode>.txt` (ADR 0021)
