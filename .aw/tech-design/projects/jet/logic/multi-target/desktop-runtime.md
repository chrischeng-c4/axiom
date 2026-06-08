---
id: projects-jet-logic-multi-target-desktop-runtime-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# jet desktop-runtime — Tauri-shell packaging for Jet apps

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/multi-target/desktop-runtime.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# jet desktop-runtime — Tauri-shell packaging for Jet apps

> Issue: #1242 — `enhancement(jet): add desktop target packaging
> through Tauri`. Sibling of #1238 (renderer-neutral contract),
> #1239 (`--target desktop` build plumbing), and #1241 (TUI
> renderer). Parent epic: #1237.

### Goal

Take the WASM bundle that `jet build --target desktop` already
produces (per `build-targets.md` Slice 2 + 3) and wrap it in a
Tauri shell so the same `Counter.tsx` source ships as a native
desktop app on macOS, Linux, and Windows. The Element / layout /
event contract from #1238 is **not** re-implemented — desktop
inherits the web profile in full (`target-profiles.md`
§"Desktop profile"). Divergence is limited to the OS shell:
window chrome, lifecycle pulses (suspend/resume), and the
local-backend bridge that replaces HTTP for in-process IPC.

The end-state demo (Slice 5) is the same Cue navigation /
list / detail / log workflow from #1240's acceptance criteria,
running inside a Tauri window on the host OS.

### Slices

The crate ships in five hand-written slices:

- **Slice 1 — spec + scaffold plan.** This document. Pins
  the public surface of `jet-tauri-shell`, the bridge contract
  with `jet-multi-target`'s `TargetRenderer`, and the
  packaging output recorded in `dist/jet-target.json`. No
  code lands in Slice 1; it locks down the architecture that
  Slices 2-5 implement.
- **Slice 2c (shipped) — OS-lifecycle types + in-process bus.**
  `lifecycle::LifecycleEvent` enums every desktop-only pulse the
  web profile cannot express (WindowCreated, WindowFocused,
  WindowBlurred, WindowMinimized, WindowRestored, WindowResized,
  WindowClosed, Suspended, Resumed). `WindowId(String)` wraps the
  Tauri window label so multi-window support drops in without an
  enum break (single-window apps tag everything `WindowId::main()`).
  `LifecycleListener: Send + Sync` is the subscriber trait;
  `LifecycleBus` (Arc/Mutex) collects listeners and fans out
  events in subscription order, cheaply clonable so the
  substrate can share a single bus across the Tauri event-loop
  thread and worker tasks. `TauriShell` carries a default bus,
  exposes `lifecycle()` (borrow) + `with_lifecycle_bus(...)`
  (swap), so callers can pre-register listeners before launch.
  9 new lifecycle unit tests + 2 shell-integration tests; no
  `tauri` crate dep yet (the Slice 2b Builder wiring will
  dispatch into [`LifecycleBus::publish`] from inside
  `tauri::Builder::on_window_event`).
- **Slice 2 (shipped) — crate scaffold + manifest validation.**
  `crates/jet-tauri-shell` lives in the workspace with NO hard
  `tauri` crate dependency yet. Public surface:
  `BundleManifest::{from_artifact_dir, from_path,
  validate_for_desktop}` deserializes `dist/jet-target.json` and
  refuses non-desktop / non-tauri / non-WASM / wrong-schema-
  version bundles loud-fast (8 typed `ManifestError` variants
  with full path context). `WindowConfig` ships with the spec'd
  defaults (1280x800, resizable, "Jet App") and chainable
  builders (`with_title`, `with_size`, `locked`). `TauriShell::
  from_artifact_dir(...)` validates eagerly and exposes
  `manifest()`, `window()`, `entry_html_path()` for the future
  Builder wiring. The bridge module ships `BackendBridge`
  (transport-agnostic IPC trait returning `BridgeFuture`) +
  `BridgeError` + an in-memory `MapBridge` test fixture. 18 unit
  tests cover parse / validate / defaults / builders / shell
  construction / bridge dispatch — all without spinning up a
  webview. Slice 2b lands the `tauri::Builder` wiring behind a
  feature flag once we pin a tauri minor version.
- **Slice 3a (shipped) — JSON-RPC wire envelope + transport-agnostic
  dispatcher.** `bridge::{RpcRequest, RpcResponse, RpcError}` lock
  down the on-wire JSON shape that BOTH the future
  `tauri::ipc::Invoke` adapter (Slice 3b) AND the future HTTP
  backend wrap. `RpcRequest { method, params }` decodes with
  `params` defaulting to `null` when the wire payload omits the
  field (matches JSON-RPC's no-params convention). `RpcResponse`
  is canonical: `Serialize` skips the unset half so the wire
  always carries exactly one of `{"result":...}` or
  `{"error":{"code":...,"message":...}}`. `BridgeError` →
  `RpcError` `From` impl maps the three variants to the reserved
  JSON-RPC-2.0 codes (`-32601` MethodNotFound, `-32602`
  InvalidParams, `-32603` Internal), exposed as `pub const`s so
  the future HTTP adapter and tests match on them without
  re-deriving. `dispatch_envelope(&dyn BackendBridge, RpcRequest)
  -> RpcResponse` is the shared async dispatcher — every transport
  owns only the wire-bytes codec; the dispatch shape is identical.
  10 new unit tests cover params-default-null, request round-trip,
  ok-omits-error / err-omits-result canonical serialization, the
  three error-code mappings, dispatcher ok+error paths, and a
  full canonical-shape assertion. (60 tests total in the crate.)
  This locks down the transport-compatibility contract from
  #1242's third AC ("the app code does not depend on browser-only
  globals or Tauri-only APIs"); Slice 3b plugs the
  `tauri::ipc::Invoke` adapter into this dispatcher behind a
  feature flag once Slice 2b pins the tauri minor version.
- **Slice 3b — `tauri::ipc::Invoke` adapter.** Behind the same
  feature flag as Slice 2b, wires `tauri::Builder::invoke_handler`
  to call `dispatch_envelope` with the decoded `RpcRequest`.
  No new wire shape; this slice only owns the tauri-side
  bytes-to-`RpcRequest` and `RpcResponse`-to-tauri-Response
  codec. The Cue concrete `BackendBridge` impl lands in #1240.
- **Slice 4a (shipped) — packager planning logic.** Pure
  function `packager::plan_package(&TauriShell, output_root) ->
  Result<PackagePlan, PackagePlanError>` (also exposed as
  `TauriShell::plan_package`). The plan describes:
  (a) `copies: Vec<FileCopy>` — three entries in stable order
  (HTML → WASM → boot loader) targeting
  `<output_root>/tauri-src/dist/`; (b) `command: PlannedCommand`
  — the `tauri build` invocation to run from
  `<output_root>/tauri-src`; (c) `output_dir` — per-OS
  `<output_root>/desktop/{macos,linux,windows}/` selected via
  `HostOs::current()` (overridable via `PackagePlan::for_host`
  for tests + cross-builds); (d) the carried `WindowConfig`.
  No filesystem mutation, no `tauri build` exec — those land
  in Slice 4b. 9 packager unit tests + 1 shell integration
  test cover stable copy order, per-OS output dirs, planned
  command shape, window-config passthrough, and the three
  missing-artifact-field error variants.
- **Slice 4b (shipped) — packager file-copy executor.**
  `execute_copies(&PackagePlan) -> Result<CopyReport,
  PackageError>` runs the filesystem half of Slice 4: pre-flight
  every `copy.source` exists (loud-fast on miss, no partial
  writes), `mkdir -p` each unique destination parent
  (deduplicated — three copies sharing `tauri-src/dist/` create
  one dir), then `fs::copy` in plan order. Returns a typed
  `CopyReport { files_copied, total_bytes, dirs_created }` so
  CI logs and the future `--dry-run` formatter can summarize
  without re-walking. `PackageError` enum wraps the three
  recoverable failure modes (`SourceMissing(PathBuf)`,
  `MkDir { path, source }`, `Copy { source_path, dest_path,
  source }`) — all `#[from]` `std::io::Error`. 5 new
  tempdir-backed unit tests cover three-copy success +
  byte-count math, idempotent re-runs (dirs_created → 0),
  loud-fast on missing source with no partial dest, parent-dir
  dedup, and `CopyReport::default`. (14 packager tests; 42
  total in the crate.) `tauri build` exec lands in Slice 4c
  once Slice 2b pins the tauri minor version.
- **Slice 4c (lib half shipped) — packager command executor +
  dry-run formatter.** `execute_command(&PackagePlan) ->
  Result<CommandReport, PackageError>` spawns
  `command.program command.args` from `command.working_dir` via
  `std::process::Command::status()`, returning a structural
  `CommandReport { program, args, working_dir }` on success.
  Two new failure variants on `PackageError`:
  `ExecSpawn { program, source: io::Error }` (OS refused to
  spawn — typically program-not-on-PATH) and
  `ExecFailed { program, status: ExitStatus }` (program ran but
  exited non-zero, status preserved verbatim). Executor is
  version-agnostic: it runs whatever `PlannedCommand` resolves
  to, so the spec's "Slice 2b pins the tauri minor version"
  gate is decoupled — the eventual `tauri` invocation can swap
  in once 2b lands without revisiting the executor. Sibling
  `format_dry_run(&PackagePlan) -> String` emits a stable
  human-readable transcript (`PackagePlan { ... }` block with
  source dirs / window / numbered copy list / final command
  line) for the future `--package --dry-run` flag. 8 new tests
  cover success path (echo + tempdir cwd), spawn-fail path
  (definitely-not-a-real-binary), exit-fail path (`#[cfg(unix)]`
  `false`), formatter stable layout (header / footer / field
  order), copies in plan order with 1-based indices, command +
  window line shape, empty-args branch (no double space), and
  `CommandReport::default`. (50 tests total in the crate.) The
  CLI wire-up — `jet build --target desktop --package` and
  `--package --dry-run` — lands in Slice 4d below.
- **Slice 4d (shipped) — CLI wire-up of the desktop packager.**
  `crates/jet/Cargo.toml` takes its first path-dep on
  `jet-tauri-shell`; `crates/jet/src/cli.rs` adds two flags to
  `jet build`: `--package` (requires `--wasm`) and
  `--package-dry-run` (requires `--package`). After
  `wasm_build::build_with_profile` returns Ok for
  `--target desktop --wasm`, the dispatcher (1) loads the
  emitted `<dist>/jet-target.json` via
  `TauriShell::from_artifact_dir(&dist)`, (2) computes
  `shell.plan_package(&dist)`, then either prints
  `format_dry_run(&plan)` to stdout and exits (if
  `--package-dry-run`) or runs `execute_copies` followed by
  `execute_command`, surfacing the per-stage report counts via
  stderr. `--package` against any non-desktop target is
  rejected loud-fast with `--package requires --target desktop
  (got --target <x>)`. The spec's "Slice 2b pins the tauri
  minor version" gate is unchanged — Slice 4d ships the wire,
  not the tauri binary; the executor runs whatever
  `PlannedCommand` resolves to (`tauri build` today; will keep
  working when 2b lands). All 859 `cargo test -p jet` tests
  still pass.
- **Slice 5 — Cue desktop demo.** The same Cue
  navigation/list/detail/status-timeline/log/command-input
  surface from #1240's acceptance criteria, running inside a
  Tauri window on the host OS. Adds the Cue-side
  `BackendBridge` impl that wraps the in-process Cue runtime
  for IPC, and the conformance harness assertion that the
  desktop bundle uses ZERO browser-only or Tauri-only
  globals at the app layer (per the issue's third
  acceptance criterion).

### Public surface (Slice 2 target)

```rust
pub struct TauriShell { /* private */ }
pub struct BundleManifest {
    pub artifact_dir: PathBuf,    // matches dist/jet-target.json `artifact.path`
    pub entry_html:    PathBuf,    // index.html relative to artifact_dir
    pub window:        WindowConfig,
}
pub struct WindowConfig {
    pub title:      String,        // default: "Jet App"
    pub width:      u32,           // default: 1280
    pub height:     u32,           // default: 800
    pub resizable:  bool,          // default: true
}
impl Default for WindowConfig { /* ... */ }

impl TauriShell {
    pub fn from_manifest(m: BundleManifest) -> Self;
    pub fn with_bridge<B: bridge::BackendBridge + 'static>(self, b: B) -> Self;
    pub fn launch(self) -> tauri::Result<()>;     // blocks; runs the Tauri event loop
}

pub mod bridge {
    pub trait BackendBridge: Send + Sync {
        fn handle(&self, method: &str, params: serde_json::Value)
            -> futures::future::BoxFuture<'_, Result<serde_json::Value, BridgeError>>;
    }
    pub enum BridgeError { NotFound, InvalidParams(String), Internal(anyhow::Error) }
}
```

The Tauri shell does NOT implement `TargetRenderer` itself.
Inside the webview the existing `WebRenderer` from
`crates/jet-multi-target/src/web.rs` is what paints — the
shell only owns the OS-level concerns the web profile cannot
express:

- Window lifecycle (created, focused, minimized, suspended,
  resumed, closed).
- Local-process IPC bridge (replaces HTTP `fetch()` calls
  from the same TSX source — see Slice 3).
- Native menu / tray / file-dialog plumbing (out of scope
  for the renderer-neutral contract; lands behind feature
  flags in Slice 4+ as the Cue demo needs them).

### Bundle manifest contract

`jet build --target desktop` produces a `dist/jet-target.json`
with this shape (from `crates/jet/src/wasm_build/manifest.rs`,
already shipped under #1239 Slice 3):

```json
{
  "schema_version": 1,
  "target":         "desktop",
  "profile_target": "web",
  "package_for":    "tauri",
  "artifact": {
    "kind":      "wasm",
    "wasm_path": "app_bg.wasm",
    "boot_path": "boot.js",
    "html_path": "index.html"
  },
  "build":  { "mode": "release", "rustc_target": "wasm32-unknown-unknown",
              "cargo_features": ["jet-multi-target/target-web",
                                 "jet-multi-target/target-desktop"] },
  "source": { "entry": "src/index.tsx", "root_component": "App",
              "jet_config_hash": "sha256:..." }
}
```

The Slice 2 `BundleManifest::validate_for_desktop` enforces:
`schema_version == 1`, `target == "desktop"`,
`package_for == Some("tauri")`, `artifact.kind == "wasm"`,
and `artifact.html_path` is `Some(...)`. Each failure returns
a typed `ManifestError` variant carrying the offending value
so packager errors are diagnosable from the message alone.
Slice 4's `package(...)` consumer concatenates
`artifact_dir + html_path` to produce the entry URI fed to
the Tauri webview.

### Cross-references

- `element-contract.md` — the `TargetRenderer` trait and the
  C1-C10 invariants the desktop profile inherits from web.
- `target-profiles.md` §"Desktop profile" — the desktop
  capability matrix (web in full + OS lifecycle additions).
- `build-targets.md` — `--target desktop` selection +
  cargo-feature wiring + the `dist/jet-target.json` shape.
- `tui-renderer.md` — sibling spec for the TUI target;
  shares the slice-by-slice authoring style.
- `crates/jet-multi-target/src/web.rs` — the `WebRenderer`
  the desktop target reuses verbatim inside the Tauri
  webview.

### Out of scope (Slice 1)

- Auto-update / code-signing / notarization. These are
  deployment concerns; Slice 4's `tauri build` step honors
  whatever signing config the host repo already has, but
  this spec adds none.
- Cross-target HMR. `jet-wasm-dev-server.md` already covers
  HMR over the web bundle; running that server behind the
  Tauri webview is a follow-up under #1250 (HMR
  invalidation) once that lands.
- A ratatui-flavored "desktop" embedding of the TUI
  renderer. Desktop = web profile + OS shell, period.
  Embedding TUI inside an OS window is a separate concern.
