/**
 * What the `modelwise` command prints with --json, at schema 1, as the Rust
 * side hands it on. The source of truth is `src/output.ts` in @modelwise/cli;
 * the Rust side refuses any other schema number, and adds `health` and
 * `problems` to each connection (src-tauri/src/health.rs) and `app` to the
 * status.
 */
export interface LastSync {
  at: string;
  sent: number;
  error: string | null;
}

export interface Check {
  settings: "ok" | "missing" | "drifted" | "unreadable";
  server: "reachable" | "unreachable";
  token: "live" | "revoked" | "unknown";
}

export type Health = "working" | "attention" | "broken";

export interface Connection {
  configDir: string;
  url: string;
  name: string;
  syncedThrough: string | null;
  lastSync: LastSync | null;
  check?: Check;
  health: Health;
  /** What is wrong, most serious first. Empty when working. */
  problems: string[];
}

export interface Status {
  schema: 1;
  connections: Connection[];
  app: { paused: boolean };
}

export interface SyncResult {
  schema: 1;
  results: { configDir: string; dryRun: boolean; transcripts: number; sent: number; through: string | null; error: string | null }[];
}

export type SettingsOutcome = "unchanged" | "written" | "not-written";

export interface RepairResult {
  schema: 1;
  repaired: { configDir: string; path: string; settings: SettingsOutcome }[];
}

export interface DisconnectResult {
  schema: 1;
  disconnected: { configDir: string; url: string; name: string };
  settings: SettingsOutcome;
  tokenRevoked: boolean;
}

/** One line of `connect --json`, as it happens. */
export type ConnectEvent =
  | { event: "started"; configDir: string; name: string; verificationUrl: string; userCode: string; expiresIn: number }
  | { event: "approved" }
  | { event: "saved"; connectionsFile: string }
  | { event: "settings"; path: string; result: SettingsOutcome }
  | { event: "synced"; transcripts: number; sent: number; through: string | null; error: string | null }
  | { event: "done"; usageUrl: string };
