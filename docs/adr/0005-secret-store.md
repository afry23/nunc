# ADR 0005 – SecretStore (Spike M0-10)

- Status: akzeptiert (macOS, Android-Emulator, iOS-Simulator verifiziert; Windows/Linux nur Compile bzw. offen)
- Datum: 2026-10-03
- Prototyp: Branch `spike/m0-10-secretstore` (`crates/nunc-platform/src/secret.rs`, `tests/keychain_compat.rs`)

## Kontext

Tokens (Todoist, Google Drive) und das WebDAV-Passwort gehören in den OS-Tresor. Die Go-App nutzt
`zalando/go-keyring` mit den Einträgen `nunc-todoist/{access_token,refresh_token}`, `nunc/google-drive`
(JSON) und `nunc/<sha256(url+user)>` (WebDAV). Beim Umstieg soll kein Re-Login nötig sein (M7-07).
Die Roadmap sah ein eigenes Kotlin-Plugin für Android vor.

## Entscheidung

`keyring-core` 1.x plus je ein Store-Crate pro Plattform, gekapselt hinter dem Trait
`SecretStore { get, set, delete }` in `nunc-platform`:

| Plattform | Store-Crate | Backend |
|---|---|---|
| macOS | `apple-native-keyring-store` (`keychain`) | Login-Keychain, gleiche Einträge wie die Go-App |
| iOS | `apple-native-keyring-store` (`protected`) | Keychain (protected data); `protected` ist auf iOS Pflicht |
| Windows | `windows-native-keyring-store` | Credential Manager |
| Linux | `zbus-secret-service-keyring-store` (`crypto-rust`) | Secret Service (GNOME Keyring / KWallet) |
| Android | `android-native-keyring-store` | Keystore-verschlüsselte SharedPreferences |

**Das Kotlin-Mini-Plugin entfällt.** Der Android-Store spricht Android per JNI direkt an und bekommt den
Kontext über `ndk-context`, den Tauri Mobile bereits initialisiert. Es ist keine Kotlin-Klasse, kein `.so` und
keine Manifest-Änderung nötig.

Das ältere Meta-Crate `keyring` 4.x wird bewusst nicht verwendet (laut eigener Doku nur für Demo/CLI gedacht,
zieht alle Stores nach).

## Ergebnisse

| Prüfung | Ergebnis |
|---|---|
| macOS: Rust liest Wert, den die Go-Logik schreibt (`security -i add-generic-password`, Präfix `go-keyring-base64:`, Umlaute, Zeilenumbruch, JSON) | ✅ nach Dekodierung (`decode_go_keyring`) identisch |
| macOS: Go-Logik (`security find-generic-password -wa`) liest Wert, den Rust schreibt | ✅ (Rust schreibt Klartext ohne Präfix; go-keyring gibt Werte ohne Präfix unverändert zurück) |
| macOS: Rust überschreibt Go-Eintrag | ✅ |
| Android-Emulator (API 37): Schreiben → Lesen → Löschen mit Umlauten | ✅ ohne eigenen Kotlin-Code |
| iOS-Simulator (iPhone 17, iOS 27): dto. | ✅ |
| Windows: `cargo check --target x86_64-pc-windows-msvc` | ✅ kompiliert, Laufzeit ungetestet |
| Linux | ⏳ nicht geprüft (Target lokal nicht installiert) → in CI-Matrix prüfen |
| Echte Go-Einträge (`nunc-todoist`, `nunc/google-drive`, WebDAV) | ⏳ **von Hand zu prüfen**, siehe unten |

Die Tests gegen die Keychain nutzten ausschließlich eigene Wegwerf-Einträge (`nunc-compat-test-*`, danach
gelöscht). Echte Einträge der Go-App wurden bewusst nicht ausgelesen.

### Offen: echte Go-Einträge

`cargo test -p nunc-platform --test keychain_compat real_go_entries -- --ignored --nocapture` prüft nur, ob
`nunc-todoist/access_token` und `nunc/google-drive` vorhanden und dekodierbar sind, und gibt nur die
Zeichenzahl aus, nie den Wert. Der Test muss auf dem Rechner mit der Go-App von Hand laufen.

## Konsequenzen / Risiken

- **Format-Kompatibilität:** `go-keyring` kodiert auf macOS alle Werte als `go-keyring-base64:<b64>`
  (ältere Versionen `go-keyring-encoded:<hex>`). Der `SecretStore` dekodiert beim Lesen. Auf Windows/Linux
  schreibt go-keyring Klartext (Windows: UTF-16-Blob im Credential Manager, Linux: Secret-Service-Attribute
  `service`/`username`), das ist für M7-07 auf diesen Plattformen noch gegenzuprüfen.
- **macOS-Zugriffsdialog:** Der erste Zugriff einer anderen (unsignierten) Binary auf einen von `/usr/bin/security`
  angelegten Eintrag löst einen Keychain-Dialog aus (im Test über 60 s Wartezeit). Bei Dev-Builds erscheint er
  nach jedem Neubau, signierte Release-Builds fragen einmal („Immer erlauben"). In Tests/CI: ignorieren
  oder eigene Keychain nutzen.
- **Android-Store ist jung (1.0.0):** Keystore-Schlüssel liegt pro Store, Backups/„Daten löschen" führen zu
  einem leeren Tresor → App muss „nicht verbunden" sauber behandeln. Persistenz über Neustart/Kill wurde im
  Spike nicht gesondert geprüft.
- Linux: Ohne laufenden Secret-Service-Daemon (headless, minimale WMs) schlägt der Zugriff fehl → Fehlertext
  und Fallback-Hinweis in M8-02.
- Der Trait bleibt klein, sodass ein Tausch des Backends (z. B. `db-keystore`) lokal bleibt.
