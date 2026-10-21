---
name: panel-command
description: How to add or change a command the panel can call (Rust command, handler registration, TypeScript wrapper, result type). Load when editing src-tauri/src/commands.rs, src-tauri/src/lib.rs, src/lib/cli.ts or src/lib/schema.ts.
---

Every panel action is a fixed `modelwise` invocation. The panel never chooses a program or an argument.

Adding a command touches four places:

1. `src-tauri/src/commands.rs`: add a `#[tauri::command] pub async fn` with a doc comment. Run the CLI as `tauri::async_runtime::spawn_blocking(move || cli::run(&[...]))`. Map a join error with `.map_err(|error| error.to_string())?`. Return `Result<Value, String>`. A command that does not call the CLI (`set_paused`, `set_autostart`, `resize_panel`) is a plain `pub fn` returning `Result<(), String>`.
2. `src-tauri/src/lib.rs`: add `commands::<name>` to `tauri::generate_handler![...]`. There is no other registry.
3. `src/lib/cli.ts`: add a wrapper. Which kind depends on what the Rust command returns:
   - `Result<Value, String>` from `cli::run`: `invoke<Result>("snake_case_name", { camelCaseArgs })`, returning `Promise<Result>`.
   - `Result<(), String>` (`set_paused`, `set_autostart`, `resize_panel`): untyped `invoke("snake_case_name", { camelCaseArgs })`, returning `Promise<void>`.
   - A streaming command (`connect`): an `async` function that takes an `onEvent` callback, calls `listen<Event>("<event name>", …)` before the untyped `invoke`, and unlistens in `finally`. It resolves with `void` when the command ends.
4. `src/lib/schema.ts`: add a type only when the wrapper needs one:
   - For a `cli::run` command, a result interface with `schema: 1`. `cli.rs` refuses any other schema number.
   - For a streaming command, a union of event objects discriminated on `event` (see `ConnectEvent`). Events carry no `schema` field.
   - Commands returning `Result<(), String>` get no type here.

Rules for values the panel supplies:

- A config folder (`repair`, `disconnect`) must go through `known(&app, &config_dir)?` before it reaches `cli::run`. Those two commands also pass `--yes`.
- A URL (`connect`) is trimmed and checked for an http(s) scheme and no whitespace. It is also the only streaming command: it uses `cli::run_streaming` and emits `connect` events.
- `status`, `sync_now` and `usage` take no panel values.
- `set_paused`, `set_autostart` and `resize_panel` do not call the CLI at all.
- Do not add permissions to `src-tauri/capabilities/default.json`. The panel gets only event listen and unlisten.
