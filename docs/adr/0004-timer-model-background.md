# ADR 0004 – Timer-Modell und Hintergrundverhalten (Spike M0-09)

- Status: akzeptiert (Android und iOS-Simulator verifiziert, echtes iOS-Gerät offen)
- Datum: 2026-10-03
- Prototyp: Branch `spike/m0-09-background-timer`

## Kontext

Mobile Betriebssysteme drosseln oder suspendieren Apps im Hintergrund. Ein Pomodoro-Timer muss trotzdem
pünktlich melden und nach Resume bzw. App-Kill den korrekten Stand zeigen. Geprüft wurden ein
Zeitstempel-Timer (`endAt` in absoluter Zeit) plus eine mit `tauri-plugin-notification` 2.5.1 geplante
Notification (`Schedule.at(date, false, allowWhileIdle = true)`).

## Testaufbau

Android-Emulator `Medium_Phone_API_35` (Android-API 37, arm64), Debug-APK, `POST_NOTIFICATIONS` per `adb` erteilt.
30-s-Timer, App per Home-Taste in den Hintergrund, Beobachtung per `dumpsys alarm` / `dumpsys notification`.

## Ergebnisse (Android)

| Szenario | Ergebnis |
|---|---|
| Ohne Exact-Alarm-Permission (Plugin-Default) | Plugin fällt auf `setAndAllowWhileIdle` zurück (`flags=0x20`, Fenster ≈ 22 s bei 30 s). Notification kam **~17–24 s zu spät**. Bei 25-min-Timern drohen Minuten Verzug. **Nicht akzeptabel.** |
| Mit `USE_EXACT_ALARM` im Manifest | `setExactAndAllowWhileIdle` (`window=0`, `exactAllowReason=policy_permission`). Notification **< 1 s** nach Zielzeit. |
| Mit `USE_EXACT_ALARM` + erzwungener Deep-Idle (`dumpsys deviceidle force-idle`) | Notification pünktlich (< 1 s). |
| App-Prozess hart beendet (`kill -9`) | Alarm bleibt im AlarmManager, Notification pünktlich. Relaunch vor Ablauf zeigt korrekte Restzeit (14,4 s statt erwarteter ~14,2 s), Relaunch nach Ablauf `0`. |
| Resume aus Hintergrund (Prozess lebt) | `visibilitychange` feuert, Restzeit wird aus `endAt − Date.now()` neu berechnet; keine Drift durch gedrosselte JS-Timer. |

iOS (Simulator iPhone 17, iOS 27, `tauri ios dev`, Zeitzone UTC+2):

| Szenario | Ergebnis |
|---|---|
| `sendNotification({ schedule: Schedule.at(...) })` unverändert | **Schlägt still fehl.** Die iOS-Seite des Plugins (2.5.1) parst das vom Rust-Teil gelieferte UTC-Datum mit einem `DateFormatter` ohne Zeitzone, also als Ortszeit. In Zeitzonen östlich von UTC liegt der Termin dadurch in der Vergangenheit → `NotificationError.pastScheduledTime`, in Zeitzonen westlich von UTC käme die Notification entsprechend Stunden zu spät. Es erscheint nichts, auch kein Fehler in der UI. |
| Dasselbe Datum um den Zeitzonen-Offset korrigiert (`end − getTimezoneOffset()·60 000 ms`) | Notification mit App im Hintergrund (Einstellungen in den Vordergrund geholt) **auf die Sekunde pünktlich** (Banner 14:23:44 bei Ziel 14:23:43.9); `pending()` liefert die geplante Notification. |

Ursache der Stille: `sendNotification` ruft intern `new window.Notification(...)` auf, einen nicht awaiteten
Shim. Ablehnungen des Backends landen als unbehandelte Promise-Rejection und sind mit `try/catch` um
`sendNotification` nicht sichtbar. Fehler lassen sich nur über `invoke("plugin:notification|notify", …)`
bzw. später über den eigenen `TauriNotifier` (Rust, `show()`) abfangen.

Der Simulator läuft headless; das Backgrounding wurde über `simctl launch com.apple.Preferences`
ausgelöst, die Berechtigung war im Simulator bereits erteilt (Dialog-Verhalten daher nicht geprüft).
Echtes Gerät steht weiterhin aus (Focus-Modi, Low-Power-Mode).

Nicht getestet: `am force-stop` (löscht Alarme laut Android-Verhalten, App gilt dann als „gestoppt"),
Reboot (Plugin bringt `LocalNotificationRestoreReceiver` mit), herstellerspezifische Akku-Optimierung
(echte Geräte, Milestone M5).

## Entscheidung

1. **Autoritative Zeit ist ein absoluter Zeitstempel** (`started_at` + Segmente bzw. `ends_at`), nie ein
   hochgezählter Zähler. Ticks (`timer:tick`) dienen nur der Anzeige im Vordergrund und werden bei Resume
   aus dem Zeitstempel neu berechnet. Der Timer-Zustand wird persistiert (`active_timer`, M2-02), damit er
   App-Kill überlebt.
2. **Ablauf im Hintergrund meldet eine geplante lokale Notification** (`Schedule.at`, `allowWhileIdle = true`),
   bei Start/Resume gesetzt, bei Pause/Stop/Abbruch per `cancel` entfernt.
3. **Android:** `USE_EXACT_ALARM` im Manifest deklarieren (`src-tauri/gen/android/app/src/main/AndroidManifest.xml`).
   Der Permission-Typ wird für Timer-/Wecker-Apps automatisch gewährt und ist nicht widerrufbar.
   `SCHEDULE_EXACT_ALARM` wurde bewusst nicht gewählt (ab Android 14 standardmäßig verweigert, vom Nutzer
   widerrufbar, Fallback auf ungenau).
4. **Notification-Permission** (`POST_NOTIFICATIONS`, Android 13+; iOS-Dialog) wird per Plugin beim ersten
   Timer-Start angefragt (M5-03). Ohne Erlaubnis bleibt der Timer korrekt, nur ohne Hintergrund-Hinweis.
5. Timer-Laufzeit im Backend (Rust, M3-04) bleibt gültig; die Notification-Planung läuft über das Plugin
   (`TauriNotifier`-Adapter in `nunc-platform`), nicht aus dem WebView.

## Risiken / offen

- **Play-Store-Policy:** `USE_EXACT_ALARM` ist auf Alarm-/Timer-/Kalender-Apps beschränkt. Ein Pomodoro-Timer
  fällt darunter; in der Play-Console-Deklaration (M8-05) begründen. Fallback: `SCHEDULE_EXACT_ALARM` +
  Hinweisbanner, falls abgelehnt.
- **iOS-Plugin-Bug (Zeitzone):** Notifications dürfen auf iOS nicht über `Schedule.at` des unveränderten Plugins
  geplant werden. Optionen: (a) Datum im `TauriNotifier` vor dem Plugin-Aufruf um den lokalen UTC-Offset
  verschieben (funktioniert, aber zu DST-Wechseln hin ungenau), (b) Plugin per `[patch.crates-io]` mit
  `dateFormatter.timeZone = UTC` patchen und den Fix upstream melden (empfohlen), (c) `Schedule.interval`/
  eigener Swift-Befehl. Entscheidung in M3-02/M5-03; bis dahin Test im Spike-Branch reproduzierbar.
- iOS-Gerät: Verifikation auf echtem Gerät (Fokus-Modi, Low-Power-Mode, Berechtigungsdialog) vor M5-03.
  Erwartung: lokale `UNCalendarNotificationTrigger`-Notifications feuern auch bei suspendierter App;
  Verifikation auf Simulator/Gerät nachholen, bevor M5 startet.
- Die Notification liefert nur den Hinweis, nicht den Sound/Zustandswechsel. Nach dem Öffnen muss die App
  den abgelaufenen Timer verbuchen (Session `completed`) – Teil von M3-04.
- Laufender Timer außerhalb der App (Foreground-Service, Live Activity) bleibt Backlog nach 1.0.
