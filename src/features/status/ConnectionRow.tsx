import { useState } from "react";
import { disconnect, repair } from "../../lib/cli";
import type { Connection } from "../../lib/schema";
import { UsageLine } from "./UsageLine";

const LABEL = { working: "Working", attention: "Needs attention", broken: "Not working" } as const;

function host(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
}

function lastSync(connection: Connection): string {
  if (!connection.lastSync) {
    return "Never synced";
  }

  const at = new Date(connection.lastSync.at);
  return `Last sync ${at.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}`;
}

export function ConnectionRow({ connection, onChanged }: { connection: Connection; onChanged: () => void }) {
  const { health, problems } = connection;
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState<"repair" | "disconnect" | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const canRepair = connection.check !== undefined && connection.check.settings !== "ok" && connection.check.token !== "revoked";

  async function act(what: "repair" | "disconnect") {
    setBusy(what);
    setFailure(null);

    try {
      await (what === "repair" ? repair(connection.configDir) : disconnect(connection.configDir));
      onChanged();
    } catch (error) {
      setFailure(String(error));
    } finally {
      setBusy(null);
      setConfirming(false);
    }
  }

  return (
    <li className="connection">
      <div className="connection-head">
        <span className={`dot dot-${health}`} role="img" aria-label={LABEL[health]} />
        <span className="connection-name">{connection.name}</span>
        <span className="connection-host">{host(connection.url)}</span>
      </div>
      <div className="connection-detail" title={connection.configDir}>
        {connection.configDir}
      </div>
      <div className="connection-detail">{lastSync(connection)}</div>
      {connection.check?.token === "live" && <UsageLine configDir={connection.configDir} />}
      {problems.map((problem) => (
        <div key={problem} className={`problem problem-${health}`}>
          {problem}
        </div>
      ))}
      {failure && <div className="problem problem-broken">{failure}</div>}
      <div className="actions">
        {canRepair && (
          <button type="button" onClick={() => act("repair")} disabled={busy !== null}>
            {busy === "repair" ? "Repairing…" : "Repair settings"}
          </button>
        )}
        {confirming ? (
          <>
            <span className="hint">Remove the settings and revoke this machine's token?</span>
            <button type="button" className="danger" onClick={() => act("disconnect")} disabled={busy !== null}>
              {busy === "disconnect" ? "Disconnecting…" : "Disconnect"}
            </button>
            <button type="button" onClick={() => setConfirming(false)} disabled={busy !== null}>
              Keep
            </button>
          </>
        ) : (
          <button type="button" onClick={() => setConfirming(true)} disabled={busy !== null}>
            Disconnect…
          </button>
        )}
      </div>
    </li>
  );
}
