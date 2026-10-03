// SPIKE M0-10 – nicht für main gedacht.
import { useEffect, useState } from "react";
import { commands } from "./lib/api";

function App() {
  const [result, setResult] = useState("läuft …");
  useEffect(() => {
    commands
      .secretSelftest()
      .then((r) =>
        setResult(r.status === "ok" ? `OK ${r.data}` : `ERR ${r.error}`),
      )
      .catch((e) => setResult(`EXC ${String(e)}`));
  }, []);
  return (
    <main style={{ padding: 16, paddingTop: 64 }}>
      <h1>nunc secret spike</h1>
      <pre style={{ whiteSpace: "pre-wrap", wordBreak: "break-all" }}>
        {result}
      </pre>
    </main>
  );
}

export default App;
