import { useEffect, useState } from "react";
import { usage } from "../../lib/cli";
import type { OwnUsage } from "../../lib/schema";

type State = { phase: "loading" } | { phase: "ready"; usage: OwnUsage } | { phase: "failed"; message: string };

/** Your own usage over the last day, as Modelwise counts it. Only yours: nobody else's is shown. */
export function UsageLine({ configDir }: { configDir: string }) {
  const [state, setState] = useState<State>({ phase: "loading" });

  useEffect(() => {
    let current = true;
    usage().then(
      (result) => {
        const mine = result.usage.find((row) => row.configDir === configDir);
        if (!current) return;
        if (mine?.usage) setState({ phase: "ready", usage: mine.usage });
        else setState({ phase: "failed", message: mine?.error ?? "No usage reported." });
      },
      (error) => current && setState({ phase: "failed", message: String(error) })
    );
    return () => {
      current = false;
    };
  }, [configDir]);

  if (state.phase === "loading") {
    return <div className="connection-detail">Fetching your usage…</div>;
  }

  if (state.phase === "failed") {
    return <div className="connection-detail" title={state.message}>Usage unavailable</div>;
  }

  const { tokens, invocations, meteredSpend } = state.usage;
  const calls = `${invocations.toLocaleString()} ${invocations === 1 ? "call" : "calls"}`;
  const money = meteredSpend > 0 ? `, $${meteredSpend.toFixed(2)} metered` : "";

  return (
    <div className="connection-detail">
      Last 24 hours: {tokens.toLocaleString()} tokens in {calls}
      {money}
    </div>
  );
}
