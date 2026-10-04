# Mobile-E2E (Maestro)

Flows für Android und iOS (M5-06). Sie laufen gegen einen **Debug-Build auf Emulator/Simulator oder Gerät**; die
Szenarien stehen in `features/` (`@e2e @mobile`), die Flows setzen sie um.

| Flow | Szenario |
|---|---|
| `smoke.yaml` | Start, Todo anlegen, Timer starten/pausieren/stoppen |
| `kill-resume.yaml` | Timer überlebt `stopApp` (App-Kill) |
| `background-notification.yaml` | Ende im Hintergrund meldet eine Benachrichtigung, Resume zeigt den Stand (nur Android: `openNotification`) |

## Ausführen

```bash
brew install maestro                      # einmalig (braucht Java)
npm run tauri android build -- --debug --apk --target aarch64
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
npm run test:mobile                       # alle Flows; einzeln: maestro test e2e/mobile/smoke.yaml
```

iOS-Simulator: `npm run tauri ios build -- --debug --target aarch64-sim`, App mit `xcrun simctl install booted <app>`
installieren, dann `maestro test e2e/mobile/smoke.yaml e2e/mobile/kill-resume.yaml`. Der Hintergrund-Flow braucht
auf iOS eine eigene Variante (kein `openNotification`); offen, bis ein Gerät/Simulator mit Benachrichtigungs-Banner
zur Verfügung steht.

Hinweise: `todo-input` ist die `id` des Eingabefelds („Neue Aufgabe“). Der Test-Pomodoro wird über die
Einstellungen auf 1 Minute gesetzt, `NUNC_TEST_FAST_TIMER` wirkt auf Mobilgeräten nicht (keine Umgebung). Das
Antippen von Texten hängt von der Systemsprache der App ab (Deutsch); die Berechtigung wird per `permissions`
vorab erteilt. Die Flows wurden noch nicht mit Maestro ausgeführt (nicht installiert); die Abläufe wurden auf dem
Emulator von Hand geprüft (Start, Navigation, Tastatur, Exact-Alarm-Brücke).
