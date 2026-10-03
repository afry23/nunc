# ADR 0004 – Timer-Modell und Hintergrundverhalten (Spike M0-09)

- Status: akzeptiert (Android verifiziert, iOS offen)
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
- **iOS nicht verifiziert** (Xcode-Lizenz/`xcode-select` auf der Entwicklermaschine fehlt, siehe M0-08).
  Erwartung: lokale `UNCalendarNotificationTrigger`-Notifications feuern auch bei suspendierter App;
  Verifikation auf Simulator/Gerät nachholen, bevor M5 startet.
- Die Notification liefert nur den Hinweis, nicht den Sound/Zustandswechsel. Nach dem Öffnen muss die App
  den abgelaufenen Timer verbuchen (Session `completed`) – Teil von M3-04.
- Laufender Timer außerhalb der App (Foreground-Service, Live Activity) bleibt Backlog nach 1.0.
