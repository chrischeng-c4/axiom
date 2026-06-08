---
id: projects-jet-logic-multi-target-build-targets-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet build targets — CLI + manifest

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/multi-target/build-targets.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet build targets — CLI + manifest

> Issue: #1239 — `enhancement(jet): add build target plumbing for web
> desktop and TUI`. Sibling of #1238 (the renderer-neutral contract
> this build flag *selects against*).

### Scope

`jet build` learns a single `--target` flag whose argument is the
canonical target name from `target-profiles.yaml`. The build pipeline:

1. Validates the chosen target against the registered profiles.
2. Maps it onto the corresponding cargo feature on `jet-multi-target`
   (`target-web` / `target-desktop` / `target-tui`) so `use_target()`
   returns the matching profile at runtime (H3 from
   `target-profiles.md`).
3. Emits a `dist/jet-target.json` manifest that downstream packagers
   (Tauri shim for desktop, ratatui launcher for TUI) consume.
4. Rejects unsupported `(target, mode)` combinations with an
   actionable error before any expensive build step runs.

This spec defines the CLI surface, the manifest schema, and the
validation table. The implementation lands as four hand-written
slices:

- **Slice 1 (this issue's foundation):** `--target` CLI flag + typed
  enum + early-fail validation. No wiring to the bundler yet.
- **Slice 2:** wire the flag into `wasm_build::build_with_profile`
  so `--target web --wasm` flips on `jet-multi-target/target-web` +
  `jet-multi-target/web` features when invoking cargo.
- **Slice 3 (shipped):** emit `dist/jet-target.json` from each successful
  build; downstream packagers (#1241 / #1242) consume the documented
  schema. Implementation lives in `crates/jet/src/wasm_build/manifest.rs`.
- **Slice 4 (shipped):** unsupported-combo validation table + tests.
  Eleven tests in `crates/jet/src/cli.rs::build_target_validation_table_tests`
  exercise every row of the table below through the real `cli::command()`
  parser. With Slice 4 landed, #1239 is feature-complete; what remains is
  consumer wiring (#1241 TUI, #1242 Tauri).

### CLI surface

```text
jet build --target {web|desktop|tui} [--wasm] [--watch] [-o dir] ...
```

| Field | Behavior |
|-------|----------|
| `--target` | Optional; when omitted, defaults to `web` (the same default `jet-multi-target` ships with). Closed enum — any value outside `{web,desktop,tui}` is a hard parse error. |
| `--target web` | Builds the canvas/WASM artifact (TSX → Rust → wasm32). Requires `--wasm`; `jet build --target web` without `--wasm` falls through to the legacy bundler so existing pipelines are unbroken. |
| `--target desktop` | Builds the WASM artifact AND records `package_for: tauri` in the manifest. Slice 1 only validates; Slice 4 wires the Tauri shim from #1242. |
| `--target tui` | Builds a native binary (no WASM); records `package_for: ratatui`. Slice 1 only validates; #1241 wires the ratatui renderer. |

### Mutually-exclusive vs. bundler flags

- `--target tui` is **incompatible** with `--wasm`. Surface as:
  `error: --target tui builds a native binary; drop --wasm`.
- `--target tui` is incompatible with `--minify`, `--sourcemap`,
  `--splitting`, `--drop` (those are JS-bundle concerns). Each rejected
  flag names which target rejects it.
- `--target {web,desktop}` accepts the full bundler flag set.

### Manifest schema

Emit `<output>/jet-target.json` on every successful build. Single
small JSON file consumed by downstream packagers and CI dashboards.

```json
{
  "schema_version": 1,
  "target": "web",
  "profile_target": "web",
  "package_for": null,
  "artifact": {
    "kind": "wasm",
    "wasm_path": "app.wasm",
    "boot_path": "boot.js",
    "html_path": "index.html"
  },
  "build": {
    "mode": "release",
    "rustc_target": "wasm32-unknown-unknown",
    "cargo_features": ["jet-multi-target/target-web", "jet-multi-target/web"]
  },
  "source": {
    "entry": "src/index.tsx",
    "root_component": "App",
    "jet_config_hash": "sha256:..."
  }
}
```

| Field | Source |
|-------|--------|
| `schema_version` | Hard-coded `1`; bumped on incompatible changes. |
| `target` | Echo of `--target`. |
| `profile_target` | Resolves via the `target-profiles.yaml` `inherits:` chain — for `desktop` this is `web`. Lets a packager skip the chain walk. |
| `package_for` | One of `null` (web), `tauri` (desktop), `ratatui` (TUI). Hint to downstream tooling, not a directive. |
| `artifact.kind` | `wasm` for web/desktop; `native-bin` for TUI. |
| `artifact.*_path` | Relative paths under `<output>/`. Absent fields are simply omitted. |
| `build.cargo_features` | The exact feature set passed to cargo so a CI verifier can reproduce the build. |
| `source.jet_config_hash` | SHA-256 of `jet.config.toml` to make manifest diffs round-trip-able. |

### Validation table

| Case | Outcome |
|------|---------|
| No `--target` | Default to `web`. Print a single `info: target=web` line so logs are unambiguous. |
| `--target tui --wasm` | Error: `tui builds a native binary; drop --wasm`. |
| `--target web` (without `--wasm`) | Fall through to legacy bundler; record `target: web` in the manifest emitted by the bundler exit path. |
| `--target tui --minify` | Error: `--minify is a JS-bundle flag; not valid for --target tui`. |
| Multiple `--target` | Clap rejects (single-value arg). |
| Unknown `--target X` | Clap rejects with the closed-enum list. |

Tests cover each row of this table with `assert_cmd` against the CLI.

### Out of scope

- Cargo feature wiring inside `wasm_build` — Slice 2.
- Manifest emission — Slice 3 (this slice only validates the flag).
- Tauri / ratatui packagers — #1241 / #1242.
- Multi-target bundle in a single invocation (e.g. `--target web,desktop`)
  — discussed once Slice 4 ships and we have telemetry on real usage.

### Cross-references

- `element-contract.md` — the renderer trait the chosen target must implement.
- `target-profiles.md` — the capability matrix the manifest's `profile_target` field points into.
- Issue #1238 — landed; provides the `target-*` cargo features this flag selects.
- Issue #1241 — TUI renderer; consumer of `--target tui`.
- Issue #1242 — Tauri packaging; consumer of `--target desktop`.
