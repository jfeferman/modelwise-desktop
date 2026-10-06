import { invoke } from "@tauri-apps/api/core";
import type { Status } from "./schema";

/** Every connection, and whether each one still works. Rejects with a message to show. */
export function status(): Promise<Status> {
  return invoke<Status>("status");
}
