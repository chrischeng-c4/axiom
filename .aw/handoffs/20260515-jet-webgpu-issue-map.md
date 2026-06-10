---
topic: jet-webgpu-issue-map
date: '20260515'
project: project-jet
branch: project-jet
---

## Status

- `project-jet` is aligned with `origin/project-jet`; WebGPU wasm foundation is pushed through `3603f3cd6`, and the only local untracked file is this handoff.
- Current focus: continue #1254 `epic(jet): WasmView renderer and cclab-grid migration` via real WebGPU text pass integration.
- Estimated state: WebGPU wasm foundation/bridge is about 60-70% of MVP; renderer parity is still about 35-45% because text, clip, stroke, resize, and pixel e2e remain.

## Findings

- `score wi epicize --project jet --json` works; CLI spelling is `epicize`, not `epicise`.
- `score wi update <github-number> --push` returned `{"error":"issue '<id>' not found","code":"NOT_FOUND"}` for remote numeric issue ids in this checkout, so GitHub metadata updates were applied with `gh issue edit`.
- `score wi list --backend github` is not supported by this CLI version; use `score wi list --label project:jet --json` or `gh issue list --label project:jet ...`.
- Active open `project:jet` issue inventory after refresh: 23 open issues, including two new parent epics #2114 and #2115.
- WebGPU bridge status:
  - Jet now preserves text runs and passes them to `renderFrameWithText(cells, textRuns)`.
  - `cclab-grid-wasm` validates and records text-run count, then falls back to the current cell-only render path.
  - Real glyph/text drawing is not integrated into `cclab-grid-render-webgpu` yet.
- Hard technical blocker for next slice: lower renderer currently exposes a cell-oriented frame path; `GlyphInstance`/atlas/text primitives exist but need an integrated text pass API through `FrameBuilder`/`WebGpuRenderer`/wasm bridge.
- #1314 remains `needs-clarify` and intentionally was not grouped under any active epic because it is cross-labeled `project:cue` and `project:jet`.

## Done

- Tested and pushed WebGPU wasm implementation commits:
  - `d37284b5d feat(jet): add wasm webgpu paint backend`
  - `93c177844 feat(jet): pack webgpu frame plans`
  - `abadea3b5 feat(jet): add wasm webgpu app bridge`
  - `4cb5710d1 feat(jet): select webgpu wasm renderer`
  - `8d837ed76 test(jet): smoke webgpu wasm scaffold`
  - `faf48a639 feat(jet): expose webgpu runtime status`
  - `adf1996b4 test(jet): smoke webgpu runtime status`
  - `df5b2e779 feat(jet): preserve webgpu text runs`
  - `6d2e3187c feat(jet): add webgpu text bridge wire`
  - `3603f3cd6 test(jet): assert webgpu text bridge path`
- Done and tested: `projects/jet/wasm/src/renderer/webgpu.rs`
  - Added `WebGpuBackend` paint backend.
  - Lowers `FillRect` into `cclab_grid_render_webgpu::cell_rect::CellInstance`.
  - Packs cells into f32 arrays for wasm bridge.
  - Preserves text as `WebGpuTextRun` and exports JS text-run payloads.
  - Tracks unsupported `StrokeRect`, `Text`, and `Clip`; text is preserved structurally but still counted unsupported for paint parity.
- Done and tested: `projects/jet/wasm/src/react/webgpu_app.rs`
  - Added wasm32-only `JetWebGpuApp`.
  - Initializes `cclab_grid_wasm::init_renderer`.
  - Repaints initial frame and click-triggered frame.
  - Publishes `window.__jet_webgpu_status` with `phase`, `frames`, `lastCellCount`, `lastTextRunCount`, `lastUnsupportedCount`, `bridgeMode`, and `error`.
  - Calls `renderFrameWithText(cells, textRuns)` when present; falls back to `renderFrame(cells)`.
- Done and tested: `projects/jet/src/wasm_build/config.rs` and `projects/jet/src/wasm_build/mod.rs`
  - Added `[wasm].renderer = "canvas" | "web-gpu"` selector.
  - Threads WebGPU feature selection into generated Cargo scaffold and entrypoint.
- Done and tested: `crates/cclab-grid-wasm/src/renderer_bridge.rs`
  - Added `TextRunWire`.
  - Added `validate_text_runs`.
  - Added `RendererHandle.renderFrameWithText(cells, textRuns)` for wasm32.
  - Tracks `last_text_run_count`.
- Done and tested: `projects/jet/tests/wasm/wasm_build_end_to_end.rs`
  - Added WebGPU scaffold selector smoke.
  - Added runtime browser smoke with Chromium `--enable-unsafe-webgpu`.
  - Added assertion that bridge mode is `text` and text-run count is nonzero.
- Done and tested: specs/managed coverage
  - Updated `.score/tech_design/projects/jet/logic/wasm-renderer-webgpu-backend.md`.
  - Added `.score/tech_design/crates/cclab-grid-wasm/logic/renderer_bridge.md`.
  - `score standardize managed next --scope 'projects/jet/**' --json` reported 255/255 managed.
  - `score standardize managed next --scope 'crates/cclab-grid-wasm/src/renderer_bridge.rs' --json` reported 1/1 managed.
- Done via GitHub metadata:
  - Ran `score wi epicize --project jet --json`; artifact at `/tmp/score/jet/epics/20260515035020-jet-next-phase.md`.
  - Created labels: `epic:wasmview-webgpu`, `epic:tsx-wasm-runtime`, `epic:multi-target`, `epic:dev-reliability`, `epic:bundler-correctness`, `epic:test-runner`.
  - Added issue map comments to #1254, #1236, #1237, #1268.
  - Created #2114 `epic(jet): bundler and transpiler correctness`.
  - Created #2115 `epic(jet): package manager and dev-server reliability`.
  - Applied grouping labels to current open `project:jet` issues.

## Next

- Resume #1254 first. Do not switch to package/dev reliability unless user redirects.
- Start with specs:
  - Inspect `.score/tech_design/projects/jet/logic/wasm-renderer-webgpu-backend.md`.
  - Inspect `.score/tech_design/crates/cclab-grid-wasm/logic/renderer_bridge.md`.
  - Inspect existing text/glyph specs under `.score/tech_design/**` before code.
- Implement next slice: real WebGPU text pass integration.
  - Inspect `crates/cclab-grid-render-webgpu/src/**` for `GlyphInstance`, text WGSL, atlas, and frame builder APIs.
  - Add an integrated text pass path from wasm `renderFrameWithText` into lower WebGPU renderer.
  - Keep `renderFrame(cells)` backward-compatible.
  - Preserve browser status fields and extend them only if needed.
- Suggested commands to begin:
  - `rg -n "GlyphInstance|glyph|atlas|text|FrameBuilder|render_frame|encode_cell_pass" crates/cclab-grid-render-webgpu crates/cclab-grid-wasm projects/jet/wasm`
  - `score wi show 1254`
  - `score td create <new-or-existing-webgpu-text-slice-slug>` if a new TD lifecycle is needed.
- After text is visible, continue #1254 follow-ups in this order:
  - Clip stack / scissor support.
  - `StrokeRect` lowering, likely as four thin rects first.
  - Resize observer / DPR repaint robustness in Jet WebGPU app.
  - Pixel-level browser regression test that proves text is visible, not only counted.
- Backlog grouping now:
  - #1254 `epic:wasmview-webgpu`: #1333, #1334, #1335, #1336.
  - #1236 `epic:tsx-wasm-runtime`: #1235, #1409, #1535.
  - #1237 `epic:multi-target`: #1242.
  - #1268 `epic:test-runner`: #1534.
  - #2115 `epic:dev-reliability`: #1907, #1908, #1930, #1941, #1413.
  - #2114 `epic:bundler-correctness`: #1342, #1344.
  - #1314: leave as `needs-clarify`.

## Criteria

- [ ] `git status --short --branch` shows no unexpected worktree changes except this handoff if it is intentionally uncommitted.
- [ ] `cargo test -p jet-wasm --features webgpu renderer::webgpu` passes.
- [ ] `cargo check -p jet-wasm --features webgpu-app` passes.
- [ ] `PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" cargo check -p jet-wasm --target wasm32-unknown-unknown --features webgpu-app` passes.
- [ ] `cargo test -p jet wasm_build` passes.
- [ ] `cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_build_selects_webgpu_scaffold` passes.
- [ ] `cargo test -p jet --test wasm_build_end_to_end webgpu_renderer_reports_runtime_status_when_available` passes.
- [ ] `cargo test -p cclab-grid-wasm renderer_bridge` passes.
- [ ] `score standardize managed next --scope 'projects/jet/**' --json` reports 100%.
- [ ] `score standardize managed next --scope 'crates/cclab-grid-wasm/src/renderer_bridge.rs' --json` reports 100%.
- [ ] `git diff --check` passes.
- [ ] `gh issue list --label project:jet --state open --limit 100 --json number,title,labels --jq '.[] | [.number, .title, ([.labels[].name] | map(select(startswith("epic:") or .=="type:epic" or startswith("priority:" ) or .=="needs-clarify")) | join(","))] | @tsv'` shows the refreshed epic grouping.
