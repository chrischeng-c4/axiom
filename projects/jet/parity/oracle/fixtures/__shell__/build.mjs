// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-parity-oracle-fixtures-shell.md#logic
// CODEGEN-BEGIN
//
// Pre-bundles every `fixtures/*.tsx` into `fixtures/__shell__/dist/<name>.js`
// so the shell can lazy-load by name. Invoked manually (or via a future
// build.rs shim) — does not run automatically yet because the browser
// harness is gated on the #2139 follow-up.
//
// Usage:
//   node fixtures/__shell__/build.mjs
//
// Requires `esbuild` + `react` + `react-dom` + `@mui/material` installed
// in a sibling `node_modules` (CI image bootstrapped separately).

import { build } from "esbuild";
import { readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, basename } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const fixturesDir = join(here, "..");
const outdir = join(here, "dist");

const entries = readdirSync(fixturesDir)
  .filter((f) => f.endsWith(".tsx"))
  .map((f) => join(fixturesDir, f));

await build({
  entryPoints: entries,
  outdir,
  bundle: true,
  format: "esm",
  splitting: false,
  jsx: "automatic",
  target: ["chrome120"],
  loader: { ".tsx": "tsx", ".ts": "ts" },
  external: ["react", "react-dom"],
  entryNames: "[name]",
  metafile: false,
  sourcemap: false,
  minify: false,
});

console.log(`built ${entries.length} fixture(s) -> ${outdir}`);
// CODEGEN-END
