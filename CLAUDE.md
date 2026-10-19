# Working in modelwise-desktop

A macOS menu bar app: a tray icon, a panel that exists only while it is open, and the
`modelwise` command embedded in the bundle. The command does everything that matters
(sign in, edit Claude Code's settings, upload sessions); the app only runs it and shows
what it printed. Read README.md first for what the app is and how it is built.

## Commands

```bash
npm install
npm run typecheck                                   # the panel (src/)
cargo test --manifest-path src-tauri/Cargo.toml     # the Rust side; the only tests there are
npm run cli                                         # compile the embedded command; needs Bun and Rust
npm run tauri build                                 # the app, in src-tauri/target/release/bundle/
npm run dmg                                         # the disk image, from that app
```

The panel has no tests. Verify a panel change by running a debug build with the panel
held open:

```bash
npm run tauri build -- --debug --bundles app
MODELWISE_DESKTOP_OPEN=1 src-tauri/target/debug/bundle/macos/Modelwise.app/Contents/MacOS/modelwise-desktop
```

`MODELWISE_DESKTOP_CLICK=1` instead opens it as a click would, blur-to-close included.
Both are ignored by a release build. Vite serves on 1420 and nothing else, because
`tauri.conf.json` names that address.

## Invariants every change keeps

- **The panel never chooses what runs.** `src-tauri/src/commands.rs` is a fixed list of
  `modelwise` invocations. The only values the panel supplies are a Modelwise address,
  validated there, and a configuration folder, which must be one the last `status`
  listed (`AppState.known`). A new panel action is a new function in `commands.rs`,
  registered in `lib.rs`, with a wrapper in `src/lib/cli.ts` and its shape in
  `src/lib/schema.ts`. Never a parameter that names a program, an argument or a path.
- **`capabilities/default.json` stays as it is**: events only. No shell, file, HTTP or
  dialog permission, whatever a feature would find convenient. The README promises this.
- **The app makes no network connection of its own.** Anything that must reach a server
  goes through the command, which is the one place a token exists.
- **The token never reaches the panel.** The Rust side passes the command's documents
  through unchanged, apart from `health`, `problems` and `app`. Do not log a document,
  and do not add a field the command does not print.
- **One JSON schema.** `SCHEMA` in `src-tauri/src/cli.rs` is the version of the
  command's `--json` output the app reads, and the command is called with `--json`
  always. When the command's shape changes, `@modelwise/cli` raises its number; this app
  then raises `SCHEMA`, updates `src/lib/schema.ts` to match `src/output.ts` in the
  command's source, and pins the new version in `package.json`. `scripts/build-cli.mjs`
  refuses to embed a command that prints any other number.
- **Health is decided once**, in `src-tauri/src/health.rs`, because the tray icon needs
  it with no panel open. The panel shows `health` and `problems`; it does not judge a
  connection itself. No connection at all is `working`.
- **The panel is a window that is created on click and destroyed on blur.** Hold it
  open with `tray::hold_panel` for anything that takes the focus away, as `connect`
  does for the browser. Closing it is not quitting: `ExitRequested` with no code is
  prevented in `lib.rs`.
- **Closing the panel must not reopen it.** A click on the icon while the panel is open
  blurs it first; `REOPEN_GUARD` in `tray.rs` swallows that click.

## Traps

- After `tray.set_icon(...)`, call `set_icon_as_template(true)` again, or the icon turns
  black: a new image is not a template until said so.
- `src-tauri/binaries/` and `src-tauri/gen/` are build output and ignored. The embedded
  command must be named `modelwise-<target triple>`; Tauri finds it by that name.
- The panel's CSP allows no external resource. Styles are inline-capable; scripts and
  fonts come from the bundle only.
- The background thread checks every 15 minutes and syncs on every fourth check unless
  paused (`scheduler.rs`); pausing stops sync, never checks.
- The app's own settings live in the app's configuration folder (`settings.rs`), apart
  from the command's `connections.json`. Do not read or write the command's files here.

## Working on the command and the app together

The command's source is `packages/scanner` in the modelwise repository, published as
`@modelwise/cli`. Build its `dist/cli.js` there and point `MODELWISE_CLI_BUNDLE` at it
before `npm run cli`; the schema check still applies.

## Style

- Comments state an invariant or a trap, never what the code does or why an
  alternative was rejected. Module docs (`//!`) say what the file is for in a sentence.
- The panel's text is plain English in the JSX; there is no translation layer here.
- Commit messages say what changed, in the imperative, as the history does.
