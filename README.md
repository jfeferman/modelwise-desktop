# Modelwise Desktop

A small app that sits in the macOS menu bar and shows whether Claude Code on this machine is
connected to Modelwise and sending its telemetry. Click the icon for each connection's state; the
icon itself changes when something needs attention. From the panel you can connect (you sign in
through your browser), sync now, repair Claude Code's settings and disconnect, and past sessions
are uploaded in the background every hour unless you pause it.

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

Then click the icon and enter your Modelwise's address to connect. The same connection can be made
from a terminal with `npx @modelwise/cli connect --url <your Modelwise>`; the app and the command
share it.

## Build it yourself

You need Rust, [Bun](https://bun.sh) and Node 20 or later.

```bash
npm install
npm run tauri build
npm run dmg
```

`scripts/build-cli.mjs` compiles the `modelwise` command into one executable with Bun, from the
`@modelwise/cli` version `package.json` pins, and checks it prints the JSON version this app reads.
To build against a command you are working on instead, point `MODELWISE_CLI_BUNDLE` at its
`dist/cli.js`. `scripts/make-dmg.sh` puts the app and a link to Applications on a disk image, by
mounting a blank one rather than with `hdiutil create -srcfolder`, which fails on some machines.
The app and the disk image land in `src-tauri/target/release/bundle/`.

For working on the panel, a debug build opens it at launch and keeps it open:

```bash
npm run tauri build -- --debug --bundles app
MODELWISE_DESKTOP_OPEN=1 src-tauri/target/debug/bundle/macos/Modelwise.app/Contents/MacOS/modelwise-desktop
```

## How it is put together

- `src-tauri/` is the Rust side: the tray icon (`tray.rs`), the embedded command and the one JSON
  version it accepts (`cli.rs`), what a connection's state means (`health.rs`), the commands the
  panel may call (`commands.rs`), the hourly background sync (`scheduler.rs`) and the app's own
  settings (`settings.rs`).
- The panel supplies two kinds of value to those commands: a Modelwise address to connect to, and a
  configuration folder, which must be one the command itself listed. It cannot name a program or an
  argument.
- `src/` is the panel, in React. It exists only while it is open: the window is created when the
  icon is clicked and destroyed when it loses focus, so no web view is alive while nobody is looking.
- The panel never sees the token. The command never prints it, and the Rust side passes documents
  through as they are.

## License

MIT
