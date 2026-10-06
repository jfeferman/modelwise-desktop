import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ConnectEvent, DisconnectResult, RepairResult, Status, SyncResult, UsageResult } from "./schema";

/** Each of these is one fixed `modelwise` invocation on the Rust side. All reject with a message to show. */

export function status(): Promise<Status> {
  return invoke<Status>("status");
}

export function syncNow(): Promise<SyncResult> {
  return invoke<SyncResult>("sync_now");
}

export function repair(configDir: string): Promise<RepairResult> {
  return invoke<RepairResult>("repair", { configDir });
}

export function disconnect(configDir: string): Promise<DisconnectResult> {
  return invoke<DisconnectResult>("disconnect", { configDir });
}

export function usage(): Promise<UsageResult> {
  return invoke<UsageResult>("usage");
}

export function setPaused(paused: boolean): Promise<void> {
  return invoke("set_paused", { paused });
}

export function setAutostart(enabled: boolean): Promise<void> {
  return invoke("set_autostart", { enabled });
}

/** Signs in to `url`, reporting each step. Resolves when the command ends, well or badly. */
export async function connect(url: string, onEvent: (event: ConnectEvent) => void): Promise<void> {
  let stop: UnlistenFn | null = await listen<ConnectEvent>("connect", (received) => onEvent(received.payload));

  try {
    await invoke("connect", { url });
  } finally {
    stop();
    stop = null;
  }
}
