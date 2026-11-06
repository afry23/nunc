# ADR 0031 – Informationsarchitektur: Heute · Aufgaben · Verlauf · Rückblick

- Status: vorgeschlagen
- Datum: 2026-11-06
- Ergänzt: ADR 0025 (Rückblick), ADR 0027 (Tagesplaner)
- Plan: [ui-umbau-plan.md](../ui-umbau-plan.md)

## Kontext

Die Navigation zeigte Werkzeuge (Timer, Aufgaben, Zeiten, Statistik) statt des Arbeitsablaufs. „Heute“ war nur über
einen Link erreichbar, „Zeiten“ mischte Wochenübersicht und Rückblick, der Timer war die Eingangstür der App.

## Entscheidung

Vier gleichrangige Hauptbereiche, auf Desktop (Seitenleiste) und Android (untere Leiste) gleich:

| Route | Bereich | Frage |
|---|---|---|
| `/` | Heute (Startseite) | Was nehme ich mir heute vor? |
| `/tasks` | Aufgaben | Was gibt es zu tun? |
| `/log` | Verlauf | Was ist wirklich passiert? |
| `/review` | Rückblick | Was ist dabei herausgekommen? |

- Der Timer ist keine Startseite mehr, sondern der **Fokusmodus** unter `/focus` (E1: über „Timer“ in Heute erreichbar;
  E2: globale Aktiv-Leiste).
- Die Timerstatistik (`/stats`) bleibt vorerst bestehen und hängt als „Arbeitsweise“ am Rückblick (E5 ordnet sie ein).
- Alte Adressen `/today`, `/todos`, `/times` leiten auf die neuen um.
- Tastenkürzel 1–4 folgen den vier Bereichen.
- „Heute“ trennt **Geplanter Aufwand** (Schätzungen gegen Budget) von **Erfasster Arbeit** (Vereinigung der
  Arbeitsblöcke des Tages, Pausen zählen nicht, Überlappungen einmal). Die Zahl wird aus den Sessions berechnet, nicht
  gespeichert.

## Folgen

- Kein Backend- und kein Schema-Eingriff in dieser Stufe.
- Maestro- und WebdriverIO-Flows starten auf „Heute“ und gehen für den Timer über „Timer“.
- Weitere Etappen: [ui-umbau-plan.md](../ui-umbau-plan.md) (E2–E6, darunter das Entfernen des Habit-Trackers als eigener ADR).
