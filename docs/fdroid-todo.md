# F-Droid / IzzyOnDroid – offene Schritte

Stand: 2026-10-11, Branch `build/fdroid-metadata` (Commit `0ffaebb`). Entscheidung und Hintergrund:
[ADR 0021](./adr/0021-fdroid-verteilung.md), Ablauf pro Release: [release.md](./release.md#f-droid--izzyondroid-adr-0021).

Bereits erledigt: Store-Metadaten unter `fastlane/metadata/android/{de-DE,en-US}/` (Titel, Kurz- und Langbeschreibung,
Icon), Release-Workflow hängt zusätzlich eine APK je ABI an (11–16 MB statt ≈ 57 MB universal), ADR, Doku, Roadmap.

## 1. Branch abschließen

- [ ] Branch pushen, PR öffnen, nach grünen Checks mergen.

## 2. Repository öffentlich machen

- [ ] Secret-Scan über die gesamte Git-History, z. B.:
      ```bash
      brew install gitleaks
      gitleaks detect --source . --log-opts="--all"
      ```
      Bei Funden (`.env`, Keystores, Tokens, Client-Secrets): Secret widerrufen und History bereinigen
      (`git filter-repo`), **bevor** das Repository öffentlich wird.
- [ ] GitHub → `afry23/nunc` → Settings → General → Danger Zone → „Change visibility“ → Public.

## 3. Metadaten vervollständigen

- [ ] Screenshots (Hochformat) nach `fastlane/metadata/android/en-US/images/phoneScreenshots/1.png`, `2.png`, …
      Vom Emulator/Gerät: `adb exec-out screencap -p > 1.png`. Sinnvoll: Timer, Aufgaben, Statistik, Einstellungen.
      Optional deutsche Screenshots unter `de-DE/images/phoneScreenshots/`.
- [ ] Changelog je Sprache für den ersten Release:
      `fastlane/metadata/android/{de-DE,en-US}/changelogs/<versionCode>.txt`
      (`versionCode` = Major·1 000 000 + Minor·1 000 + Patch, z. B. 0.4.0 → `4000.txt`, höchstens 500 Zeichen).

## 4. Ersten Release veröffentlichen

- [ ] Android-Signier-Secrets in GitHub gesetzt (`NUNC_ANDROID_KEYSTORE_BASE64`, `…_PASSWORD`, `…_KEY_ALIAS`,
      `…_KEY_PASSWORD`), sonst sind die APKs unsigniert und IzzyOnDroid kann sie nicht nutzen.
- [ ] Release nach [release.md](./release.md) taggen; prüfen, dass der Entwurf diese Assets enthält:
      `nunc_<v>_android.apk`, `nunc_<v>_android_arm64-v8a.apk`, `…_armeabi-v7a.apk`, `…_x86.apk`, `…_x86_64.apk`.
- [ ] Release **veröffentlichen** (Entwürfe sieht IzzyOnDroid nicht).
- [ ] Signatur-Fingerprint auslesen und in `docs/release.md` eintragen:
      ```bash
      apksigner verify --print-certs nunc_<v>_android_arm64-v8a.apk   # Zeile „SHA-256 digest“
      ```
      (`apksigner` liegt in `$ANDROID_HOME/build-tools/<version>/`.)
- [ ] APK auf einem Gerät installieren und kurz prüfen (Timer im Hintergrund, Benachrichtigung, WebDAV-Sync).

## 5. Aufnahme bei IzzyOnDroid beantragen

- [ ] Issue „New app“ unter <https://codeberg.org/IzzyOnDroid/repodata/issues> (Codeberg-Konto nötig) mit:
  - Repository: `https://github.com/afry23/nunc`
  - Paketname: `com.afry.nunc`
  - APK-Muster: `nunc_.*_android_(arm64-v8a|armeabi-v7a|x86|x86_64)\.apk`
  - Signatur-Fingerprint (SHA-256)
  - Lizenz GPL-3.0, keine Anti-Features (Android synchronisiert nur über WebDAV)
- [ ] Rückfragen der Maintainer beantworten (prüfen u. a. Abhängigkeiten und Berechtigungen; `USE_EXACT_ALARM` =
      Timer-Ende als Wecker).
- [ ] Nach Aufnahme: README-Abschnitt „Installation“ um IzzyOnDroid (Badge/Link) ergänzen.

## 6. Offizielles F-Droid (danach, aufwändiger)

- [ ] Reproduzierbarer Android-Build, damit F-Droid die eigene Signatur übernehmen kann (`AllowedAPKSigningKeys`);
      sonst ist kein Update zwischen F-Droid- und GitHub-/IzzyOnDroid-APK möglich:
  - Rust-Toolchain fest (`rust-toolchain.toml` oder nur im Android-Build) und NDK-Version fest (CI und
    `ndkVersion` in `src-tauri/gen/android/app/build.gradle.kts`, statt `ANDROID_NDK_LATEST_HOME`)
  - `--remap-path-prefix` für Rust-Pfade, `SOURCE_DATE_EPOCH`
  - Prüfen: zwei Builds aus sauberem Checkout vergleichen (`apksigcopier compare` oder `diffoscope`)
- [ ] Klären, wie mit nativen npm-Binaries im Build umgegangen wird (`@tauri-apps/cli`, rolldown in Vite 8,
      `@tailwindcss/oxide`). Ansatz: `tauri-cli` per `cargo install --locked` aus Quelle; Rest mit den Reviewern klären.
- [ ] Konto auf gitlab.com, `fdroid/fdroiddata` forken, `fdroidserver` installieren (am besten Docker-Image).
- [ ] `metadata/com.afry.nunc.yml` anlegen (Skizze):
      ```yaml
      Categories: [Time]
      License: GPL-3.0-only
      SourceCode: https://github.com/afry23/nunc
      IssueTracker: https://github.com/afry23/nunc/issues
      RepoType: git
      Repo: https://github.com/afry23/nunc.git
      Builds:
        - versionName: 0.4.0
          versionCode: 4000
          commit: v0.4.0
          subdir: src-tauri/gen/android/app
          sudo: [apt-get update, apt-get install -y nodejs npm]
          srclibs: [rustup@<tag>]
          ndk: <fest gewählte NDK-Version>
          prebuild:
            - bash $$rustup$$/rustup-init.sh -y --default-toolchain <fest>
            - rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
            - cargo install tauri-cli --version <exakt> --locked
          build:
            - cd ../../.. && npm ci && cargo tauri android build --apk
          output: build/outputs/apk/universal/release/app-universal-release-unsigned.apk
          scandelete: [node_modules]
      AutoUpdateMode: Version
      UpdateCheckMode: Tags
      ```
- [ ] Lokal prüfen: `fdroid readmeta`, `fdroid lint com.afry.nunc`, `fdroid build -v -l com.afry.nunc`.
- [ ] Merge Request in `fdroiddata` öffnen, Review begleiten (dauert erfahrungsgemäß Wochen).

## Nebenbei aufgefallen

- [ ] Der Android-Release-Build warnt: `on_shutdown` (`src-tauri`) wird auf Android nicht benutzt. Laut CLAUDE.md soll
      der Build warnungsfrei sein → mit `#[cfg(desktop)]` o. Ä. beheben.
