import { useState, type FormEvent } from "react";
import { connect } from "../../lib/cli";
import type { ConnectEvent } from "../../lib/schema";

type Progress =
  | { phase: "idle" }
  | { phase: "starting" }
  | { phase: "waiting"; userCode: string; verificationUrl: string }
  | { phase: "finishing"; userCode: string }
  | { phase: "failed"; message: string };

/** Sign in to a Modelwise. The command opens the browser; the person approves there. */
export function ConnectForm({ onConnected }: { onConnected: () => void }) {
  const [url, setUrl] = useState("");
  const [progress, setProgress] = useState<Progress>({ phase: "idle" });
  const busy = progress.phase === "starting" || progress.phase === "waiting" || progress.phase === "finishing";

  async function submit(event: FormEvent) {
    event.preventDefault();
    setProgress({ phase: "starting" });

    try {
      await connect(url, (step: ConnectEvent) => {
        if (step.event === "started") {
          setProgress({ phase: "waiting", userCode: step.userCode, verificationUrl: step.verificationUrl });
        } else if (step.event === "approved") {
          setProgress((current) => ({ phase: "finishing", userCode: current.phase === "waiting" ? current.userCode : "" }));
        }
      });
      setProgress({ phase: "idle" });
      setUrl("");
      onConnected();
    } catch (error) {
      setProgress({ phase: "failed", message: String(error) });
    }
  }

  return (
    <form className="connect" onSubmit={submit}>
      <p className="hint">
        Connect Claude Code on this machine. You sign in through your browser; the app then sets Claude Code up to
        send its traces and uploads your past sessions. Only counts and timings leave this machine.
      </p>
      <div className="connect-row">
        <input
          type="url"
          value={url}
          onChange={(event) => setUrl(event.target.value)}
          placeholder="https://your-modelwise.example"
          required
          disabled={busy}
          spellCheck={false}
        />
        <button type="submit" disabled={busy || url.trim() === ""}>
          {busy ? "Connecting…" : "Connect"}
        </button>
      </div>
      {progress.phase === "waiting" && (
        <p className="hint">
          Check that your browser shows the code <strong className="code">{progress.userCode}</strong> and approve.
          If it did not open, go to <code>{progress.verificationUrl}</code>.
        </p>
      )}
      {progress.phase === "finishing" && <p className="hint">Approved. Setting Claude Code up and uploading past sessions…</p>}
      {progress.phase === "failed" && <p className="problem problem-broken">{progress.message}</p>}
    </form>
  );
}
