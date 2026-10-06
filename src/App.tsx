import { useCallback, useEffect, useState } from "react";
import { ConnectForm } from "./features/connect/ConnectForm";
import { ConnectionRow } from "./features/status/ConnectionRow";
import { setPaused, status, syncNow } from "./lib/cli";
import type { Status } from "./lib/schema";

type State = { phase: "loading" } | { phase: "ready"; status: Status } | { phase: "failed"; message: string };

export function App() {
  const [state, setState] = useState<State>({ phase: "loading" });
  const [syncing, setSyncing] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [showConnect, setShowConnect] = useState(false);

  const refresh = useCallback(() => {
    setState({ phase: "loading" });
    status().then(
      (result) => setState({ phase: "ready", status: result }),
      (error) => setState({ phase: "failed", message: String(error) })
    );
  }, []);

  useEffect(refresh, [refresh]);

  async function sync() {
    setSyncing(true);
    setNotice(null);

    try {
      const sent = (await syncNow()).results.reduce((total, result) => total + result.sent, 0);
      setNotice(sent === 0 ? "Nothing new to upload." : `Uploaded ${sent.toLocaleString()} spans.`);
    } catch (error) {
      setNotice(String(error));
    } finally {
      setSyncing(false);
      refresh();
    }
  }

  async function togglePaused(paused: boolean) {
    try {
      await setPaused(paused);
    } catch (error) {
      setNotice(String(error));
    }
    refresh();
  }

  const ready = state.phase === "ready" ? state.status : null;
  const connected = ready !== null && ready.connections.length > 0;

  return (
    <main className="panel">
      <header className="panel-head">
        <h1>Modelwise</h1>
        <div className="actions">
          {connected && (
            <button type="button" onClick={sync} disabled={syncing}>
              {syncing ? "Syncing…" : "Sync now"}
            </button>
          )}
          <button type="button" onClick={refresh} disabled={state.phase === "loading"}>
            {state.phase === "loading" ? "Checking…" : "Check again"}
          </button>
        </div>
      </header>

      {state.phase === "failed" && <p className="problem problem-broken">{state.message}</p>}
      {notice && <p className="hint">{notice}</p>}

      {ready &&
        (connected ? (
          <ul className="connections">
            {ready.connections.map((connection) => (
              <ConnectionRow key={connection.configDir} connection={connection} onChanged={refresh} />
            ))}
          </ul>
        ) : (
          <ConnectForm onConnected={refresh} />
        ))}

      {connected && (
        <footer className="panel-foot">
          <label className="toggle">
            <input
              type="checkbox"
              checked={!ready.app.paused}
              onChange={(event) => togglePaused(!event.target.checked)}
            />
            Sync in the background every hour
          </label>
          {showConnect ? (
            <ConnectForm onConnected={() => { setShowConnect(false); refresh(); }} />
          ) : (
            <button type="button" className="link" onClick={() => setShowConnect(true)}>
              Connect another Modelwise…
            </button>
          )}
        </footer>
      )}
    </main>
  );
}
