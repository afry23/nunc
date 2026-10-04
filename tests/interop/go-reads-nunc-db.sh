#!/usr/bin/env bash
# Interop-Check (manuell, Szenario "A database written by nunc can be opened by the Go app"):
# nunc-store schreibt eine Datenbank, die Go-App-Logik (db.OpenDB, Settings-/Todo-/Stats-Service) liest sie.
# Voraussetzung: Go und ../nunc-go. Ausgabe: JSON der Go-Sicht; Exit-Code != 0 bei Fehler.
set -euo pipefail
cd "$(dirname "$0")/../.."

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

cargo run -q -p nunc-store --example write_sample -- "$tmp/nunc.db" >&2
cp -R ../nunc-go/internal ../nunc-go/go.mod ../nunc-go/go.sum "$tmp/"
mkdir -p "$tmp/cmd/read" && cp tests/interop/go_reads.go.txt "$tmp/cmd/read/main.go"
(cd "$tmp" && go run ./cmd/read "$tmp/nunc.db" "$tmp/backups/nunc-backup-2026-10-04.db")
