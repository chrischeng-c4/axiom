# build fails loudly on unresolved bare specifiers

> **Issue**: #1317
> **Crate**: `jet` (`projects/jet/src/bundler/mod.rs`)
> **Type**: bug

## Problem

`jet build` exits `0` even when imports cannot be resolved at bundle
time. From `projects/cue/fe` (2026-05-08):

```text
$ cargo run -p jet -- build -o ../be/static
Build complete in 132ms: ../be/static/main.297aaa65.js (… KB)

$ node --check ../be/static/main.297aaa65.js
SyntaxError: Unexpected token 'const'
```

Cue's `package.json` declares `react`, `react-dom`, and `lucide-react`,
but they are not present in `node_modules` (or the resolver does not
find them via the configured conditions). The emitted bundle is full
of runtime shim calls referencing the missing packages:

```js
e('react/jsx-runtime'), e('react'), e('react-dom/client'), e('lucide-react')
```

The current code path:

- `projects/jet/src/bundler/mod.rs:412-444` — static-import branch of
  `build_graph`. When `resolve_dependency` returns
  `Err("Cannot resolve package: react")`, the error is downgraded to a
  `tracing::warn!` and the loop continues.
- `projects/jet/src/bundler/mod.rs:446-472` — dynamic-import branch.
  Same swallow.
- `projects/jet/src/bundler/mod.rs:398-410` — implicit
  `react/jsx-runtime` JSX dep. Same swallow.

The downstream transform still emits `require(...)` / `import(...)`
calls referencing the missing module ids, which fail at runtime — but
only after the user has already shipped the artifact.

## Scope

In:

- Aggregate the **non-external** resolution failures in
  `build_graph` into a typed `UnresolvedDependency` list on the
  `Bundler`. "Non-external" = the resolver did not return
  `is_external = true`; the error message does **not** contain
  `External module:`.
- After `build_graph` returns, if any unresolved deps were collected,
  return an `anyhow::Error` from `Bundler::bundle()` listing every
  unresolved specifier together with the importer paths. The
  existing `.context("Bundle failed")` wrap at
  `projects/jet/src/cli.rs:1235` propagates this through to a non-zero
  exit code (anyhow's standard surface).
- Preserve the **expected** "External module: X" silent-skip for
  bundlers running with `externalize_all_packages = true` (lib mode,
  `cli.rs:1784`) or an explicit `externals` set. Those are the
  user's deliberate opt-in.
- Add unit-test coverage that pins:
  - Unresolved bare specifier → typed error, never a successful
    `BundleOutput`.
  - `externalize_all_packages = true` skips the new error path.
  - Explicit `externals` set skips the new error path.

Out:

- The sibling adjacent-declaration concat symptom
  (`const _m0_seedSpec=c const defaultPrompt=...`). That is a
  separate bundler-codegen bug in the Phase 2 flatten path; closing
  this issue's "exits 0" half does not require fixing the codegen
  half. Tracked separately.
- Actually resolving / bundling the missing npm packages. That is
  the larger "make Cue build" enhancement; this issue is purely
  about the silent-success symptom.
- Surfacing dynamic-import unresolved deps as warnings vs errors.
  Treat them the same as static imports for now — both promise
  symbols at runtime that don't exist.

## Interface

```rust
/// A bare-specifier import that the resolver could not find on disk
/// and that the user did not explicitly mark as external.
#[derive(Debug, Clone)]
struct UnresolvedDependency {
    /// The bare specifier as written in the source (e.g. "react").
    specifier: String,
    /// The module that tried to import it.
    importer: PathBuf,
    /// The resolver's error message, preserved for diagnostics.
    reason: String,
}

pub struct Bundler {
    // … existing fields …
    /// Collected during `build_graph`; drained into the typed error
    /// returned by `bundle()` if non-empty. Anchor: #1317.
    unresolved_deps: parking_lot::Mutex<Vec<UnresolvedDependency>>,
}
```

The error returned from `bundle()` has the form:

```text
Unresolved imports — `jet build` cannot continue. Resolve these
specifiers (install the missing package, fix the import path, or
mark the specifier as external via [build].externals) and re-run:
  • `react`           imported from src/CueWasmApp.tsx
  • `react-dom/client` imported from src/main.tsx
  • `lucide-react`     imported from src/components/Toolbar.tsx
See: https://github.com/.../issues/1317
```

## Acceptance Criteria

- [x] `Bundler::bundle()` returns `Err` when any static / dynamic /
      implicit-JSX import fails to resolve and the importer did not
      opt in to externalizing the specifier.
- [x] The error message lists every unresolved specifier together
      with the importer path (deduplicated by specifier, in stable
      lexical order).
- [x] `externalize_all_packages = true` (lib mode) continues to
      silently skip external bare specifiers — no false positive.
- [x] An explicit `externals = ["react"]` set continues to silently
      skip those specifiers — no false positive.
- [x] `cargo test -p jet --lib bundler::unresolved_deps_tests`
      passes.
- [x] `jet build` against a fixture importing a missing bare
      specifier exits non-zero (CI no longer ships invalid JS).

## Reference Context

- `projects/jet/src/bundler/mod.rs:412-444` — static-import swallow.
- `projects/jet/src/bundler/mod.rs:446-472` — dynamic-import swallow.
- `projects/jet/src/bundler/mod.rs:398-410` — JSX implicit-runtime
  swallow.
- `projects/jet/src/bundler/mod.rs:491-509` — `resolve_dependency`
  returns "External module: X" only when the resolver flags it
  external; otherwise it propagates the resolver's own error
  message ("Cannot resolve package: X").
- `projects/jet/src/resolver/mod.rs:107-128` — resolver
  `is_external` short-circuit and the normal-path bail.
- `projects/jet/src/cli.rs:1230-1235` — `bundler.bundle()` call site
  with `.context("Bundle failed")`; this is the propagation point
  that turns an `Err` into a non-zero process exit.
- `projects/jet/src/cli.rs:1779-1786` — lib mode flips
  `externalize_all_packages` on; the new error path must not
  trigger here.
