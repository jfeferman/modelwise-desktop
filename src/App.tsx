import { useCallback, useEffect, useState } from "react";
import { ConnectionRow } from "./features/status/ConnectionRow";
import { status } from "./lib/cli";
import type { Status } from "./lib/schema";

type State = { phase: "loading" } | { phase: "ready"; status: Status } | { phase: "failed"; message: string };

export function App() {
  const [state, setState] = useState<State>({ phase: "loading" });

  const refresh = useCallback(() => {
    setState({ phase: "loading" });
    status().then(
      (result) => setState({ phase: "ready", status: result }),
      (error) => setState({ phase: "failed", message: String(error) })
    );
  }, []);

  useEffect(refresh, [refresh]);

  return (
    <main className="panel">
      <header className="panel-head">
        <h1>Modelwise</h1>
        <button type="button" onClick={refresh} disabled={state.phase === "loading"}>
          {state.phase === "loading" ? "Checking…" : "Check again"}
        </button>
      </header>

      {state.phase === "failed" && <p className="problem problem-broken">{state.message}</p>}

      {state.phase === "ready" &&
        (state.status.connections.length === 0 ? (
          <p className="empty">
            Claude Code on this machine is not connected to Modelwise. Connect it from a terminal with{" "}
            <code>modelwise connect</code>.
          </p>
        ) : (
          <ul className="connections">
            {state.status.connections.map((connection) => (
              <ConnectionRow key={connection.configDir} connection={connection} />
            ))}
          </ul>
        ))}
    </main>
  );
}
