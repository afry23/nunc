// SPIKE M0-09 – nicht für main gedacht.
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Schedule,
  pending as pendingNotifications,
  cancel,
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

// Workaround: iOS-Plugin parst das UTC-Datum als Ortszeit (siehe ADR 0004).
const iosOffsetMs = () =>
  /iPhone|iPad/.test(navigator.userAgent)
    ? -new Date().getTimezoneOffset() * 60000
    : 0;
const KEY = "spike.endAt";
const NOTIF_ID = 4711;

function App() {
  const [endAt, setEndAt] = useState<number | null>(() => {
    const v = localStorage.getItem(KEY);
    return v ? Number(v) : null;
  });
  const [now, setNow] = useState(Date.now());
  const [log, setLog] = useState<string[]>([]);
  const add = (m: string) => {
    const line = `${new Date().toISOString().slice(11, 23)} ${m}`;
    console.log(line);
    setLog((l) => [line, ...l].slice(0, 12));
  };

  // eslint-disable-next-line
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 250);
    const vis = () => add(`visibility=${document.visibilityState}`);
    document.addEventListener("visibilitychange", vis);
    add("mounted");
    void start(30);
    return () => {
      clearInterval(t);
      document.removeEventListener("visibilitychange", vis);
    };
  }, []);

  async function start(seconds: number) {
    let granted = await isPermissionGranted();
    if (!granted) granted = (await requestPermission()) === "granted";
    add(`permission=${granted}`);
    const end = Date.now() + seconds * 1000;
    localStorage.setItem(KEY, String(end));
    setEndAt(end);
    try {
      void sendNotification;
      await invoke("plugin:notification|notify", {
        options: {
          id: NOTIF_ID,
          title: "nunc",
          body: `Timer ${seconds}s fertig`,
          schedule: Schedule.at(new Date(end + iosOffsetMs()), false, true),
        },
      });
      const pending = await pendingNotifications();
      add(`pending=${pending.length} ${JSON.stringify(pending).slice(0, 160)}`);
      add(`scheduled for ${new Date(end).toISOString().slice(11, 23)}`);
    } catch (e) {
      add(`ERROR ${String(e)}`);
    }
  }

  async function stop() {
    await cancel([NOTIF_ID]);
    localStorage.removeItem(KEY);
    setEndAt(null);
    add("cancelled");
  }

  const remaining = endAt ? Math.max(0, endAt - now) : 0;
  return (
    <main style={{ padding: 16, paddingTop: 64 }}>
      <h1>nunc spike</h1>
      <p id="remaining">{endAt ? (remaining / 1000).toFixed(1) : "-"}</p>
      <button onClick={() => start(30)}>30s</button>{" "}
      <button onClick={() => start(120)}>120s</button>{" "}
      <button onClick={() => void stop()}>stop</button>
      <pre
        style={{ fontSize: 11, whiteSpace: "pre-wrap", wordBreak: "break-all" }}
      >
        {log.join("\n")}
      </pre>
    </main>
  );
}

export default App;
