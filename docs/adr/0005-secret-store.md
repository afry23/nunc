# ADR 0005 – SecretStore (Spike M0-10)

- Status: akzeptiert (macOS, Android-Emulator, iOS-Simulator verifiziert; Windows nur Compile, Linux offen)
- Ergänzung 2026-10-03: Kompatibilität zu Go-Keychain-Einträgen ist **nicht** mehr gefordert (siehe Kontext)
- Datum: 2026-10-03
- Prototyp: Tag `archive/spike-m0-10` (früher Branch `spike/m0-10-secretstore`; `crates/nunc-platform/src/secret.rs`, `tests/keychain_compat.rs`)

## Kontext

Tokens (Todoist, Google Drive) und das WebDAV-Passwort gehören in den OS-Tresor. Die Roadmap sah ein eigenes
Kotlin-Plugin für Android vor und die Weiterverwendung der Go-Keychain-Einträge (`go-keyring`: `nunc-todoist/…`,
`nunc/google-drive`, `nunc/<sha256(url+user)>`), damit nach dem Umstieg kein Re-Login nötig ist.

**Entscheidung des Auftraggebers:** Die neue App ersetzt die Go-App; Nutzer richten Konten und Konfiguration neu ein.
Alte Go-Secrets müssen deshalb **nicht** gelesen werden können. Die Datenkompatibilität der SQLite-DB und der
Sync-Datei bleibt davon unberührt.

## Entscheidung

`keyring-core` 1.x plus je ein Store-Crate pro Plattform, gekapselt hinter dem Trait
`SecretStore { get, set, delete }` in `nunc-platform`:

| Plattform | Store-Crate | Backend |
|---|---|---|
| macOS | `apple-native-keyring-store` (`keychain`) | Login-Keychain |
| iOS | `apple-native-keyring-store` (`protected`) | Keychain (protected data); `protected` ist auf iOS Pflicht |
| Windows | `windows-native-keyring-store` | Credential Manager |
| Linux | ⏳ nicht gebaut: der Store-Code liegt nur auf dem Spike-Branch, `main`-CI baut ihn nicht. Beim Übernehmen nach `main` (M3-02) in der CI-Matrix prüfen |
| Android | `android-native-keyring-store` | Keystore-verschlüsselte SharedPreferences |

**Das Kotlin-Mini-Plugin entfällt.** Der Android-Store spricht Android per JNI direkt an und bekommt den
Kontext über `ndk-context`, den Tauri Mobile bereits initialisiert. Es ist keine Kotlin-Klasse, kein `.so` und
keine Manifest-Änderung nötig.

Es gibt keine Go-Dekodierung (`go-keyring-base64:`-Präfix) und keine festen Service-Namen der Go-App; die neue
App wählt eigene Namen (z. B. `nunc`, Account je Verwendung).

Das ältere Meta-Crate `keyring` 4.x wird bewusst nicht verwendet (laut eigener Doku nur für Demo/CLI gedacht,
zieht alle Stores nach).

## Ergebnisse

| Prüfung | Ergebnis |
|---|---|
| macOS: Rust schreibt Wert, anderer Prozess (`security find-generic-password`) liest ihn | ✅ |
| macOS: Rust liest Eintrag, den `security -i add-generic-password` angelegt hat; Überschreiben | ✅ (zeigt, dass der Store normale Keychain-Einträge nutzt; Go-Format wird nicht benötigt) |
| Android-Emulator (API 37): Schreiben → Lesen → Löschen mit Umlauten | ✅ ohne eigenen Kotlin-Code |
| iOS-Simulator (iPhone 17, iOS 27): dto. | ✅ |
| Windows: `cargo check --target x86_64-pc-windows-msvc` (Spike-Branch) | ✅ kompiliert, Laufzeit ungetestet |
| Linux | ⏳ nicht gebaut: der Store-Code liegt nur auf dem Spike-Branch, `main`-CI baut ihn nicht. Beim Übernehmen nach `main` (M3-02) in der CI-Matrix prüfen |

Die Tests gegen die Keychain nutzten ausschließlich eigene Wegwerf-Einträge (`nunc-compat-test-*`, danach
gelöscht).

## Konsequenzen / Risiken

- **macOS-Zugriffsdialog:** Der erste Zugriff einer anderen (unsignierten) Binary auf einen von einem anderen Programm
  angelegten Eintrag löst einen Keychain-Dialog aus (im Test über 60 s Wartezeit). Bei Dev-Builds erscheint er
  nach jedem Neubau, signierte Release-Builds fragen einmal („Immer erlauben"). In Tests/CI: ignorieren
  oder eigene Keychain nutzen.
- **Android-Store ist jung (1.0.0):** Keystore-Schlüssel liegt pro Store, Backups/„Daten löschen" führen zu
  einem leeren Tresor → App muss „nicht verbunden" sauber behandeln. Persistenz über Neustart/Kill wurde im
  Spike nicht gesondert geprüft.
- Linux: Ohne laufenden Secret-Service-Daemon (headless, minimale WMs) schlägt der Zugriff fehl → Fehlertext
  und Fallback-Hinweis in M8-02.
- Der Trait bleibt klein, sodass ein Tausch des Backends (z. B. `db-keystore`) lokal bleibt.
