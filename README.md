# Modelwise Desktop

A small app that sits in the macOS menu bar and shows whether Claude Code on this machine is
connected to Modelwise and sending its telemetry. Click the icon for each
connection's state; the icon itself changes when something needs attention.

It is a shell around the `modelwise` command ([`@modelwise/cli`](https://www.npmjs.com/package/@modelwise/cli)),
which it carries inside the app, so you need neither Node nor npm. Everything that signs in, edits
Claude Code's settings or uploads anything is that command's doing, and the terminal and the menu
bar can never disagree about what happens.

## What leaves your machine

Only what the command sends: token counts, timings, the model used, whether a call succeeded, and
the names of the tools, skills, subagents and MCP servers that ran. Never your prompts, Claude's
replies, file contents, commands or tool output. The app itself makes no network connections of its
own, and its panel cannot run programs, read files or reach the network: see
`src-tauri/capabilities/default.json`.

## Install

Download `Modelwise_<version>_aarch64.dmg` from the releases page, open it and drag Modelwise to
Applications. The app is not yet signed with an Apple Developer ID, so macOS will refuse to open it
at first; allow it once with

```bash
xattr -dr com.apple.quarantine /Applications/Modelwise.app
```

Connect Claude Code from a terminal with `npx @modelwise/cli connect --url <your Modelwise>`;
connecting from the app comes later.

## Build it yourself

You need Rust, [Bun](https://bun.sh) and Node 20 or later.

```bash
npm install
MODELWISE_CLI_BUNDLE=<path to dist/cli.js> npm run tauri build
```

`scripts/build-cli.mjs` compiles the `modelwise` command into one executable with Bun and checks it
prints the JSON version this app reads. Until the version of `@modelwise/cli` with `--json` is on npm,
the bundle comes from a checkout of its source; after that, `package.json` pins the version and the
variable is not needed. The app and the disk image land in `src-tauri/target/release/bundle/`.

For working on the panel, a debug build opens it at launch and keeps it open:

```bash
npm run tauri build -- --debug --bundles app
MODELWISE_DESKTOP_OPEN=1 src-tauri/target/debug/bundle/macos/Modelwise.app/Contents/MacOS/modelwise-desktop
```

## How it is put together

- `src-tauri/` is the Rust side: the tray icon (`tray.rs`), the embedded command and the one JSON
  version it accepts (`cli.rs`), what a connection's state means (`health.rs`), and the commands the
  panel may call (`commands.rs`).
- `src/` is the panel, in React. It exists only while it is open: the window is created when the
  icon is clicked and destroyed when it loses focus, so no web view is alive while nobody is looking.
- The panel never sees the token. The command never prints it, and the Rust side passes documents
  through as they are.

## License

MIT
