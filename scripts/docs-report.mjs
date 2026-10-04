#!/usr/bin/env node
// Erzeugt aus `features/**/*.feature` die Abnahme-Berichte (roadmap M8-07/M8-08):
//   - traceability.md  Story/Anforderung → Szenario → Status
//   - living-docs.md   alle Features mit ihren Szenarien (Living Documentation)
// Optional werden Testergebnisse aus dem Cucumber-JSON (NUNC_BDD_REPORT) eingemischt.
//
//   node scripts/docs-report.mjs [--out target/docs] [--check] [--results target/bdd/domain.json]
//
// `--check` beendet mit Fehlercode 1, wenn ein Szenario `@wip` trägt, keinen Ebenen-Tag
// (@domain/@integration/@ui/@e2e) hat oder keine Story/Anforderung (@US-…/@REQ-…) nennt.
import {
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
  existsSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
/** Runner-Selbsttests (`features/smoke*.feature`) gehören zu keiner Story. */
const SELFTEST = /(^|\/)smoke[^/]*\.feature$/;
const LAYERS = ["domain", "integration", "ui", "e2e"];

/** Alle `.feature`-Dateien unterhalb von `dir`, sortiert. */
export function featureFiles(dir) {
  const files = [];
  for (const name of readdirSync(dir).sort()) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) files.push(...featureFiles(path));
    else if (name.endsWith(".feature")) files.push(path);
  }
  return files;
}

/** Liest Feature-Titel und Szenarien (inkl. Feature-Tags, Rule-Tags werden nicht benutzt). */
export function parseFeature(text, file) {
  const feature = { file, title: file, tags: [], scenarios: [] };
  let pending = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.startsWith("@")) {
      pending.push(...line.split(/\s+/).filter((t) => t.startsWith("@")));
    } else if (/^Feature:/.test(line)) {
      feature.title = line.replace(/^Feature:\s*/, "");
      feature.tags = pending;
      pending = [];
    } else if (/^(Scenario|Scenario Outline):/.test(line)) {
      const title = line.replace(/^Scenario( Outline)?:\s*/, "");
      const tags = [...feature.tags, ...pending];
      feature.scenarios.push({ title, tags: tags.map((t) => t.slice(1)) });
      pending = [];
    } else if (line !== "" && !line.startsWith("#")) {
      pending = [];
    }
  }
  return feature;
}

export function statusOf(scenario) {
  if (scenario.tags.includes("wip")) return "wip";
  if (scenario.tags.includes("manual")) return "manuell";
  if (scenario.tags.includes("mobile") && scenario.tags.includes("e2e"))
    return "Maestro";
  return "automatisiert";
}

export function layerOf(scenario) {
  return LAYERS.filter((l) => scenario.tags.includes(l)).join("+") || "?";
}

export function storiesOf(scenario) {
  return scenario.tags.filter((t) => /^(US|REQ)-/.test(t));
}

/** Probleme, die die Abnahme (M8-08) verhindern. */
export function problems(features) {
  const found = [];
  for (const f of features) {
    for (const s of f.scenarios) {
      const where = `${f.file}: ${s.title}`;
      if (s.tags.includes("wip")) found.push(`${where} – @wip`);
      if (layerOf(s) === "?") found.push(`${where} – kein Ebenen-Tag`);
      if (storiesOf(s).length === 0 && !SELFTEST.test(f.file))
        found.push(`${where} – keine Story/Anforderung`);
    }
  }
  return found;
}

/** `Szenario-Titel → passed|failed|skipped` aus einem Cucumber-JSON (cucumber-rs/-js). */
export function readResults(path) {
  const results = new Map();
  if (!path || !existsSync(path)) return results;
  for (const feature of JSON.parse(readFileSync(path, "utf8"))) {
    for (const element of feature.elements ?? []) {
      const steps = element.steps ?? [];
      const statuses = steps.map((s) => s.result?.status ?? "skipped");
      const status = statuses.includes("failed")
        ? "failed"
        : statuses.some(
              (s) => s === "skipped" || s === "undefined" || s === "pending",
            )
          ? "skipped"
          : "passed";
      results.set(`${feature.name}::${element.name}`, status);
    }
  }
  return results;
}

const cell = (text) => text.replaceAll("|", "\\|");

export function traceability(features, results = new Map()) {
  const rows = new Map();
  for (const f of features) {
    for (const s of f.scenarios) {
      for (const id of storiesOf(s)) {
        if (!rows.has(id)) rows.set(id, []);
        rows.get(id).push({ f, s });
      }
    }
  }
  const out = ["# Traceability: Story → Szenario → Status", ""];
  const total = features.reduce((n, f) => n + f.scenarios.length, 0);
  const wip = features.reduce(
    (n, f) => n + f.scenarios.filter((s) => s.tags.includes("wip")).length,
    0,
  );
  out.push(
    `${rows.size} Stories/Anforderungen, ${total} Szenarien, ${wip} davon \`@wip\`.`,
    "",
  );
  for (const id of [...rows.keys()].sort()) {
    out.push(
      `## ${id}`,
      "",
      "| Szenario | Datei | Ebene | Status | Lauf |",
      "|---|---|---|---|---|",
    );
    for (const { f, s } of rows.get(id)) {
      const run = results.get(`${f.title}::${s.title}`) ?? "";
      out.push(
        `| ${cell(s.title)} | \`${f.file}\` | ${layerOf(s)} | ${statusOf(s)} | ${run} |`,
      );
    }
    out.push("");
  }
  return out.join("\n");
}

export function livingDocs(features) {
  const out = [
    "# Living Documentation",
    "",
    "Erzeugt aus `features/**/*.feature` – die Szenarien sind die Spezifikation.",
    "",
  ];
  for (const f of features) {
    out.push(`## ${f.title}`, "", `\`${f.file}\``, "");
    for (const s of f.scenarios) {
      const ids = storiesOf(s).join(", ");
      out.push(
        `- ${s.title} _(${layerOf(s)}, ${statusOf(s)}${ids ? `, ${ids}` : ""})_`,
      );
    }
    out.push("");
  }
  return out.join("\n");
}

function main(argv) {
  const arg = (name, fallback) => {
    const i = argv.indexOf(name);
    return i >= 0 ? argv[i + 1] : fallback;
  };
  const out = join(root, arg("--out", "target/docs"));
  const dir = join(root, "features");
  const features = featureFiles(dir)
    .map((file) =>
      parseFeature(readFileSync(file, "utf8"), relative(root, file)),
    )
    .filter((f) => f.scenarios.length > 0);
  const results = readResults(arg("--results") && join(root, arg("--results")));
  mkdirSync(out, { recursive: true });
  writeFileSync(join(out, "traceability.md"), traceability(features, results));
  writeFileSync(join(out, "living-docs.md"), livingDocs(features));
  console.log(`Berichte in ${relative(root, out)}/`);
  const found = problems(features);
  for (const p of found) console.error(`✗ ${p}`);
  if (argv.includes("--check") && found.length > 0) process.exit(1);
}

if (process.argv[1] === fileURLToPath(import.meta.url))
  main(process.argv.slice(2));
