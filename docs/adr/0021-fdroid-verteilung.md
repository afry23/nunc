# ADR 0021 – Android-Verteilung über IzzyOnDroid und F-Droid

- Status: akzeptiert
- Datum: 2026-10-11
- Ergänzt: ADR 0012 (Release)
- Referenz: [docs/release.md](../release.md), `fastlane/metadata/android/`, `.github/workflows/release.yml`

## Kontext

nunc soll auf Android auch ohne Play Store installierbar sein. Der Release-Workflow baut bereits eine signierte APK und
hängt sie an den GitHub-Release (`android-asset`). Für F-Droid-Clients gibt es zwei Wege:

- **IzzyOnDroid**: F-Droid-kompatibles Repository, übernimmt die vom Entwickler signierte APK aus GitHub Releases. Prüft
  Lizenz, Abhängigkeiten (Tracker, proprietäre Bibliotheken) und Größe, baut aber nicht selbst. Aufnahme in Tagen.
- **Offizielles F-Droid**: baut jede Version selbst aus dem Quellcode (Rezept in `fdroiddata`) und signiert mit eigenem
  Schlüssel, außer der Build ist reproduzierbar. Strenge Regeln für vorgefertigte Binärdateien im Build; npm-Pakete mit
  nativen Binaries (`@tauri-apps/cli`, rolldown in Vite 8, `@tailwindcss/oxide`) sind dort Diskussionsstoff. Aufnahme in
  Wochen.

Voraussetzungen für beide: öffentlicher Quellcode, freie Lizenz (GPLv3 ✓), keine proprietären Abhängigkeiten (keine
Play-Services/Firebase ✓; `androidx`, `material`, `android-native-keyring-store` sind frei).

## Entscheidung

- **Zuerst IzzyOnDroid**, das offizielle F-Droid parallel. Das Repository wird öffentlich.
- **Metadaten im Repository** unter `fastlane/metadata/android/{de-DE,en-US}/` (Titel, Kurz- und Langbeschreibung, Icon,
  Screenshots, `changelogs/<versionCode>.txt`). Beide Repositories lesen diesen Pfad; es gibt nur eine Quelle.
- **Eine APK je ABI.** Die Universal-APK ist ≈ 57 MB groß (vier native Bibliotheken), IzzyOnDroid nimmt höchstens
  30 MB. Der Release hängt deshalb zusätzlich `nunc_<version>_android_<abi>.apk` an (`arm64-v8a`, `armeabi-v7a`, `x86`,
  `x86_64`, je 11–16 MB); `nunc_<version>_android.apk` (universal) bleibt für die manuelle Installation.
- **Asset-Namen bleiben stabil**, IzzyOnDroid findet die APKs per Muster. Der GitHub-Release muss veröffentlicht sein
  (Entwürfe sieht IzzyOnDroid nicht).
- **Signatur**: ein Schlüssel für GitHub, IzzyOnDroid (und Play-Upload). Der SHA-256-Fingerprint steht in
  [docs/release.md](../release.md), damit Nutzer ihn prüfen können. Für das offizielle F-Droid ist das Ziel ein
  reproduzierbarer Build, damit F-Droid die Entwickler-Signatur übernimmt (`AllowedAPKSigningKeys`); sonst können Nutzer
  zwischen F-Droid- und GitHub-/IzzyOnDroid-APK nicht aktualisieren.
- **Anti-Features**: keine. Die Android-App synchronisiert nur über WebDAV (Google Drive und Todoist sind Desktop).
  Kommen Google Drive oder Todoist auf Android, ist `NonFreeNet` in beiden Repositories zu melden.

## Folgen

- Pro Release eine Changelog-Datei je Sprache (`versionCode` = Major·1 000 000 + Minor·1 000 + Patch, wie Tauri ihn bildet).
- Offen für das offizielle F-Droid: reproduzierbarer Android-Build (Rust-Toolchain und NDK fest, `--remap-path-prefix`,
  `SOURCE_DATE_EPOCH`) und ein Build-Rezept, das ohne native npm-Binaries auskommt oder mit den Reviewern geklärt ist.
- Store-Texte sind Teil des Repositorys und werden bei Funktionsänderungen mitgepflegt (z. B. Sync-Anbieter auf Android).
