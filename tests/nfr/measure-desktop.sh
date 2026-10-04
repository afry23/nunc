#!/usr/bin/env bash
# NFR-Messung Desktop (roadmap M8-01): Startzeit und Arbeitsspeicher des Release-Builds.
#
#   npm run tauri build -- --no-bundle     # einmal bauen
#   tests/nfr/measure-desktop.sh [Binary] [Runden]
#
# Ziele: Oberfläche geladen < 2000 ms, Speicher (App + WebView-Prozesse) < 100 MB.
# Gemessen wird mit leerem Datenordner (HOME zeigt auf einen Wegwerf-Ordner), die Zeiten stammen aus dem
# Protokoll der App ("Start: … nach N ms", siehe src-tauri/src/logging.rs). Die App wird danach beendet.
# macOS: die WebView-Prozesse sind keine Kindprozesse; berücksichtigt werden alle `com.apple.WebKit.*`-Prozesse,
# die erst nach dem Start der App existieren. Linux: Kindprozesse (WebKitWebProcess, WebKitNetworkProcess).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
binary="${1:-$root/target/release/nunc}"
rounds="${2:-3}"
limit_start_ms=2000
limit_ram_mb=100

[[ -x "$binary" ]] || { echo "Binary fehlt: $binary (npm run tauri build -- --no-bundle)" >&2; exit 2; }

# Speicher eines Prozesses in KB. RSS zählt geteilte Seiten (Systembibliotheken, WebKit) mehrfach mit, deshalb:
# macOS `footprint` (phys_footprint, wie die Aktivitätsanzeige), Linux PSS aus smaps_rollup, sonst RSS.
mem_kb() {
  local pid="$1" value
  if [[ "$(uname)" == "Darwin" ]]; then
    value="$(footprint -p "$pid" 2>/dev/null | awk '/phys_footprint:/ {
      n = $2; unit = $3
      if (unit == "KB") n = n; else if (unit == "MB") n = n * 1024; else if (unit == "GB") n = n * 1024 * 1024
      printf "%d", n; exit }')"
  elif [[ -r "/proc/$pid/smaps_rollup" ]]; then
    value="$(awk '/^Pss:/ {print $2; exit}' "/proc/$pid/smaps_rollup")"
  fi
  [[ -n "${value:-}" ]] || value="$(ps -o rss= -p "$pid" 2>/dev/null | awk '{s+=$1} END {print s+0}')"
  echo "$value"
}

webkit_pids() { # macOS: alle WebKit-Hilfsprozesse
  pgrep -f 'com.apple.WebKit' 2>/dev/null || true
}

descendants() { # Linux: Kindprozesse rekursiv
  local pid="$1"
  for child in $(pgrep -P "$pid" 2>/dev/null || true); do
    echo "$child"
    descendants "$child"
  done
}

worst_start=0
worst_ram=0
for round in $(seq 1 "$rounds"); do
  home="$(mktemp -d)"
  before="$(webkit_pids | sort)"
  HOME="$home" XDG_DATA_HOME="$home/.local/share" XDG_CONFIG_HOME="$home/.config" "$binary" >/dev/null 2>&1 &
  app=$!
  log=""
  for _ in $(seq 1 100); do # bis zu 10 s warten
    log="$(find "$home" -name 'nunc*.log' 2>/dev/null | head -1)"
    if [[ -n "$log" ]] && grep -q 'Oberfläche geladen' "$log" 2>/dev/null; then break; fi
    sleep 0.1
  done
  sleep 5 # einschwingen lassen

  start_ms="$(grep -o 'Oberfläche geladen nach [0-9]* ms' "$log" 2>/dev/null | grep -o '[0-9]*' | head -1 || true)"
  setup_ms="$(grep -o 'Einrichtung abgeschlossen nach [0-9]* ms' "$log" 2>/dev/null | grep -o '[0-9]*' | head -1 || true)"

  pids="$app"
  if [[ "$(uname)" == "Darwin" ]]; then
    after="$(webkit_pids | sort)"
    pids="$pids $(comm -13 <(echo "$before") <(echo "$after") | tr '\n' ' ')"
  else
    pids="$pids $(descendants "$app" | tr '\n' ' ')"
  fi
  total_kb=0
  for pid in $pids; do total_kb=$((total_kb + $(mem_kb "$pid"))); done
  ram_mb=$((total_kb / 1024))

  echo "Runde $round: Einrichtung ${setup_ms:-?} ms, Oberfläche geladen ${start_ms:-?} ms, RAM ${ram_mb} MB (${pids})"
  [[ -n "${start_ms:-}" && "$start_ms" -gt "$worst_start" ]] && worst_start="$start_ms"
  [[ "$ram_mb" -gt "$worst_ram" ]] && worst_ram="$ram_mb"

  kill "$app" 2>/dev/null || true
  wait "$app" 2>/dev/null || true
  rm -rf "$home"
done

echo "Schlechtester Wert: Start ${worst_start} ms (Ziel < ${limit_start_ms}), RAM ${worst_ram} MB (Ziel < ${limit_ram_mb})"
if [[ "$worst_start" -eq 0 ]]; then echo "Startzeit nicht gemessen (kein Protokoll gefunden)" >&2; exit 2; fi
[[ "$worst_start" -lt "$limit_start_ms" && "$worst_ram" -lt "$limit_ram_mb" ]]
