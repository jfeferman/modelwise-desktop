#!/usr/bin/env node
/**
 * Compiles the `modelwise` command into one executable the app carries, so
 * nobody needs Node or npm to run it:
 *
 *   src-tauri/binaries/modelwise-<target triple>
 *
 * The source is the published bundle, node_modules/@modelwise/cli/dist/cli.js,
 * at the version package.json pins. MODELWISE_CLI_BUNDLE names another bundle
 * to use instead, for working on the command and the app together.
 *
 * Needs Bun (https://bun.sh) and Rust on the PATH.
 */
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const bundle = process.env.MODELWISE_CLI_BUNDLE
  ? resolve(process.env.MODELWISE_CLI_BUNDLE)
  : resolve(root, "node_modules/@modelwise/cli/dist/cli.js");

if (!existsSync(bundle)) {
  console.error(
    `No modelwise bundle at ${bundle}.\n` +
      "Install the pinned @modelwise/cli, or set MODELWISE_CLI_BUNDLE to a dist/cli.js built from its source."
  );
  process.exit(1);
}

// The app reads one version of the command's JSON (src-tauri/src/cli.rs), so
// a bundle from before --json, or from after the next change, must not be
// embedded by mistake.
const SCHEMA = Number(/pub const SCHEMA: u64 = (\d+);/.exec(readFileSync(resolve(root, "src-tauri/src/cli.rs"), "utf8"))?.[1]);
const printed = execFileSync(process.execPath, [bundle, "status", "--json"], {
  encoding: "utf8",
  env: { ...process.env, MODELWISE_CONNECTIONS: join(mkdtempSync(join(tmpdir(), "modelwise-cli-")), "none.json") }
});
let schema;
try {
  schema = JSON.parse(printed.trim().split("\n").at(-1) ?? "").schema;
} catch {
  // A bundle from before --json prints prose.
}

if (schema !== SCHEMA) {
  console.error(`${bundle} prints ${schema ? `JSON schema ${schema}` : "no JSON"}; this app reads schema ${SCHEMA}.`);
  process.exit(1);
}

// Tauri looks for an embedded executable under the name of the platform it is building for.
const triple = /^host: (.+)$/m.exec(execFileSync("rustc", ["-vV"], { encoding: "utf8" }))?.[1];

if (!triple) {
  console.error("Could not read the target triple from `rustc -vV`.");
  process.exit(1);
}

const out = resolve(root, "src-tauri/binaries", `modelwise-${triple}${process.platform === "win32" ? ".exe" : ""}`);
mkdirSync(dirname(out), { recursive: true });
// From the bundle's own directory: Bun leaves its temporary file where it runs.
execFileSync("bun", ["build", "--compile", bundle, "--outfile", out], { cwd: dirname(out), stdio: "inherit" });

// Bun leaves a copy of its runtime behind.
for (const name of readdirSync(dirname(out))) {
  if (name.endsWith(".bun-build")) {
    rmSync(resolve(dirname(out), name), { force: true });
  }
}

console.log(`${out}\n  from ${bundle}`);
