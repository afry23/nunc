import { describe, expect, it } from "vitest";
import {
  TARGETS,
  isVersion,
  readVersion,
  replaceVersion,
} from "./bump-version.mjs";

const sample = {
  "package.json":
    '{\n  "name": "nunc",\n  "version": "0.1.0",\n  "dependencies": { "x": "^1.2.3" }\n}',
  "src-tauri/tauri.conf.json":
    '{\n  "productName": "nunc",\n  "version": "0.1.0"\n}',
  "Cargo.toml":
    '[workspace]\n[workspace.package]\nversion = "0.1.0"\n[workspace.dependencies]\nserde = { version = "1" }',
  "src-tauri/gen/apple/project.yml":
    '      properties:\n        CFBundleShortVersionString: 0.1.0\n        CFBundleVersion: "0.1.0"\n',
};

describe("bump-version", () => {
  it("erkennt nur X.Y.Z als Version", () => {
    expect(isVersion("1.0.0")).toBe(true);
    expect(isVersion("1.0")).toBe(false);
    expect(isVersion("v1.0.0")).toBe(false);
  });

  it("findet und ersetzt die Version in jeder Datei, ohne Abhängigkeiten anzufassen", () => {
    for (const { file, pattern } of TARGETS) {
      const text = sample[file];
      expect(readVersion(text, pattern), file).toBe("0.1.0");
      const next = replaceVersion(text, pattern, "1.2.3");
      expect(readVersion(next, pattern), file).toBe("1.2.3");
    }
    const cargo = TARGETS.find((t) => t.file === "Cargo.toml");
    expect(
      replaceVersion(sample["Cargo.toml"], cargo.pattern, "1.2.3"),
    ).toContain('serde = { version = "1" }');
  });
});
