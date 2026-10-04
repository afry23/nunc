import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { mkdtempSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";

/**
 * Desktop-E2E-Smoke (M4-12): WebdriverIO + `tauri-driver` + Cucumber (Linux/Windows; macOS hat keinen WebDriver
 * für WKWebView). Läuft gegen den Debug-Build; Daten liegen in einem Wegwerf-Ordner, der Pomodoro dauert
 * dank `NUNC_TEST_FAST_TIMER` nur wenige Sekunden.
 */
const root = path.resolve(import.meta.dirname, "..");
const exe = process.platform === "win32" ? "nunc.exe" : "nunc";
const dataHome = mkdtempSync(path.join(os.tmpdir(), "nunc-e2e-"));

// tauri-driver erbt die Umgebung und reicht sie an die App weiter
process.env.NUNC_TEST_FAST_TIMER = "3";
process.env.XDG_CONFIG_HOME = dataHome; // Linux
process.env.APPDATA = dataHome; // Windows

let driver: ChildProcess | undefined;

export const config = {
  host: "127.0.0.1",
  port: 4444,
  specs: [path.join(root, "features/e2e/**/*.feature")],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application: path.join(root, "target/debug", exe) },
    },
  ],
  framework: "cucumber",
  reporters: ["spec"],
  cucumberOpts: {
    import: [path.join(root, "e2e/steps/*.ts")],
    tags: "@e2e and @desktop and not @wip",
    timeout: 60_000,
  },
  onPrepare: () => {
    const build = spawnSync(
      "npm",
      ["run", "tauri", "build", "--", "--debug", "--no-bundle"],
      {
        cwd: root,
        stdio: "inherit",
        shell: true,
      },
    );
    if (build.status !== 0) throw new Error("tauri build fehlgeschlagen");
  },
  beforeSession: () => {
    driver = spawn(
      path.join(os.homedir(), ".cargo", "bin", "tauri-driver"),
      [],
      {
        stdio: [null, process.stdout, process.stderr],
      },
    );
  },
  afterSession: () => {
    driver?.kill();
  },
};
