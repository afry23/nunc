// SPIKE M0-09 – nicht für main gedacht.
import { useEffect, useState } from "react";
import {
  Schedule,
  cancel,
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

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

  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 250);
    const vis = () => add(`visibility=${document.visibilityState}`);
    document.addEventListener("visibilitychange", vis);
    add("mounted");
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
    await sendNotification({
      id: NOTIF_ID,
      title: "nunc",
      body: `Timer ${seconds}s fertig`,
      schedule: Schedule.at(new Date(end), false, true),
    });
    add(`scheduled for ${new Date(end).toISOString().slice(11, 23)}`);
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
      <pre style={{ fontSize: 11 }}>{log.join("\n")}</pre>
    </main>
  );
}

export default App;
