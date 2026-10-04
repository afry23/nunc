#!/usr/bin/env node
// Setzt die Version überall gleich (roadmap M8-04) oder prüft sie:
//   node scripts/bump-version.mjs 1.0.0            schreibt package.json, Cargo.toml (Workspace),
//                                                   tauri.conf.json und gen/apple/project.yml
//   node scripts/bump-version.mjs --check 1.0.0    beendet mit Fehler, wenn eine Datei abweicht
// Android leitet versionName/versionCode aus `tauri.conf.json` ab (Tauri schreibt tauri.properties).
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

/** Jede Datei: Muster, das die Version in Gruppe 2 findet (Gruppe 1 und 3 bleiben stehen). */
export const TARGETS = [
  { file: "package.json", pattern: /^(\s*"version":\s*")([^"]+)(")/m },
  {
    file: "src-tauri/tauri.conf.json",
    pattern: /^(\s*"version":\s*")([^"]+)(")/m,
  },
  { file: "Cargo.toml", pattern: /^(version\s*=\s*")([^"]+)(")/m },
  {
    file: "src-tauri/gen/apple/project.yml",
    pattern: /^(\s*CFBundleShortVersionString:\s*)([^\s]+)()/m,
  },
  {
    file: "src-tauri/gen/apple/project.yml",
    pattern: /^(\s*CFBundleVersion:\s*")([^"]+)(")/m,
  },
];

export const isVersion = (text) => /^\d+\.\d+\.\d+$/.test(text);

/** Liest die Version aus dem Text einer Datei. */
export function readVersion(text, pattern) {
  return pattern.exec(text)?.[2];
}

export function replaceVersion(text, pattern, version) {
  return text.replace(
    pattern,
    (_, before, _old, after) => `${before}${version}${after}`,
  );
}

function main(argv) {
  const check = argv.includes("--check");
  const version = argv.find((a) => !a.startsWith("--"));
  if (!version || !isVersion(version)) {
    console.error("Aufruf: bump-version.mjs [--check] X.Y.Z");
    process.exit(2);
  }
  let failed = false;
  for (const { file, pattern } of TARGETS) {
    const path = join(root, file);
    const text = readFileSync(path, "utf8");
    const found = readVersion(text, pattern);
    if (check) {
      if (found !== version) {
        console.error(
          `✗ ${file}: ${found ?? "keine Version gefunden"} statt ${version}`,
        );
        failed = true;
      }
    } else if (found === undefined) {
      console.error(`✗ ${file}: keine Version gefunden`);
      failed = true;
    } else {
      writeFileSync(path, replaceVersion(text, pattern, version));
      console.log(`${file}: ${found} → ${version}`);
    }
  }
  if (failed) process.exit(1);
}

if (process.argv[1] === fileURLToPath(import.meta.url))
  main(process.argv.slice(2));
