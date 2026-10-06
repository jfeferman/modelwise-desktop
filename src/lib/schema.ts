/**
 * What the `modelwise` command prints with --json, at schema 1, as the Rust
 * side hands it on. The source of truth is `src/output.ts` in @modelwise/cli;
 * the Rust side refuses any other schema number, and adds `health` and
 * `problems` to each connection (src-tauri/src/health.rs).
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

export type Health = "working" | "attention" | "broken";

export interface Status {
  schema: 1;
  connections: Connection[];
}
