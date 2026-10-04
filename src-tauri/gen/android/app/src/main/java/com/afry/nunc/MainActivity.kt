package com.afry.nunc

import android.app.AlarmManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.graphics.Color
import android.os.Bundle
import android.provider.Settings
import android.view.View
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    // Die App ist dunkel: helle Symbole in Status- und Navigationsleiste, auch wenn das System hell ist
    enableEdgeToEdge(
      statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
      navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
    )
    super.onCreate(savedInstanceState)
    // Edge-to-edge: Android verkleinert das Fenster nicht mehr für die Tastatur. Der Inhalt bekommt
    // deshalb unten den Tastaturabstand, damit das Eingabefeld sichtbar bleibt.
    ViewCompat.setOnApplyWindowInsetsListener(findViewById<View>(android.R.id.content)) { view, insets ->
      view.setPadding(0, 0, 0, insets.getInsets(WindowInsetsCompat.Type.ime()).bottom)
      insets
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(NuncBridge(this), "NuncAndroid")
  }
}

/**
 * Kleine Brücke zu Android-Funktionen, die kein Tauri-Plugin bietet (siehe `src/lib/notifications.ts`):
 * Abfrage der Exact-Alarm-Erlaubnis und Sprung in die Systemeinstellungen. Ohne Schreibzugriff auf Daten.
 */
class NuncBridge(private val context: Context) {
  /** Android 12+: ohne die Erlaubnis verspäten sich geplante Benachrichtigungen (ADR 0004). */
  @JavascriptInterface
  fun canScheduleExactAlarms(): Boolean {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) return true
    val alarms = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
    return alarms.canScheduleExactAlarms()
  }

  /** „Wecker & Erinnerungen“ der App (nur Android 12+; davor gibt es nichts umzustellen). */
  @JavascriptInterface
  fun openExactAlarmSettings() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) return
    open(Intent(Settings.ACTION_REQUEST_SCHEDULE_EXACT_ALARM, Uri.parse("package:${context.packageName}")))
  }

  @JavascriptInterface
  fun openNotificationSettings() {
    open(
      Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
        .putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
    )
  }

  private fun open(intent: Intent) {
    try {
      context.startActivity(intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    } catch (_: Exception) {
      // Einstellungsseite auf diesem Gerät nicht vorhanden: der Hinweis in der App bleibt stehen
    }
  }
}
